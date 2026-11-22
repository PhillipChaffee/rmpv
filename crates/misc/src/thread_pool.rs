//! Port of `misc/thread_pool.c`: worker threads that run queued work.
//!
//! A work item is a boxed closure where the C has a function pointer and a
//! `void *`. Threads, counts, the work list, the idle timeout and the
//! shutdown follow the C field by field.

use std::collections::VecDeque;
use std::sync::Arc;

use crate::threads::{Condvar, Mutex, NS_PER_S, lock, mp_time_ns, thread, timedwait_until, wait};

/// Threads destroy themselves after this many seconds without work, if
/// there are more than the minimum (`DESTROY_TIMEOUT`).
pub const DESTROY_TIMEOUT: i64 = 10;

/// `struct work`.
pub type Work = Box<dyn FnOnce() + Send>;

/// The fields of `struct mp_thread_pool` that its `lock` guards.
struct State {
    threads: Vec<thread::JoinHandle<()>>,
    /// Threads that have taken up work and are still running it.
    busy_threads: i32,
    terminate: bool,
    /// The C's `work` array: new work goes in at the front and workers take
    /// it from the back, so it runs in the order it was queued.
    work: VecDeque<Work>,
}

/// `struct mp_thread_pool`, shared by the pool's owner and its workers.
struct Pool {
    min_threads: i32,
    max_threads: i32,
    state: Mutex<State>,
    wakeup: Condvar,
}

/// A thread pool. Dropping it is `talloc_free`: it waits for all queued
/// work and joins the workers.
pub struct MpThreadPool {
    pool: Arc<Pool>,
}

impl std::fmt::Debug for MpThreadPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpThreadPool").finish_non_exhaustive()
    }
}

/// `worker_thread`.
fn worker_thread(pool: &Arc<Pool>) {
    let mut state = lock(&pool.state);
    let mut destroy_deadline: i64 = 0;
    let mut got_timeout = false;
    loop {
        let Some(work) = state.work.pop_back() else {
            if got_timeout || state.terminate {
                break;
            }
            if state.threads.len() as i32 > pool.min_threads {
                if destroy_deadline == 0 {
                    destroy_deadline = mp_time_ns() + DESTROY_TIMEOUT * NS_PER_S;
                }
                let (guard, timed_out) =
                    timedwait_until(&pool.wakeup, &pool.state, state, destroy_deadline);
                state = guard;
                if timed_out {
                    got_timeout = state.threads.len() as i32 > pool.min_threads;
                }
            } else {
                state = wait(&pool.wakeup, state);
            }
            continue;
        };
        state.busy_threads += 1;
        drop(state);

        work();

        state = lock(&pool.state);
        state.busy_threads -= 1;
        destroy_deadline = 0;
        got_timeout = false;
    }
    // Without a termination request, the thread timed out and nobody waits
    // for it: it removes itself.
    if !state.terminate {
        let me = thread::current().id();
        let at = state.threads.iter().position(|t| t.thread().id() == me);
        let Some(at) = at else {
            unreachable!("a worker is always in its pool's list");
        };
        // mp_thread_detach: dropping the handle detaches the thread.
        drop(state.threads.remove(at));
    }
}

/// `thread_pool_dtor`.
impl Drop for MpThreadPool {
    fn drop(&mut self) {
        let mut state = lock(&self.pool.state);
        state.terminate = true;
        self.pool.wakeup.notify_all();
        let threads = std::mem::take(&mut state.threads);
        drop(state);

        for t in threads {
            // A worker ends early only if a work item panicked, which the C
            // has no counterpart for; its panic is not passed on here.
            let _ = t.join();
        }

        let (no_work, no_threads) = {
            let state = lock(&self.pool.state);
            (state.work.is_empty(), state.threads.is_empty())
        };
        assert!(no_work);
        assert!(no_threads);
    }
}

/// `add_thread`, with the pool's lock held as in the C.
fn add_thread(pool: &Arc<Pool>, state: &mut State) -> bool {
    let worker = Arc::clone(pool);
    let spawned = thread::Builder::new()
        .name("worker".to_owned())
        .spawn(move || worker_thread(&worker));
    match spawned {
        Ok(t) => {
            state.threads.push(t);
            true
        }
        Err(_) => false,
    }
}

/// `mp_thread_pool_create`: a pool with `init_threads` threads started now,
/// at least `min_threads` kept and at most `max_threads`. `None` if the
/// initial threads could not be started.
///
/// # Panics
///
/// Unless `0 <= init_threads <= min_threads <= max_threads` and
/// `max_threads > 0`, where the C fails `mp_assert`. (The C does not check
/// `init_threads >= 0`; a negative count starts no thread.)
#[must_use]
pub fn mp_thread_pool_create(
    init_threads: i32,
    min_threads: i32,
    max_threads: i32,
) -> Option<MpThreadPool> {
    assert!(min_threads >= 0);
    assert!(init_threads <= min_threads);
    assert!(max_threads > 0 && max_threads >= min_threads);

    let pool = MpThreadPool {
        pool: Arc::new(Pool {
            min_threads,
            max_threads,
            state: Mutex::new(State {
                threads: Vec::new(),
                busy_threads: 0,
                terminate: false,
                work: VecDeque::new(),
            }),
            wakeup: Condvar::new(),
        }),
    };

    let mut state = lock(&pool.pool.state);
    for _ in 0..init_threads {
        add_thread(&pool.pool, &mut state);
    }
    let ok = state.threads.len() as i32 >= init_threads;
    drop(state);

    if ok { Some(pool) } else { None }
}

/// `thread_pool_add`. On failure the work is handed back, where the C leaves
/// `fn_ctx` with its caller.
fn thread_pool_add(pool: &MpThreadPool, work: Work, allow_queue: bool) -> Result<(), Work> {
    let p = &pool.pool;
    let mut ok = true;
    let mut state = lock(&p.state);
    // If there are not enough threads to run all work at once, but one more
    // may be started, start it. Threads that have not picked up their work
    // yet count as idle, so queued work counts as busy.
    let num_threads = state.threads.len() as i32;
    if state.busy_threads + state.work.len() as i32 + 1 > num_threads
        && num_threads < p.max_threads
        && !add_thread(p, &mut state)
    {
        // Queued work still gets done if there is one thread.
        ok = allow_queue && !state.threads.is_empty();
    }
    let refused = if ok {
        state.work.push_front(work);
        p.wakeup.notify_one();
        None
    } else {
        Some(work)
    };
    drop(state);
    refused.map_or(Ok(()), Err)
}

/// `mp_thread_pool_queue`: runs `work` on a worker thread, queueing it if
/// none is free. It fails only if no worker exists or can be started.
///
/// # Errors
///
/// The work, if it could not be queued.
pub fn mp_thread_pool_queue(pool: &MpThreadPool, work: Work) -> Result<(), Work> {
    thread_pool_add(pool, work, true)
}

/// `mp_thread_pool_run`: like `mp_thread_pool_queue`, but fails instead of
/// queueing when no thread could be started for the work.
///
/// # Errors
///
/// The work, if it was not taken.
pub fn mp_thread_pool_run(pool: &MpThreadPool, work: Work) -> Result<(), Work> {
    thread_pool_add(pool, work, false)
}
