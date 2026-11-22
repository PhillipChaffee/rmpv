//! Schedule exploration of the threaded ports with loom.
//!
//! Run with `make loom`, which builds the crate with `--cfg loom`: the
//! port's mutexes, condition variables, atomics and threads are then loom's,
//! and each `loom::model` below runs its body under every thread schedule
//! loom's exploration reaches (all of them, up to the preemption bound
//! where one is set). A deadlock, a lost wakeup or a failed assertion in
//! any schedule fails the test.
//!
//! loom does not model time. A wait without timeout is what it is; a wait
//! with a finite timeout releases the lock, lets other threads run and
//! then times out (see `rmpv_misc::threads`). The models therefore use
//! timeouts of "none" (longer than 1000 days) or 0.

// Without --cfg loom the file holds no test.
#[cfg(not(loom))]
use rmpv_misc as _;

#[cfg(loom)]
mod models {

    use std::sync::Arc;

    use loom::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use loom::thread;
    use rmpv_misc::dispatch::{self, DispatchKey, MpDispatchQueue};
    use rmpv_misc::thread_pool;
    use rmpv_misc::thread_tools::{self, MpWaiter};

    /// Runs `f` under loom's exploration with `builder` and prints how many
    /// schedules it ran.
    fn explore(name: &str, builder: &loom::model::Builder, f: impl Fn() + Sync + Send + 'static) {
        static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let before = RUNS.load(std::sync::atomic::Ordering::SeqCst);
        builder.check(move || {
            RUNS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            f();
        });
        let runs = RUNS.load(std::sync::atomic::Ordering::SeqCst) - before;
        println!("loom {name}: {runs} schedules explored, all pass");
    }

    /// The default exploration: every schedule.
    fn every() -> loom::model::Builder {
        loom::model::Builder::new()
    }

    /// Exploration with at most `n` preemptions per schedule.
    fn bounded(n: usize) -> loom::model::Builder {
        let mut b = loom::model::Builder::new();
        b.preemption_bound = Some(n);
        b
    }

    /// A timeout that `mp_cond_timedwait` treats as none.
    const FOREVER: f64 = 1e300;

    const KEY: DispatchKey = DispatchKey { func: 1, data: 1 };

    /// A queue with a wakeup callback that does nothing, so that enqueueing
    /// does not interrupt `mp_dispatch_queue_process`.
    fn queue() -> Arc<MpDispatchQueue> {
        let mut q = dispatch::mp_dispatch_create();
        dispatch::mp_dispatch_set_wakeup_fn(&mut q, Some(Box::new(|| {})));
        Arc::new(q)
    }

    /// A target thread that processes until interrupted.
    fn target(q: &Arc<MpDispatchQueue>) -> thread::JoinHandle<()> {
        let q = Arc::clone(q);
        thread::spawn(move || dispatch::mp_dispatch_queue_process(&q, FOREVER))
    }

    /// `mp_waiter_wakeup` against `mp_waiter_wait` and `mp_waiter_poll`: the
    /// waiter always gets the value, and a poll after the wakeup sees it.
    #[test]
    fn waiter() {
        explore("waiter", &every(), || {
            let w = Arc::new(MpWaiter::new());
            let w2 = Arc::clone(&w);
            let waker = thread::spawn(move || {
                let before = thread_tools::mp_waiter_poll(&w2);
                thread_tools::mp_waiter_wakeup(&w2, 42);
                assert!(thread_tools::mp_waiter_poll(&w2));
                before
            });
            assert_eq!(thread_tools::mp_waiter_wait(&w), 42);
            let _ = waker.join().unwrap();
        });
    }

    /// `mp_dispatch_lock` against a queued callback: the callback never runs
    /// while the lock is held, it runs once, and the target returns once
    /// interrupted.
    #[test]
    fn dispatch_lock_excludes_callbacks() {
        explore("dispatch_lock_excludes_callbacks", &every(), || {
            let q = queue();
            let in_lock = Arc::new(AtomicBool::new(false));
            let ran = Arc::new(AtomicUsize::new(0));
            let t = target(&q);
            let (l, r) = (Arc::clone(&in_lock), Arc::clone(&ran));
            dispatch::mp_dispatch_enqueue(&q, KEY, move || {
                assert!(!l.load(Ordering::SeqCst));
                r.fetch_add(1, Ordering::SeqCst);
            });
            dispatch::mp_dispatch_lock(&q);
            in_lock.store(true, Ordering::SeqCst);
            in_lock.store(false, Ordering::SeqCst);
            dispatch::mp_dispatch_unlock(&q);
            dispatch::mp_dispatch_interrupt(&q);
            t.join().unwrap();
            assert_eq!(ran.load(Ordering::SeqCst), 1);
        });
    }

    /// Two threads take `mp_dispatch_lock` at once: they exclude each other,
    /// and both get it.
    #[test]
    fn dispatch_lock_two_lockers() {
        explore("dispatch_lock_two_lockers", &bounded(3), || {
            let q = queue();
            let held = Arc::new(AtomicUsize::new(0));
            let t = target(&q);
            let lockers: Vec<_> = (0..2)
                .map(|_| {
                    let (q, held) = (Arc::clone(&q), Arc::clone(&held));
                    thread::spawn(move || {
                        dispatch::mp_dispatch_lock(&q);
                        assert_eq!(held.fetch_add(1, Ordering::SeqCst), 0);
                        held.fetch_sub(1, Ordering::SeqCst);
                        dispatch::mp_dispatch_unlock(&q);
                    })
                })
                .collect();
            for l in lockers {
                l.join().unwrap();
            }
            dispatch::mp_dispatch_interrupt(&q);
            t.join().unwrap();
        });
    }

