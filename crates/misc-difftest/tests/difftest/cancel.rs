//! `mp_cancel` of `misc/thread_tools.c`.
//!
//! The sequential test builds forests of up to eight `mp_cancel` objects in
//! the C and in the port and applies the same random operations to both:
//! trigger, reset, test, wait with zero, negative and short timeouts,
//! reparenting, callbacks set and removed, wakeup pipes made, and objects
//! freed. After every operation it compares, for every object, the flag,
//! how often its callback ran, and for a pipe how many bytes are waiting
//! and whether `poll` reports it readable.
//!
//! The threaded tests wake a waiting thread from another one and trigger a
//! tree from several threads at once. They sample schedules, they do not
//! explore them; `crates/misc/tests/loom.rs` explores the races of the port.

use std::ffi::c_void;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use rmpv_misc::thread_tools::{self as port, MpCancel};
use rmpv_misc_difftest::input::Rng;
use rmpv_misc_difftest::{c, fd_pending, fd_readable, with_timeout};

use crate::run;

const NODES: usize = 8;

extern "C" fn count(ctx: *mut c_void) {
    // SAFETY: ctx is the address of an AtomicUsize in the test's `Forest`,
    // which outlives every C object that holds it.
    let counter = unsafe { &*ctx.cast::<AtomicUsize>() };
    counter.fetch_add(1, Ordering::SeqCst);
}

struct Node {
    c: *mut c_void,
    r: Arc<MpCancel>,
    parent: Option<usize>,
    fds: Option<(i32, i32)>,
}

/// The objects of both sides, side by side.
struct Forest {
    nodes: Vec<Option<Node>>,
    c_counts: Box<[AtomicUsize; NODES]>,
    r_counts: Arc<[AtomicUsize; NODES]>,
}

impl Forest {
    fn new() -> Self {
        Self {
            nodes: (0..NODES).map(|_| None).collect(),
            c_counts: Box::new(std::array::from_fn(|_| AtomicUsize::new(0))),
            r_counts: Arc::new(std::array::from_fn(|_| AtomicUsize::new(0))),
        }
    }

    fn live(&self) -> Vec<usize> {
        (0..NODES).filter(|&i| self.nodes[i].is_some()).collect()
    }

    fn node(&self, i: usize) -> &Node {
        self.nodes[i].as_ref().unwrap()
    }

    fn has_children(&self, i: usize) -> bool {
        self.nodes.iter().flatten().any(|n| n.parent == Some(i))
    }

    /// Whether `j` is `i` or below it.
    fn is_below(&self, j: usize, i: usize) -> bool {
        let mut at = Some(j);
        while let Some(k) = at {
            if k == i {
                return true;
            }
            at = self.node(k).parent;
        }
        false
    }

    fn check(&self, what: &str) {
        for i in self.live() {
            let n = self.node(i);
            assert_eq!(
                c!(mp_cancel_test(n.c)),
                port::mp_cancel_test(Some(&n.r)),
                "{what}: test({i})"
            );
            assert_eq!(
                self.c_counts[i].load(Ordering::SeqCst),
                self.r_counts[i].load(Ordering::SeqCst),
                "{what}: callback count of {i}"
            );
            if let Some((cfd, rfd)) = n.fds {
                assert_eq!(
                    fd_pending(cfd),
                    fd_pending(rfd),
                    "{what}: pipe bytes of {i}"
                );
                assert_eq!(fd_readable(cfd), fd_readable(rfd), "{what}: readable {i}");
            }
        }
    }

    fn free(&mut self, i: usize) {
        let n = self.nodes[i].take().unwrap();
        c!(ta_free(n.c));
        drop(n.r);
    }

    fn free_all(&mut self) {
        while !self.live().is_empty() {
            for i in self.live() {
                if !self.has_children(i) {
                    self.free(i);
                }
            }
        }
    }
}

