//! Port of the `mp_waiter` part of `misc/thread_tools.c`.
//!
//! `mp_cancel` is not ported yet.

use std::sync::PoisonError;

use crate::threads::{Condvar, Mutex};

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
