//! Port of `misc/thread_tools.c`: `mp_waiter`, a one-shot semaphore, and
//! `mp_cancel`, a flag that signals a tree of objects, callbacks and a
//! pollable file descriptor.

use std::sync::PoisonError;
use std::sync::{Arc, Weak};

use crate::os::WakeupPipe;
use crate::threads::{
    AtomicBool, Condvar, Mutex, Ordering, lock, mp_time_ns, mp_time_ns_add, timedwait_until,
};

/// `struct mp_waiter`: a one-shot semaphore that carries a value.
/// `MpWaiter::new()` is `MP_WAITER_INITIALIZER`.
#[cfg_attr(not(aeneas), derive(Debug, Default))]
pub struct MpWaiter {
    /// `done` and `value`, which the C guards with `lock`.
    state: Mutex<(bool, usize)>,
    wakeup: Condvar,
}

impl MpWaiter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new((false, 0)),
            wakeup: Condvar::new(),
        }
    }
}

/// `mp_waiter_wait`: blocks until `mp_waiter_wakeup` and returns its value.
///
/// The C destroys the waiter here and fills it with 0xCA. The port leaves
/// it as it is; like the C's, it must not be waited on again.
pub fn mp_waiter_wait(waiter: &MpWaiter) -> usize {
    // A panic cannot leave the pair half-written, so a poisoned lock still
    // holds a consistent state.
    let mut state = waiter.state.lock().unwrap_or_else(PoisonError::into_inner);
    while !state.0 {
        state = waiter
            .wakeup
            .wait(state)
            .unwrap_or_else(PoisonError::into_inner);
    }
    state.1
}

/// `mp_waiter_wakeup`.
///
/// # Panics
///
/// If the waiter was already woken, where the C fails `mp_assert`.
#[expect(
    clippy::significant_drop_tightening,
    reason = "the C signals the condition while it holds the lock"
)]
pub fn mp_waiter_wakeup(waiter: &MpWaiter, value: usize) {
    let mut state = waiter.state.lock().unwrap_or_else(PoisonError::into_inner);
    assert!(!state.0);
    *state = (true, value);
    waiter.wakeup.notify_one();
}

/// `mp_waiter_poll`: whether the waiter was woken.
#[must_use]
pub fn mp_waiter_poll(waiter: &MpWaiter) -> bool {
    waiter
        .state
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .0
}

/// The fields of `struct mp_cancel` that its `lock` guards.
struct CancelState {
    /// `cb` and `cb_ctx`. It may be called more than once, so it is `Fn`.
    cb: Option<Box<dyn Fn() + Send + Sync>>,
    /// `wakeup_pipe`, made by the first `mp_cancel_get_fd`. The C keeps
    /// `{-1, -1}` until then, and after a failed attempt.
    wakeup_pipe: Option<WakeupPipe>,
    /// `slaves`, in the order they were added. A slave keeps its parent
    /// alive, so the parent holds only a weak reference back.
    slaves: Vec<Weak<MpCancel>>,
}

/// `struct mp_cancel`. Share it with `Arc`; [`mp_cancel_new`] returns one.
pub struct MpCancel {
    lock: Mutex<CancelState>,
    wakeup: Condvar,
    triggered: AtomicBool,
    /// `parent`. The C reads and writes it without a lock, as calls of
    /// `mp_cancel_set_parent` on one slave must not overlap; the mutex here
    /// is never contended.
    parent: Mutex<Option<Arc<Self>>>,
}

impl std::fmt::Debug for MpCancel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MpCancel").finish_non_exhaustive()
    }
}

/// `cancel_destroy`: runs when the last reference goes.
///
/// A slave holds a reference to its parent, so a parent cannot be freed
/// while it has slaves, the API error the C asserts against.
impl Drop for MpCancel {
    fn drop(&mut self) {
        let me: *const Self = self;
        let state = self.lock.get_mut().unwrap_or_else(PoisonError::into_inner);
        assert!(state.slaves.is_empty());
        // mp_cancel_set_parent(c, NULL), for an object no Arc points to.
        let parent = self
            .parent
            .get_mut()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(parent) = parent {
            let mut ps = lock(&parent.lock);
            ps.slaves.retain(|w| !std::ptr::eq(w.as_ptr(), me));
        }
        // Dropping the pipe closes both ends.
    }
}

/// `mp_cancel_new`.
#[must_use]
pub fn mp_cancel_new() -> Arc<MpCancel> {
    Arc::new(MpCancel {
        lock: Mutex::new(CancelState {
            cb: None,
            wakeup_pipe: None,
            slaves: Vec::new(),
        }),
        wakeup: Condvar::new(),
        triggered: AtomicBool::new(false),
        parent: Mutex::new(None),
    })
}