fn step(rng: &mut Rng, f: &mut Forest) -> String {
    let live = f.live();
    if live.is_empty() || (live.len() < NODES && rng.chance(6)) {
        let i = (0..NODES).find(|&i| f.nodes[i].is_none()).unwrap();
        f.c_counts[i].store(0, Ordering::SeqCst);
        f.r_counts[i].store(0, Ordering::SeqCst);
        f.nodes[i] = Some(Node {
            c: c!(mp_cancel_new(std::ptr::null_mut())),
            r: port::mp_cancel_new(),
            parent: None,
            fds: None,
        });
        return format!("new {i}");
    }
    let i = rng.pick(&live);
    match rng.below(12) {
        0 | 1 => {
            c!(mp_cancel_trigger(f.node(i).c));
            port::mp_cancel_trigger(&f.node(i).r);
            format!("trigger {i}")
        }
        2 => {
            c!(mp_cancel_reset(f.node(i).c));
            port::mp_cancel_reset(&f.node(i).r);
            format!("reset {i}")
        }
        3 => {
            let timeout = rng.pick(&[0.0, -1.0, -1e300, f64::NEG_INFINITY, 1e-4, f64::NAN]);
            let want = c!(mp_cancel_wait(f.node(i).c, timeout));
            let got = port::mp_cancel_wait(&f.node(i).r, timeout);
            assert_eq!(want, got, "wait {i} {timeout}");
            format!("wait {i} {timeout}")
        }
        4 | 5 => {
            // A parent that is not below i, so that no loop forms.
            let candidates: Vec<usize> = live
                .iter()
                .copied()
                .filter(|&j| !f.is_below(j, i))
                .collect();
            let parent = if rng.chance(4) || candidates.is_empty() {
                None
            } else {
                Some(rng.pick(&candidates))
            };
            let cp = parent.map_or(std::ptr::null_mut(), |p| f.node(p).c);
            c!(mp_cancel_set_parent(f.node(i).c, cp));
            port::mp_cancel_set_parent(&f.node(i).r, parent.map(|p| &f.node(p).r));
            f.nodes[i].as_mut().unwrap().parent = parent;
            format!("parent {i} {parent:?}")
        }
        6 | 7 => {
            if rng.chance(3) {
                c!(mp_cancel_set_cb(f.node(i).c, None, std::ptr::null_mut()));
                port::mp_cancel_set_cb(&f.node(i).r, None);
                format!("cb {i} none")
            } else {
                let ctx = (&raw const f.c_counts[i]).cast_mut().cast();
                c!(mp_cancel_set_cb(f.node(i).c, Some(count), ctx));
                let counts = Arc::clone(&f.r_counts);
                port::mp_cancel_set_cb(
                    &f.node(i).r,
                    Some(Box::new(move || {
                        counts[i].fetch_add(1, Ordering::SeqCst);
                    })),
                );
                format!("cb {i}")
            }
        }
        8 => {
            let cfd = c!(mp_cancel_get_fd(f.node(i).c));
            let rfd = port::mp_cancel_get_fd(&f.node(i).r);
            assert_eq!(cfd >= 0, rfd >= 0);
            f.nodes[i].as_mut().unwrap().fds = Some((cfd, rfd));
            format!("fd {i}")
        }
        9 if !f.has_children(i) => {
            f.free(i);
            format!("free {i}")
        }
        _ => {
            assert_eq!(
                c!(mp_cancel_test(f.node(i).c)),
                port::mp_cancel_test(Some(&f.node(i).r))
            );
            format!("test {i}")
        }
    }
}

