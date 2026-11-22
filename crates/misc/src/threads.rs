//! What the threaded files of `misc/` use from mpv's `osdep/`: the clock and
//! deadline arithmetic of `osdep/timer.c`, and the mutex, condition variable
//! and timed wait of `osdep/threads-posix.h`.
//!
//! With `--cfg loom` the synchronization types come from the `loom` crate,
//! so that `tests/loom.rs` can run the ports under every thread schedule
//! loom explores. Without it they are the standard library's.
//!
//! Charon is told to leave this module out: `Mutex` and `Condvar` are
//! outside what Aeneas translates.

use std::sync::PoisonError;
#[cfg(not(loom))]
use std::time::Duration;
use std::time::Instant;

#[cfg(loom)]
pub(crate) use loom::sync::atomic::{AtomicBool, Ordering};
#[cfg(loom)]
pub(crate) use loom::sync::{Condvar, Mutex, MutexGuard};
#[cfg(loom)]
pub(crate) use loom::thread;
#[cfg(not(loom))]
pub(crate) use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(loom))]
pub(crate) use std::sync::{Condvar, Mutex, MutexGuard};
#[cfg(not(loom))]
pub(crate) use std::thread;

/// `mp_mutex_lock`. Every lock in the port guards state that no panic can
/// leave half-written, so a poisoned lock still holds a consistent state.
pub(crate) fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// `mp_cond_wait`.
pub(crate) fn wait<'a, T>(cond: &Condvar, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
    cond.wait(guard).unwrap_or_else(PoisonError::into_inner)
}

/// One second in `mp_time_ns` units (`MP_TIME_S_TO_NS(1)`).
pub const NS_PER_S: i64 = 1_000_000_000;

/// `mp_cond_timedwait_until`: waits until `until` on the `mp_time_ns` clock
/// at the latest, and returns whether it timed out (the C returns
/// `ETIMEDOUT`). As in mpv, a wait of more than 1000 days waits without a
/// timeout.
///
/// loom does not model time: under `--cfg loom` a finite wait releases
/// `mutex`, lets any other thread run, takes `mutex` again and times out,
/// which is one of the schedules the real wait allows.
pub(crate) fn timedwait_until<'a, T>(
    cond: &Condvar,
    mutex: &'a Mutex<T>,
    guard: MutexGuard<'a, T>,
    until: i64,
) -> (MutexGuard<'a, T>, bool) {
    let timeout = until.saturating_sub(mp_time_ns()).max(0);
    if timeout > 1000 * 24 * 60 * 60 * NS_PER_S {
        return (wait(cond, guard), false);
    }
    timed_wait(cond, mutex, guard, timeout as u64)
}

#[cfg(not(loom))]
fn timed_wait<'a, T>(
    cond: &Condvar,
    _mutex: &'a Mutex<T>,
    guard: MutexGuard<'a, T>,
    timeout_ns: u64,
) -> (MutexGuard<'a, T>, bool) {
    let (guard, result) = cond
        .wait_timeout(guard, Duration::from_nanos(timeout_ns))
        .unwrap_or_else(PoisonError::into_inner);
    (guard, result.timed_out())
}

#[cfg(loom)]
fn timed_wait<'a, T>(
    _cond: &Condvar,
    mutex: &'a Mutex<T>,
    guard: MutexGuard<'a, T>,
    _timeout_ns: u64,
) -> (MutexGuard<'a, T>, bool) {
    drop(guard);
    thread::yield_now();
    (lock(mutex), true)
}

/// The start of `mp_time_ns`'s clock: the first call, as mpv's
/// `mp_time_init` takes the clock when mpv starts.
static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// `mp_time_ns`: nanoseconds on a monotonic clock since the first call,
/// plus one. mpv's is relative to `mp_time_init` and is asserted to be
/// positive; the plus one makes the first call so too.
#[must_use]
pub fn mp_time_ns() -> i64 {
    let start = *START.get_or_init(Instant::now);
    let elapsed = Instant::now().saturating_duration_since(start);
    i64::try_from(elapsed.as_nanos()).unwrap_or(i64::MAX - 1) + 1
}

/// 2^63, the C's `0x1p63`.
const TWO_63: f64 = 9_223_372_036_854_775_808.0;

/// `mp_time_ns_add`: `time_ns` plus `timeout_sec` seconds, clamped to
/// `1..=i64::MAX`.
///
/// # Panics
///
/// If `time_ns` is not positive, where the C fails `mp_assert`.
#[must_use]
pub fn mp_time_ns_add(time_ns: i64, timeout_sec: f64) -> i64 {
    assert!(time_ns > 0);
    let scaled = timeout_sec * 1e9;
    // MPCLAMP, which leaves NaN as it is, as f64::clamp does.
    let t = scaled.clamp(-TWO_63, TWO_63);
    let ti: i64 = if t == TWO_63 {
        i64::MAX
    } else if t.is_nan() {
        // Converting NaN to int64_t is undefined in C; x86-64's cvttsd2si,
        // which the compiled C uses, gives INT64_MIN.
        i64::MIN
    } else {
        t as i64
    };
    if ti > i64::MAX - time_ns {
        return i64::MAX;
    }
    if ti <= -time_ns {
        return 1;
    }
    time_ns + ti
}
