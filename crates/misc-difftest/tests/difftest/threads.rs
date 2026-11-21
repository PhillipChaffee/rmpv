//! `mp_waiter` of `thread_tools.c`, and `rendezvous.c`.
//!
//! These block until another thread acts, so a test is two threads and a
//! check that each side hands over the same values. That says less than
//! the tests of sequential code do: the order in which the threads run is
//! whatever the scheduler picks, and nothing here explores the orders.
//! Each test runs 2,000 rounds, as each round starts a thread per side.

use std::ffi::c_void;

use rmpv_misc::{rendezvous, thread_tools};
use rmpv_misc_difftest::c;
use rmpv_misc_difftest::input::Rng;

use crate::run;

const ROUNDS: usize = 2000;

/// A C pointer that a test hands to a second thread.
#[derive(Clone, Copy)]
struct Shared(*mut c_void);

// SAFETY: the pointer names an mp_waiter or a rendezvous tag. mpv's functions
// lock around every access to the first and never dereference the second.
unsafe impl Send for Shared {}

impl Shared {
    fn get(self) -> *mut c_void {
        self.0
    }
}

fn dawdle(rng: &mut Rng) {
    for _ in 0..rng.below(3) {
        std::thread::yield_now();
    }
}

#[test]
fn mp_waiter() {
    run(
        "mp_waiter_wait, mp_waiter_wakeup, mp_waiter_poll",
        400,
        ROUNDS,
        |rng| {
            let value = rng.next_u64() as usize;
            let wake_first = rng.chance(2);

            let c_waiter = Shared(c!(dt_waiter_new()));
            let r_waiter = thread_tools::MpWaiter::new();
            assert_eq!(
                c!(mp_waiter_poll(c_waiter.get())),
                thread_tools::mp_waiter_poll(&r_waiter)
            );
            if wake_first {
                c!(mp_waiter_wakeup(c_waiter.get(), value));
                thread_tools::mp_waiter_wakeup(&r_waiter, value);
                assert!(c!(mp_waiter_poll(c_waiter.get())));
                assert!(thread_tools::mp_waiter_poll(&r_waiter));
            }
            let (want, got) = std::thread::scope(|scope| {
                let c_thread = scope.spawn(move || c!(mp_waiter_wait(c_waiter.get())));
                let r_thread = scope.spawn(|| thread_tools::mp_waiter_wait(&r_waiter));
                if !wake_first {
                    dawdle(rng);
                    c!(mp_waiter_wakeup(c_waiter.get(), value));
                    thread_tools::mp_waiter_wakeup(&r_waiter, value);
                }
                (c_thread.join().unwrap(), r_thread.join().unwrap())
            });
            assert_eq!((want, got), (value, value));
            // mp_waiter_wait has destroyed the C waiter; only its memory is left.
            c!(free(c_waiter.get()));
        },
    );
}

#[test]
fn mp_rendezvous() {
    run("mp_rendezvous", 401, ROUNDS, |rng| {
        let (a, b) = (rng.next_u64() as isize, rng.next_u64() as isize);
        // The tag is only a key. Each side gets its own, so the C and the
        // port cannot meet each other.
        let mut c_slot = 0_u8;
        let c_tag = Shared((&raw mut c_slot).cast());
        let r_tag = &raw const c_slot as usize + 1;
        let first_delay = rng.below(3);
        let (c_first, c_second, r_first, r_second) = std::thread::scope(|scope| {
            let c1 = scope.spawn(move || c!(mp_rendezvous(c_tag.get(), a)));
            let r1 = scope.spawn(move || rendezvous::mp_rendezvous(r_tag, a));
            for _ in 0..first_delay {
                std::thread::yield_now();
            }
            let c2 = c!(mp_rendezvous(c_tag.get(), b));
            let r2 = rendezvous::mp_rendezvous(r_tag, b);
            (c1.join().unwrap(), c2, r1.join().unwrap(), r2)
        });
        // Each caller gets the other's value.
        assert_eq!((c_first, c_second), (b, a));
        assert_eq!((r_first, r_second), (b, a));
    });
}