#[test]
fn mp_cancel_sequences() {
    assert!(!port::mp_cancel_test(None));
    assert!(!c!(mp_cancel_test(std::ptr::null_mut())));
    run(
        "mp_cancel_new, trigger_locked, mp_cancel_trigger, mp_cancel_reset, mp_cancel_test, \
         mp_cancel_wait, retrigger_locked, mp_cancel_set_cb, mp_cancel_set_parent, \
         mp_cancel_get_fd, cancel_destroy (operations)",
        420,
        5000,
        |rng| {
            let mut f = Forest::new();
            let mut done = Vec::new();
            for _ in 0..20 {
                let what = step(rng, &mut f);
                done.push(what);
                f.check(&done.join(", "));
            }
            f.free_all();
        },
    );
}

/// A pipe fills up: mpv writes a byte per trigger and ignores `EAGAIN`, so
/// after more triggers than the pipe holds, both sides hold a full pipe,
/// and a reset drains it.
#[test]
fn mp_cancel_pipe_full() {
    let cc = c!(mp_cancel_new(std::ptr::null_mut()));
    let rc = port::mp_cancel_new();
    let (cfd, rfd) = (c!(mp_cancel_get_fd(cc)), port::mp_cancel_get_fd(&rc));
    for n in 0..70_000 {
        c!(mp_cancel_trigger(cc));
        port::mp_cancel_trigger(&rc);
        if n % 9999 == 0 {
            assert_eq!(fd_pending(cfd), fd_pending(rfd), "after {n}");
        }
    }
    assert_eq!(fd_pending(cfd), 65536);
    assert_eq!(fd_pending(rfd), 65536);
    c!(mp_cancel_reset(cc));
    port::mp_cancel_reset(&rc);
    assert_eq!((fd_pending(cfd), fd_pending(rfd)), (0, 0));
    assert!(!fd_readable(rfd));
    c!(ta_free(cc));
    println!("difftest mp_cancel pipe: 70000 triggers agree");
}

/// A pointer to a C object handed to another thread.
#[derive(Clone, Copy)]
struct Shared(*mut c_void);

// SAFETY: mp_cancel locks around everything but `triggered`, which is atomic.
unsafe impl Send for Shared {}

impl Shared {
    fn get(self) -> *mut c_void {
        self.0
    }
}

/// One thread waits with a long timeout, another triggers: the wait returns
/// true, on both sides. 200 rounds, sampled.
#[test]
fn mp_cancel_wait_threads() {
    with_timeout("mp_cancel_wait_threads", 120, || {
        run("mp_cancel_wait (two threads)", 421, 200, |rng| {
            let cc = Shared(c!(mp_cancel_new(std::ptr::null_mut())));
            let rc = port::mp_cancel_new();
            let delay = rng.below(3);
            let (cw, rw) = std::thread::scope(|s| {
                let cw = s.spawn(move || c!(mp_cancel_wait(cc.get(), 30.0)));
                let rc2 = Arc::clone(&rc);
                let rw = s.spawn(move || port::mp_cancel_wait(&rc2, 30.0));
                for _ in 0..delay {
                    std::thread::yield_now();
                }
                c!(mp_cancel_trigger(cc.get()));
                port::mp_cancel_trigger(&rc);
                (cw.join().unwrap(), rw.join().unwrap())
            });
            assert!(cw && rw);
            c!(ta_free(cc.get()));
        });
    });
}

/// Several threads trigger, reset and reparent the members of a small tree
/// at once; at the end each side is triggered once more from the root and
/// every member must be triggered, with every callback run at least once.
#[test]
fn mp_cancel_stress() {
    with_timeout("mp_cancel_stress", 300, || {
        run("mp_cancel (stress, 4 threads)", 422, 200, |rng| {
            let seed = rng.next_u64();
            let c_out = stress_c(seed);
            let r_out = stress_port(seed);
            assert_eq!(c_out, r_out);
        });
    });
}

/// Root 0 with children 1 and 2; 3 moves between 1 and 2.
fn stress_ops(seed: u64, worker: u64) -> Vec<u8> {
    let mut rng = Rng(seed ^ worker.wrapping_mul(0x9e37_79b9));
    (0..50).map(|_| rng.below(6) as u8).collect()
}

