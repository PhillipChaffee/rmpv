//! Port of `misc/rendezvous.c`.

use std::sync::{Condvar, Mutex, PoisonError};

/// One blocked caller: its tag, and the value it brought until its partner
/// replaces that with its own and sets `done`.
struct Waiter {
    tag: usize,
    value: isize,
    done: bool,
    id: u64,
}

/// The C's file-level `lock`, `waiters` list and a counter that names list
/// entries, where the C links stack-allocated structs by address.
static WAITERS: Mutex<(Vec<Waiter>, u64)> = Mutex::new((Vec::new(), 0));
static WAKEUP: Condvar = Condvar::new();

/// `mp_rendezvous`: blocks until a second caller arrives with the same
/// `tag`; each then returns the other's `value`.
///
/// `tag` is a pointer in C, used only as a key. As in the C, more than two
/// callers on one tag pair up in an unspecified way.
pub fn mp_rendezvous(tag: usize, value: isize) -> isize {
    // A panic cannot leave the list half-updated, so a poisoned lock still
    // holds a consistent state.
    let mut guard = WAITERS.lock().unwrap_or_else(PoisonError::into_inner);
    let mut i = 0;
    while i < guard.0.len() {
        if guard.0[i].tag == tag && !guard.0[i].done {
            let tmp = guard.0[i].value;
            guard.0[i].value = value;
            guard.0[i].done = true;
            WAKEUP.notify_all();
            return tmp;
        }
        i += 1;
    }
    let id = guard.1;
    guard.1 = guard.1.wrapping_add(1);
    guard.0.push(Waiter {
        tag,
        value,
        done: false,
        id,
    });
    loop {
        let mut found = None;
        let mut k = 0;
        while k < guard.0.len() {
            if guard.0[k].id == id && guard.0[k].done {
                found = Some(k);
            }
            k += 1;
        }
        if let Some(k) = found {
            // The C unlinks the entry when the partner arrives; here the
            // entry stays, marked done, until its owner collects the value.
            return guard.0.swap_remove(k).value;
        }
        guard = WAKEUP.wait(guard).unwrap_or_else(PoisonError::into_inner);
    }
}
