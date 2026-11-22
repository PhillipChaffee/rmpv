//! Port of `misc/dispatch.c`: a queue that lets other threads run callbacks
//! on a target thread, and lock that thread's state.
//!
//! The C stores a callback as a function pointer and a `void *` argument.
//! The port stores a boxed closure, plus a [`DispatchKey`] that stands for
//! the pair where the C compares it: `mp_dispatch_enqueue_notify` merges an
//! item with a queued one of the same key, and `mp_dispatch_cancel_fn`
//! removes items by key. Everything else follows the C: the same fields
//! under one mutex and condition variable, the same waits, the same
//! assertions, and the wakeup and onlock callbacks called where the C calls
//! them, with or without the lock as the C holds it.
//!
//! The setters of the two callbacks take `&mut`: the C says they do no
//! synchronization and must run before other threads see the queue, which
//! is what `&mut` enforces.

use std::any::Any;
use std::collections::VecDeque;

use crate::threads::{
    Condvar, Mutex, MutexGuard, lock, mp_time_ns, mp_time_ns_add, thread, timedwait_until, wait,
};

/// The identity of a callback, which stands for the C's `(fn, fn_data)`.
///
/// `mp_dispatch_enqueue_notify` and `mp_dispatch_cancel_fn` compare both
/// pointers of the pair. Callers choose the two numbers; equal keys mean
/// "the same function on the same data".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchKey {
    /// Stands for `fn`.
    pub func: usize,
    /// Stands for `fn_data`.
    pub data: usize,
}

/// What an item runs. It returns the data that `mp_dispatch_enqueue_autofree`
/// hands over, so that the queue frees it where the C does: after the call,
/// with the lock held again.
type Run = Box<dyn FnOnce() -> Option<Box<dyn Any + Send>> + Send>;

/// The wakeup and onlock callbacks, with the context they close over.
pub type Notify = Box<dyn Fn() + Send + Sync>;

/// `struct mp_dispatch_item`.
struct Item {
    key: DispatchKey,
    run: Run,
    asynchronous: bool,
    mergeable: bool,
    /// For `mp_dispatch_run`: the number its caller waits to see in
    /// `State::completed`, where the C's caller waits for `item.completed`
    /// on its own stack.
    sync_id: u64,
}

/// The fields of `struct mp_dispatch_queue` that its `lock` guards.
#[expect(clippy::struct_excessive_bools, reason = "the C's fields, one for one")]
struct State {
    /// `head` to `tail`.
    items: VecDeque<Item>,
    /// Time at which `mp_dispatch_queue_process` should return.
    wait: i64,
    interrupted: bool,
    in_process: bool,
    in_process_thread_id: Option<thread::ThreadId>,
    locked: bool,
    lock_requests: usize,
    locked_explicit: bool,
    locked_explicit_thread_id: Option<thread::ThreadId>,
    /// `mp_dispatch_run` items that have completed and whose callers have
    /// not seen it yet.
    completed: Vec<u64>,
    next_sync_id: u64,
}

/// `struct mp_dispatch_queue`. Share it between threads with `Arc`.
pub struct MpDispatchQueue {
    state: Mutex<State>,
    cond: Condvar,
    wakeup_fn: Option<Notify>,
    onlock_fn: Option<Notify>,
}

impl std::fmt::Debug for MpDispatchQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpDispatchQueue").finish_non_exhaustive()
    }
}

/// `queue_dtor`: the queue must be empty and idle when it is freed.
///
/// While another thread is in `mp_dispatch_queue_process` or holds the lock
/// it also holds a reference, so in safe Rust only the first assertion can
/// fail.
impl Drop for MpDispatchQueue {
    fn drop(&mut self) {
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(state.items.is_empty());
        assert!(!state.in_process);
        assert!(state.lock_requests == 0);
        assert!(!state.locked);
    }
}

/// `mp_dispatch_create`.
#[must_use]
pub fn mp_dispatch_create() -> MpDispatchQueue {
    MpDispatchQueue {
        state: Mutex::new(State {
            items: VecDeque::new(),
            wait: 0,
            interrupted: false,
            in_process: false,
            in_process_thread_id: None,
            locked: false,
            lock_requests: 0,
            locked_explicit: false,
            locked_explicit_thread_id: None,
            completed: Vec::new(),
            next_sync_id: 0,
        }),
        cond: Condvar::new(),
        wakeup_fn: None,
        onlock_fn: None,
    }
}

/// `mp_dispatch_set_wakeup_fn`. `None` removes the callback, as a NULL
/// function does in C.
pub fn mp_dispatch_set_wakeup_fn(queue: &mut MpDispatchQueue, wakeup_fn: Option<Notify>) {
    queue.wakeup_fn = wakeup_fn;
}

/// `mp_dispatch_set_onlock_fn`.
pub fn mp_dispatch_set_onlock_fn(queue: &mut MpDispatchQueue, onlock_fn: Option<Notify>) {
    queue.onlock_fn = onlock_fn;
}