    /// `mp_dispatch_run` returns only after its callback ran, on the target
    /// thread.
    #[test]
    fn dispatch_run_completes_on_target() {
        explore("dispatch_run_completes_on_target", &every(), || {
            let q = queue();
            let t = target(&q);
            let target_id = t.thread().id();
            let ran_on = Arc::new(loom::sync::Mutex::new(None));
            let r = Arc::clone(&ran_on);
            dispatch::mp_dispatch_run(&q, KEY, move || {
                *r.lock().unwrap() = Some(thread::current().id());
            });
            assert_eq!(*ran_on.lock().unwrap(), Some(target_id));
            dispatch::mp_dispatch_interrupt(&q);
            t.join().unwrap();
        });
    }

    /// `mp_dispatch_lock` while the target is outside
    /// `mp_dispatch_queue_process`: the onlock callback runs once, the wakeup
    /// callback brings the target in, and the lock is granted.
    #[test]
    fn dispatch_lock_wakes_target() {
        explore("dispatch_lock_wakes_target", &every(), || {
            let asked = Arc::new((loom::sync::Mutex::new(false), loom::sync::Condvar::new()));
            let onlock = Arc::new(AtomicUsize::new(0));
            let mut q = dispatch::mp_dispatch_create();
            let a = Arc::clone(&asked);
            dispatch::mp_dispatch_set_wakeup_fn(
                &mut q,
                Some(Box::new(move || {
                    *a.0.lock().unwrap() = true;
                    a.1.notify_all();
                })),
            );
            let o = Arc::clone(&onlock);
            dispatch::mp_dispatch_set_onlock_fn(
                &mut q,
                Some(Box::new(move || {
                    o.fetch_add(1, Ordering::SeqCst);
                })),
            );
            let q = Arc::new(q);
            let (q2, a) = (Arc::clone(&q), Arc::clone(&asked));
            let t = thread::spawn(move || {
                let mut asked = a.0.lock().unwrap();
                while !*asked {
                    asked = a.1.wait(asked).unwrap();
                }
                drop(asked);
                dispatch::mp_dispatch_queue_process(&q2, 0.0);
            });
            dispatch::mp_dispatch_lock(&q);
            dispatch::mp_dispatch_unlock(&q);
            t.join().unwrap();
            assert_eq!(onlock.load(Ordering::SeqCst), 1);
        });
    }

    /// `mp_dispatch_interrupt` against a target that is about to wait: the
    /// target always returns.
    #[test]
    fn dispatch_interrupt_not_lost() {
        explore("dispatch_interrupt_not_lost", &every(), || {
            let q = queue();
            let t = target(&q);
            dispatch::mp_dispatch_interrupt(&q);
            t.join().unwrap();
        });
    }

    /// A pool of at most one thread whose idle timeout may fire at any point:
    /// both items run exactly once and freeing the pool returns.
    #[test]
    fn thread_pool_idle_exit_vs_new_work() {
        explore("thread_pool_idle_exit_vs_new_work", &bounded(3), || {
            let ran = Arc::new(AtomicUsize::new(0));
            let pool = thread_pool::mp_thread_pool_create(0, 0, 1).unwrap();
            for _ in 0..2 {
                let r = Arc::clone(&ran);
                let queued = thread_pool::mp_thread_pool_queue(
                    &pool,
                    Box::new(move || {
                        r.fetch_add(1, Ordering::SeqCst);
                    }),
                );
                assert!(queued.is_ok());
            }
            drop(pool);
            assert_eq!(ran.load(Ordering::SeqCst), 2);
        });
    }

    /// A pool with one permanent thread: queued work runs before the pool is
    /// freed, whatever the order of the worker's wait and the shutdown.
    #[test]
    fn thread_pool_shutdown_runs_work() {
        explore("thread_pool_shutdown_runs_work", &every(), || {
            let ran = Arc::new(AtomicUsize::new(0));
            let pool = thread_pool::mp_thread_pool_create(1, 1, 1).unwrap();
            let r = Arc::clone(&ran);
            assert!(
                thread_pool::mp_thread_pool_run(
                    &pool,
                    Box::new(move || {
                        r.fetch_add(1, Ordering::SeqCst);
                    })
                )
                .is_ok()
            );
            drop(pool);
            assert_eq!(ran.load(Ordering::SeqCst), 1);
        });
    }

    /// Queued work runs without the pool being freed: the worker is woken for
    /// it.
    #[test]
    fn thread_pool_wakes_worker() {
        explore("thread_pool_wakes_worker", &every(), || {
            let done = Arc::new((loom::sync::Mutex::new(false), loom::sync::Condvar::new()));
            let pool = thread_pool::mp_thread_pool_create(1, 1, 1).unwrap();
            let d = Arc::clone(&done);
            let queued = thread_pool::mp_thread_pool_queue(
                &pool,
                Box::new(move || {
                    *d.0.lock().unwrap() = true;
                    d.1.notify_all();
                }),
            );
            assert!(queued.is_ok());
            let mut finished = done.0.lock().unwrap();
            while !*finished {
                finished = done.1.wait(finished).unwrap();
            }
            drop(finished);
            drop(pool);
        });
    }
}