fn stress_c(seed: u64) -> (bool, bool, bool, bool, bool) {
    let nodes: Vec<Shared> = (0..4)
        .map(|_| Shared(c!(mp_cancel_new(std::ptr::null_mut()))))
        .collect();
    let counter = Box::new(AtomicUsize::new(0));
    c!(mp_cancel_set_parent(nodes[1].get(), nodes[0].get()));
    c!(mp_cancel_set_parent(nodes[2].get(), nodes[0].get()));
    c!(mp_cancel_set_parent(nodes[3].get(), nodes[1].get()));
    let ctx = Shared((&raw const *counter).cast_mut().cast());
    c!(mp_cancel_set_cb(nodes[3].get(), Some(count), ctx.get()));
    std::thread::scope(|s| {
        for w in 0..4 {
            let nodes = nodes.clone();
            s.spawn(move || {
                for op in stress_ops(seed, w) {
                    match op {
                        0 => c!(mp_cancel_trigger(nodes[(w as usize) % 3].get())),
                        1 => c!(mp_cancel_reset(nodes[(w as usize + 1) % 4].get())),
                        2 if w == 0 => c!(mp_cancel_set_parent(nodes[3].get(), nodes[2].get())),
                        3 if w == 0 => c!(mp_cancel_set_parent(nodes[3].get(), nodes[1].get())),
                        _ => {
                            let _ = c!(mp_cancel_test(nodes[3].get()));
                        }
                    }
                }
            });
        }
    });
    let before = counter.load(Ordering::SeqCst);
    for n in &nodes {
        c!(mp_cancel_reset(n.get()));
    }
    c!(mp_cancel_trigger(nodes[0].get()));
    let all: Vec<bool> = nodes.iter().map(|n| c!(mp_cancel_test(n.get()))).collect();
    let ran = counter.load(Ordering::SeqCst) > before;
    for i in [3, 2, 1, 0] {
        c!(ta_free(nodes[i].get()));
    }
    (all[0], all[1], all[2], all[3], ran)
}

fn stress_port(seed: u64) -> (bool, bool, bool, bool, bool) {
    let nodes: Vec<Arc<MpCancel>> = (0..4).map(|_| port::mp_cancel_new()).collect();
    let counter = Arc::new(AtomicUsize::new(0));
    port::mp_cancel_set_parent(&nodes[1], Some(&nodes[0]));
    port::mp_cancel_set_parent(&nodes[2], Some(&nodes[0]));
    port::mp_cancel_set_parent(&nodes[3], Some(&nodes[1]));
    let c2 = Arc::clone(&counter);
    port::mp_cancel_set_cb(
        &nodes[3],
        Some(Box::new(move || {
            c2.fetch_add(1, Ordering::SeqCst);
        })),
    );
    std::thread::scope(|s| {
        for w in 0..4 {
            let nodes = nodes.clone();
            s.spawn(move || {
                for op in stress_ops(seed, w) {
                    match op {
                        0 => port::mp_cancel_trigger(&nodes[(w as usize) % 3]),
                        1 => port::mp_cancel_reset(&nodes[(w as usize + 1) % 4]),
                        2 if w == 0 => port::mp_cancel_set_parent(&nodes[3], Some(&nodes[2])),
                        3 if w == 0 => port::mp_cancel_set_parent(&nodes[3], Some(&nodes[1])),
                        _ => {
                            let _ = port::mp_cancel_test(Some(&nodes[3]));
                        }
                    }
                }
            });
        }
    });
    let before = counter.load(Ordering::SeqCst);
    for n in &nodes {
        port::mp_cancel_reset(n);
    }
    port::mp_cancel_trigger(&nodes[0]);
    let all: Vec<bool> = nodes
        .iter()
        .map(|n| port::mp_cancel_test(Some(n)))
        .collect();
    let ran = counter.load(Ordering::SeqCst) > before;
    (all[0], all[1], all[2], all[3], ran)
}