/// `mp_dispatch_append`. An item of `mp_dispatch_run` gets its number here,
/// and the number is returned.
fn mp_dispatch_append(queue: &MpDispatchQueue, mut item: Item) -> u64 {
    let mut state = lock(&queue.state);
    if item.mergeable {
        let mut merged = false;
        for cur in &state.items {
            if cur.mergeable && cur.key == item.key {
                merged = true;
                break;
            }
        }
        if merged {
            // The C frees the new item, with the lock held.
            drop(item);
            drop(state);
            return 0;
        }
    }
    if !item.asynchronous {
        state.next_sync_id += 1;
        item.sync_id = state.next_sync_id;
    }
    let id = item.sync_id;
    state.items.push_back(item);
    // Other threads may wait on this condition for other reasons, so all
    // are woken.
    queue.cond.notify_all();
    // Without a wakeup callback, mp_dispatch_queue_process is interrupted
    // instead.
    if queue.wakeup_fn.is_none() {
        state.interrupted = true;
    }
    drop(state);
    if let Some(wakeup) = &queue.wakeup_fn {
        wakeup();
    }
    id
}

fn asynchronous_item(key: DispatchKey, run: Run, mergeable: bool) -> Item {
    Item {
        key,
        run,
        asynchronous: true,
        mergeable,
        sync_id: 0,
    }
}

/// `mp_dispatch_enqueue`: runs `f` on the target thread when it next
/// enters `mp_dispatch_queue_process`.
pub fn mp_dispatch_enqueue(
    queue: &MpDispatchQueue,
    key: DispatchKey,
    f: impl FnOnce() + Send + 'static,
) {
    let run: Run = Box::new(move || {
        f();
        None
    });
    mp_dispatch_append(queue, asynchronous_item(key, run, false));
}

/// `mp_dispatch_enqueue_autofree`: like `mp_dispatch_enqueue`, and the queue
/// owns `data` (the C's `fn_data`) and frees it after `f` has run, or when
/// the item is cancelled.
pub fn mp_dispatch_enqueue_autofree<T: Send + 'static>(
    queue: &MpDispatchQueue,
    key: DispatchKey,
    data: T,
    f: impl FnOnce(&mut T) + Send + 'static,
) {
    let run: Run = Box::new(move || {
        let mut data = data;
        f(&mut data);
        Some(Box::new(data))
    });
    mp_dispatch_append(queue, asynchronous_item(key, run, false));
}

/// `mp_dispatch_enqueue_notify`: like `mp_dispatch_enqueue`, but dropped if
/// an item enqueued this way with the same key is still queued.
pub fn mp_dispatch_enqueue_notify(
    queue: &MpDispatchQueue,
    key: DispatchKey,
    f: impl FnOnce() + Send + 'static,
) {
    let run: Run = Box::new(move || {
        f();
        None
    });
    mp_dispatch_append(queue, asynchronous_item(key, run, true));
}

/// `mp_dispatch_cancel_fn`: removes every queued item with this key.
///
/// Like the C, it removes items of every kind. Cancelling an item of
/// `mp_dispatch_run` frees an item on its caller's stack in C, which is
/// undefined; here that caller waits for ever.
pub fn mp_dispatch_cancel_fn(queue: &MpDispatchQueue, key: DispatchKey) {
    let mut state = lock(&queue.state);
    let mut kept = VecDeque::with_capacity(state.items.len());
    let mut removed = Vec::new();
    while let Some(cur) = state.items.pop_front() {
        if cur.key == key {
            removed.push(cur);
        } else {
            kept.push_back(cur);
        }
    }
    state.items = kept;
    // The C frees each removed item, and the data of an autofree one, with
    // the lock held.
    drop(removed);
    drop(state);
}

/// `mp_dispatch_run`: runs `f` on the target thread and waits until it has.
///
/// The closure must be `'static`, where the C can pass data on the
/// caller's stack: the queue holds it with no lifetime, and safe Rust
/// cannot lend it one that ends when this function returns.
pub fn mp_dispatch_run(
    queue: &MpDispatchQueue,
    key: DispatchKey,
    f: impl FnOnce() + Send + 'static,
) {
    let run: Run = Box::new(move || {
        f();
        None
    });
    let id = mp_dispatch_append(
        queue,
        Item {
            key,
            run,
            asynchronous: false,
            mergeable: false,
            sync_id: 0,
        },
    );
    let mut state = lock(&queue.state);
    loop {
        if let Some(at) = state.completed.iter().position(|&c| c == id) {
            state.completed.swap_remove(at);
            break;
        }
        state = wait(&queue.cond, state);
    }
}