/// `trigger_locked`: sets the flag and signals every mechanism, with `c`'s
/// lock held: waiters, the callback, the slaves (each under its own lock,
/// taken while this one is held), and one byte into the pipe.
fn trigger_locked(c: &MpCancel, state: &CancelState) {
    c.triggered.store(true, Ordering::SeqCst);
    // The condition is bound to `triggered`.
    c.wakeup.notify_all();
    if let Some(cb) = &state.cb {
        cb();
    }
    for sub in &state.slaves {
        // A slave whose last reference is going is unlinking itself; it can
        // no longer be observed.
        if let Some(sub) = sub.upgrade() {
            mp_cancel_trigger(&sub);
        }
    }
    if let Some(pipe) = &state.wakeup_pipe {
        // A full pipe is signalled already; the C ignores the result too.
        pipe.write_byte();
    }
}

/// `mp_cancel_trigger`: requests abort.
pub fn mp_cancel_trigger(c: &MpCancel) {
    let state = lock(&c.lock);
    trigger_locked(c, &state);
}

/// `mp_cancel_reset`: clears the flag and drains the pipe. It leaves the
/// slaves as they are.
pub fn mp_cancel_reset(c: &MpCancel) {
    let state = lock(&c.lock);
    c.triggered.store(false, Ordering::SeqCst);
    if let Some(pipe) = &state.wakeup_pipe {
        pipe.drain();
    }
}

/// `mp_cancel_test`: whether the caller should abort. `None` stands for the
/// NULL the C allows.
#[must_use]
pub fn mp_cancel_test(c: Option<&MpCancel>) -> bool {
    c.is_some_and(|c| c.triggered.load(Ordering::Relaxed))
}

/// `mp_cancel_wait`: waits until `c` is triggered or `timeout` seconds
/// have passed, and returns whether it is triggered.
///
/// The C's header says a negative `timeout` waits for ever, but its code
/// computes a deadline in the past from it, so the call returns at once.
/// The port does the same.
#[must_use]
pub fn mp_cancel_wait(c: &MpCancel, timeout: f64) -> bool {
    let wait_until = mp_time_ns_add(mp_time_ns(), timeout);
    let mut state = lock(&c.lock);
    while !mp_cancel_test(Some(c)) {
        let (guard, timed_out) = timedwait_until(&c.wakeup, &c.lock, state, wait_until);
        state = guard;
        if timed_out {
            break;
        }
    }
    drop(state);
    mp_cancel_test(Some(c))
}

/// `retrigger_locked`: if `c` is triggered, signals every mechanism again,
/// so that one just added is signalled too.
fn retrigger_locked(c: &MpCancel, state: &CancelState) {
    if mp_cancel_test(Some(c)) {
        trigger_locked(c, state);
    }
}

/// `mp_cancel_set_cb`: sets or, with `None`, removes the callback.
///
/// The callback runs at once if `c` is triggered, and on every later
/// trigger, with `c`'s lock held (and whatever locks the caller of the
/// trigger holds).
#[expect(
    clippy::significant_drop_tightening,
    reason = "the C retriggers while it holds the lock"
)]
pub fn mp_cancel_set_cb(c: &MpCancel, cb: Option<Box<dyn Fn() + Send + Sync>>) {
    let mut state = lock(&c.lock);
    state.cb = cb;
    retrigger_locked(c, &state);
}

/// `mp_cancel_set_parent`: makes a trigger of `parent` trigger `slave`
/// too, or, with `None`, ends the link.
///
/// If `parent` is triggered already, all its mechanisms are signalled
/// again, its own callback included, as in the C.
///
/// Calls for one slave must not overlap, as in the C. A loop of parents
/// recurses without end on a trigger, in the C and here.
#[expect(
    clippy::significant_drop_tightening,
    reason = "the C retriggers while it holds the lock"
)]
pub fn mp_cancel_set_parent(slave: &Arc<MpCancel>, parent: Option<&Arc<MpCancel>>) {
    let mut current = lock(&slave.parent);
    let same = match (current.as_ref(), parent) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    };
    if same {
        return;
    }
    if let Some(old) = current.take() {
        let mut ps = lock(&old.lock);
        ps.slaves
            .retain(|w| !std::ptr::eq(w.as_ptr(), Arc::as_ptr(slave)));
    }
    *current = parent.cloned();
    if let Some(new) = current.as_ref() {
        let mut ps = lock(&new.lock);
        ps.slaves.push(Arc::downgrade(slave));
        retrigger_locked(new, &ps);
    }
}

/// `mp_cancel_get_fd`: a file descriptor that is readable while `c` is
/// triggered, made on the first call; -1 if it could not be made. Callers
/// poll it and must not read it.
#[must_use]
pub fn mp_cancel_get_fd(c: &MpCancel) -> i32 {
    let mut state = lock(&c.lock);
    if state.wakeup_pipe.is_none() {
        state.wakeup_pipe = WakeupPipe::new();
        retrigger_locked(c, &state);
    }
    state.wakeup_pipe.as_ref().map_or(-1, WakeupPipe::read_fd)
}