/// `mp_dispatch_queue_process`: runs queued items on this thread and lets
/// `mp_dispatch_lock` callers in.
///
/// It returns once `timeout` seconds have passed (at once for
/// `timeout <= 0`), the queue is empty and no lock is held.
///
/// # Panics
///
/// If called again while this queue is in it, where the C fails
/// `mp_assert`.
pub fn mp_dispatch_queue_process(queue: &MpDispatchQueue, timeout: f64) {
    let mut state = lock(&queue.state);
    state.wait = if timeout > 0.0 {
        mp_time_ns_add(mp_time_ns(), timeout)
    } else {
        0
    };
    // Recursion is not allowed.
    assert!(!state.in_process);
    state.in_process = true;
    state.in_process_thread_id = Some(thread::current().id());
    // Wake up the thread that called mp_dispatch_lock.
    if state.lock_requests > 0 {
        queue.cond.notify_all();
    }
    loop {
        if state.lock_requests > 0 {
            // Held by mp_dispatch_lock.
            state = wait(&queue.cond, state);
        } else if let Some(item) = state.items.pop_front() {
            state = run_item(queue, state, item);
        } else if state.wait > 0 && !state.interrupted {
            let until = state.wait;
            let (guard, timed_out) = timedwait_until(&queue.cond, &queue.state, state, until);
            state = guard;
            if timed_out {
                state.wait = 0;
            }
        } else {
            break;
        }
    }
    assert!(!state.locked);
    state.in_process = false;
    state.interrupted = false;
}

/// One item of `mp_dispatch_queue_process`. The lock is released while the
/// item runs, so other threads can queue more, and `locked` keeps
/// `mp_dispatch_lock` from returning meanwhile.
fn run_item<'a>(
    queue: &'a MpDispatchQueue,
    mut state: MutexGuard<'a, State>,
    item: Item,
) -> MutexGuard<'a, State> {
    assert!(!state.locked);
    state.locked = true;
    drop(state);

    let freed = (item.run)();

    let mut state = lock(&queue.state);
    assert!(state.locked);
    state.locked = false;
    // Wakes mp_dispatch_run, and mp_dispatch_lock.
    queue.cond.notify_all();
    if item.asynchronous {
        drop(freed);
    } else {
        state.completed.push(item.sync_id);
    }
    state
}

#[expect(
    clippy::significant_drop_tightening,
    reason = "the C signals the condition while it holds the lock"
)]
/// `mp_dispatch_interrupt`: makes a running `mp_dispatch_queue_process`
/// return once the queue is empty, or the next call return at once.
pub fn mp_dispatch_interrupt(queue: &MpDispatchQueue) {
    let mut state = lock(&queue.state);
    state.interrupted = true;
    queue.cond.notify_all();
}

#[expect(
    clippy::significant_drop_tightening,
    reason = "the C signals the condition while it holds the lock"
)]
/// `mp_dispatch_adjust_timeout`: if `mp_dispatch_queue_process` is running
/// and would wait past `until` (on the [`mp_time_ns`] clock), makes it wait
/// until `until` only.
pub fn mp_dispatch_adjust_timeout(queue: &MpDispatchQueue, until: i64) {
    let mut state = lock(&queue.state);
    if state.in_process && state.wait > until {
        state.wait = until;
        queue.cond.notify_all();
    }
}

/// `mp_dispatch_lock`: waits until the target thread is in
/// `mp_dispatch_queue_process` and nothing else holds it, then holds it.
///
/// # Panics
///
/// If called from the target thread inside `mp_dispatch_queue_process` (a
/// dispatched callback), or again from the thread that holds the lock;
/// the C fails `mp_assert` on both.
pub fn mp_dispatch_lock(queue: &MpDispatchQueue) {
    let me = thread::current().id();
    let mut state = lock(&queue.state);
    if state.in_process {
        assert!(state.in_process_thread_id != Some(me));
    }
    if state.locked_explicit {
        assert!(state.locked_explicit_thread_id != Some(me));
    }
    state.lock_requests += 1;
    // Wait until the target thread is trapped in mp_dispatch_queue_process.
    // The onlock callback runs with the lock held, the wakeup callback
    // without.
    if let Some(onlock) = &queue.onlock_fn {
        onlock();
    }
    while !state.in_process {
        drop(state);
        if let Some(wakeup) = &queue.wakeup_fn {
            wakeup();
        }
        state = lock(&queue.state);
        if state.in_process {
            break;
        }
        state = wait(&queue.cond, state);
    }
    // Wait until the lock is free.
    while !state.in_process || state.locked {
        state = wait(&queue.cond, state);
    }
    assert!(state.lock_requests > 0);
    assert!(!state.locked);
    assert!(!state.locked_explicit);
    state.locked = true;
    state.locked_explicit = true;
    state.locked_explicit_thread_id = Some(me);
}

#[expect(
    clippy::significant_drop_tightening,
    reason = "the C signals the condition while it holds the lock"
)]
/// `mp_dispatch_unlock`.
///
/// # Panics
///
/// If this thread does not hold the lock, where the C fails `mp_assert`.
pub fn mp_dispatch_unlock(queue: &MpDispatchQueue) {
    let mut state = lock(&queue.state);
    assert!(state.locked);
    assert!(state.locked_explicit);
    assert!(state.locked_explicit_thread_id == Some(thread::current().id()));
    state.locked = false;
    state.locked_explicit = false;
    state.lock_requests -= 1;
    // Wakes mp_dispatch_queue_process, and maybe other mp_dispatch_lock
    // callers.
    queue.cond.notify_all();
}
