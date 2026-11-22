//! `misc/dispatch.c`.
//!
//! - `dispatch_sequences`: one thread, which is also the target thread.
//!   Random sequences of enqueues of the three kinds, cancels, interrupts
//!   and `mp_dispatch_queue_process` calls, with callbacks that enqueue,
//!   cancel, interrupt and adjust the timeout in turn. The log of which
//!   callback ran, which autofree data was freed and when the wakeup
//!   callback ran must be the same on both sides after every step.
//! - `dispatch_adjust_timeout`: how long `mp_dispatch_queue_process` waits
//!   when a callback moves its deadline, on many deadlines.
//! - `dispatch_lock_handshake`: `mp_dispatch_lock` against a target thread
//!   that only enters `mp_dispatch_queue_process` when the wakeup callback
//!   asks it to; the onlock and wakeup callbacks must run as in the C.
//! - `dispatch_stress`: three threads enqueue, run synchronously and lock
//!   while a fourth processes; both sides must run every item once, in
//!   order per sender, on the target thread, never while a lock is held.
//!
//! The threaded tests sample schedules; `crates/misc/tests/loom.rs`
//! explores them for the port.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use rmpv_misc::dispatch::{self as port, DispatchKey, MpDispatchQueue};
use rmpv_misc::threads;
use rmpv_misc_difftest::input::Rng;
use rmpv_misc_difftest::{c, with_timeout};

use crate::{N, run};

/// What a callback logs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Ran(u8, usize),
    Freed(usize),
    Wakeup,
}

/// How an item is enqueued.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Plain,
    Notify,
    Autofree,
}

/// What a callback does after it logs, chosen per data id.
#[derive(Debug, Clone, Copy)]
enum Action {
    None,
    Enqueue(Kind, u8, usize),
    Cancel(u8, usize),
    Interrupt,
    Adjust(i64),
}

/// Callback data: the first two fields are those of `struct dt_autofree`.
#[repr(C)]
struct Slot {
    ctx: *const CSide,
    id: usize,
}

/// The C side of a sequence test.
struct CSide {
    queue: *mut c_void,
    log: Mutex<Vec<Event>>,
    script: Vec<Action>,
    slots: Vec<Slot>,
    /// Ids for autofree data, which the C tells apart by address.
    next_autofree: AtomicUsize,
}

const IDS: usize = 12;

extern "C" fn c_cb0(data: *mut c_void) {
    c_ran(0, data);
}
extern "C" fn c_cb1(data: *mut c_void) {
    c_ran(1, data);
}
extern "C" fn c_cb2(data: *mut c_void) {
    c_ran(2, data);
}

fn c_fn(f: u8) -> extern "C" fn(*mut c_void) {
    [c_cb0, c_cb1, c_cb2][f as usize]
}

extern "C" fn c_wakeup(ctx: *mut c_void) {
    // SAFETY: ctx is the CSide of the running test, which outlives its queue.
    let side = unsafe { &*ctx.cast::<CSide>() };
    side.log.lock().unwrap().push(Event::Wakeup);
}

extern "C" fn c_freed(ctx: *mut c_void, id: usize) {
    // SAFETY: as in c_wakeup.
    let side = unsafe { &*ctx.cast::<CSide>() };
    side.log.lock().unwrap().push(Event::Freed(id));
}

fn c_ran(f: u8, data: *mut c_void) {
    // SAFETY: data is a Slot of the CSide, or a dt_autofree whose first two
    // fields have the same layout.
    let slot = unsafe { &*data.cast::<Slot>() };
    // SAFETY: the CSide outlives its queue and every item in it.
    let side = unsafe { &*slot.ctx };
    side.log.lock().unwrap().push(Event::Ran(f, slot.id));
    let action = side.script.get(slot.id).copied().unwrap_or(Action::None);
    side.act(action);
}

impl CSide {
    fn data(&self, id: usize) -> *mut c_void {
        (&raw const self.slots[id]).cast_mut().cast()
    }

    fn enqueue(&self, kind: Kind, f: u8, id: usize) {
        match kind {
            Kind::Plain => c!(mp_dispatch_enqueue(self.queue, c_fn(f), self.data(id))),
            Kind::Notify => c!(mp_dispatch_enqueue_notify(
                self.queue,
                c_fn(f),
                self.data(id)
            )),
            Kind::Autofree => {
                let n = self.next_autofree.fetch_add(1, Ordering::SeqCst);
                let data = c!(dt_autofree_new(
                    std::ptr::from_ref(self).cast_mut().cast(),
                    IDS + n,
                    c_freed
                ));
                c!(mp_dispatch_enqueue_autofree(self.queue, c_fn(f), data));
            }
        }
    }

    fn act(&self, action: Action) {
        match action {
            Action::None => {}
            Action::Enqueue(kind, f, id) => self.enqueue(kind, f, id),
            Action::Cancel(f, id) => c!(mp_dispatch_cancel_fn(self.queue, c_fn(f), self.data(id))),
            Action::Interrupt => c!(mp_dispatch_interrupt(self.queue)),
            Action::Adjust(until) => c!(mp_dispatch_adjust_timeout(self.queue, until)),
        }
    }
}

/// The port's side of a sequence test.
struct RSide {
    queue: Mutex<Option<Arc<MpDispatchQueue>>>,
    log: Mutex<Vec<Event>>,
    script: Vec<Action>,
    next_autofree: AtomicUsize,
}

/// Autofree data of the port's side: it logs its own drop.
struct FreeLog {
    side: Weak<RSide>,
    id: usize,
}

impl Drop for FreeLog {
    fn drop(&mut self) {
        if let Some(side) = self.side.upgrade() {
            side.log.lock().unwrap().push(Event::Freed(self.id));
        }
    }
}

fn key(f: u8, id: usize) -> DispatchKey {
    DispatchKey {
        func: f as usize,
        data: id,
    }
}

impl RSide {
    fn queue(&self) -> Arc<MpDispatchQueue> {
        Arc::clone(self.queue.lock().unwrap().as_ref().unwrap())
    }

    fn ran(self: &Arc<Self>, f: u8, id: usize) {
        self.log.lock().unwrap().push(Event::Ran(f, id));
        let action = self.script.get(id).copied().unwrap_or(Action::None);
        self.act(action);
    }

    fn enqueue(self: &Arc<Self>, kind: Kind, f: u8, id: usize) {
        let q = self.queue();
        let me = Arc::clone(self);
        match kind {
            Kind::Plain => port::mp_dispatch_enqueue(&q, key(f, id), move || me.ran(f, id)),
            Kind::Notify => port::mp_dispatch_enqueue_notify(&q, key(f, id), move || me.ran(f, id)),
            Kind::Autofree => {
                let n = IDS + self.next_autofree.fetch_add(1, Ordering::SeqCst);
                let data = FreeLog {
                    side: Arc::downgrade(self),
                    id: n,
                };
                port::mp_dispatch_enqueue_autofree(&q, key(f, n), data, move |d: &mut FreeLog| {
                    me.ran(f, d.id);
                });
            }
        }
    }

    fn act(self: &Arc<Self>, action: Action) {
        let q = self.queue();
        match action {
            Action::None => {}
            Action::Enqueue(kind, f, id) => self.enqueue(kind, f, id),
            Action::Cancel(f, id) => port::mp_dispatch_cancel_fn(&q, key(f, id)),
            Action::Interrupt => port::mp_dispatch_interrupt(&q),
            Action::Adjust(until) => port::mp_dispatch_adjust_timeout(&q, until),
        }
    }
}

fn script(rng: &mut Rng) -> Vec<Action> {
    (0..IDS + 1000)
        .map(|id| {
            if id >= IDS {
                // Autofree data runs no action: its ids are not chosen.
                return Action::None;
            }
            match rng.below(8) {
                // Only higher ids, so that a chain of enqueues ends.
                0 | 1 if id + 1 < IDS => Action::Enqueue(
                    rng.pick(&[Kind::Plain, Kind::Notify, Kind::Autofree]),
                    rng.below(3) as u8,
                    id + 1 + rng.below(IDS - id - 1),
                ),
                2 => Action::Cancel(rng.below(3) as u8, rng.below(IDS)),
                3 => Action::Interrupt,
                4 => Action::Adjust(rng.pick(&[i64::MIN, -5, 0, 1, i64::MAX])),
                _ => Action::None,
            }
        })
        .collect()
}

#[test]
fn dispatch_sequences() {
    c!(mp_time_init());
    run(
        "mp_dispatch_create, mp_dispatch_set_wakeup_fn, mp_dispatch_append, \
         mp_dispatch_enqueue, mp_dispatch_enqueue_autofree, mp_dispatch_enqueue_notify, \
         mp_dispatch_cancel_fn, mp_dispatch_queue_process, mp_dispatch_interrupt, \
         mp_dispatch_adjust_timeout, queue_dtor (sequences)",
        430,
        N / 10,
        |rng| {
            let script = script(rng);
            let with_wakeup = rng.chance(2);
            let mut cside = Box::new(CSide {
                queue: c!(mp_dispatch_create(std::ptr::null_mut())),
                log: Mutex::new(Vec::new()),
                script: script.clone(),
                slots: Vec::new(),
                next_autofree: AtomicUsize::new(0),
            });
            let ctx: *const CSide = &raw const *cside;
            cside.slots = (0..IDS).map(|id| Slot { ctx, id }).collect();
            let rside = Arc::new(RSide {
                queue: Mutex::new(None),
                log: Mutex::new(Vec::new()),
                script,
                next_autofree: AtomicUsize::new(0),
            });
            let mut q = port::mp_dispatch_create();
            if with_wakeup {
                c!(mp_dispatch_set_wakeup_fn(
                    cside.queue,
                    Some(c_wakeup),
                    ctx.cast_mut().cast()
                ));
                let weak = Arc::downgrade(&rside);
                port::mp_dispatch_set_wakeup_fn(
                    &mut q,
                    Some(Box::new(move || {
                        if let Some(side) = weak.upgrade() {
                            side.log.lock().unwrap().push(Event::Wakeup);
                        }
                    })),
                );
            }
            *rside.queue.lock().unwrap() = Some(Arc::new(q));
            let mut steps = Vec::new();
            for _ in 0..10 {
                let step = match rng.below(8) {
                    0..=2 => {
                        let kind = rng.pick(&[Kind::Plain, Kind::Notify, Kind::Autofree]);
                        let (f, id) = (rng.below(3) as u8, rng.below(IDS));
                        cside.enqueue(kind, f, id);
                        rside.enqueue(kind, f, id);
                        format!("enqueue {kind:?} {f} {id}")
                    }
                    3 => {
                        let (f, id) = (rng.below(3) as u8, rng.below(IDS));
                        cside.act(Action::Cancel(f, id));
                        rside.act(Action::Cancel(f, id));
                        format!("cancel {f} {id}")
                    }
                    4 => {
                        cside.act(Action::Interrupt);
                        rside.act(Action::Interrupt);
                        "interrupt".to_owned()
                    }
                    _ => {
                        let timeout = rng.pick(&[0.0, -1.0, 0.0, f64::NAN, -0.0]);
                        c!(mp_dispatch_queue_process(cside.queue, timeout));
                        port::mp_dispatch_queue_process(&rside.queue(), timeout);
                        format!("process {timeout}")
                    }
                };
                steps.push(step);
                assert_eq!(
                    *cside.log.lock().unwrap(),
                    *rside.log.lock().unwrap(),
                    "{steps:?}"
                );
            }
            // Empty both queues, then free them.
            c!(mp_dispatch_queue_process(cside.queue, 0.0));
            port::mp_dispatch_queue_process(&rside.queue(), 0.0);
            assert_eq!(
                *cside.log.lock().unwrap(),
                *rside.log.lock().unwrap(),
                "{steps:?}"
            );
            c!(ta_free(cside.queue));
            drop(rside.queue.lock().unwrap().take());
        },
    );
}

/// The port refuses to free a queue that still holds items, as the C
/// asserts. (The C aborts the process, so only the port is run.)
#[test]
fn queue_dtor_asserts() {
    let q = port::mp_dispatch_create();
    port::mp_dispatch_enqueue(&q, key(0, 0), || {});
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || drop(q)));
    assert!(r.is_err());
    println!("difftest queue_dtor: a non-empty queue is refused");
}

/// How long `mp_dispatch_queue_process(q, timeout)` takes when a callback
/// calls `mp_dispatch_adjust_timeout(q, now + offset)`.
#[test]
fn dispatch_adjust_timeout() {
    c!(mp_time_init());
    with_timeout("dispatch_adjust_timeout", 300, || {
        let mut rng = Rng(431);
        let cases: Vec<(f64, i64)> = (0..300)
            .map(|_| {
                let timeout = rng.pick(&[0.0, 0.02, 0.04, -1.0]);
                let offset_ms = rng.pick(&[-1000, -1, 0, 10, 30, 60, i64::MAX / 1_000_000]);
                (timeout, offset_ms)
            })
            .collect();
        let c_cases = cases.clone();
        let c_thread = std::thread::spawn(move || {
            c_cases
                .iter()
                .map(|&(t, off)| c_adjust_case(t, off))
                .collect::<Vec<_>>()
        });
        let r_times: Vec<Duration> = cases
            .iter()
            .map(|&(t, off)| port_adjust_case(t, off))
            .collect();
        let c_times = c_thread.join().unwrap();
        for (i, &(timeout, off)) in cases.iter().enumerate() {
            // The wait ends at the earlier of the deadline and `until`, and
            // not before the callback ran.
            let want_ms = if timeout <= 0.0 {
                0.0
            } else {
                (timeout * 1000.0).min(off.max(0) as f64)
            };
            for (side, took) in [("C", c_times[i]), ("port", r_times[i])] {
                let ms = took.as_secs_f64() * 1000.0;
                assert!(
                    ms >= want_ms - 1.0 && ms < want_ms + 15.0,
                    "{side}: timeout {timeout}, offset {off} ms: took {ms} ms, want {want_ms}"
                );
            }
        }
        println!(
            "difftest mp_dispatch_adjust_timeout: {} inputs agree",
            cases.len()
        );
    });
}

struct AdjustCtx {
    queue: *mut c_void,
    offset_ms: i64,
}

extern "C" fn c_adjust(data: *mut c_void) {
    // SAFETY: data is the AdjustCtx of c_adjust_case, alive during the call.
    let ctx = unsafe { &*data.cast::<AdjustCtx>() };
    let now = c!(mp_time_ns());
    let until = now.saturating_add(ctx.offset_ms.saturating_mul(1_000_000));
    c!(mp_dispatch_adjust_timeout(ctx.queue, until));
}

extern "C" fn nothing(_: *mut c_void) {}

fn c_adjust_case(timeout: f64, offset_ms: i64) -> Duration {
    let q = c!(mp_dispatch_create(std::ptr::null_mut()));
    // A wakeup callback, so that the enqueue does not set `interrupted`.
    c!(mp_dispatch_set_wakeup_fn(
        q,
        Some(nothing),
        std::ptr::null_mut()
    ));
    let mut ctx = AdjustCtx {
        queue: q,
        offset_ms,
    };
    c!(mp_dispatch_enqueue(q, c_adjust, (&raw mut ctx).cast()));
    let start = Instant::now();
    c!(mp_dispatch_queue_process(q, timeout));
    let took = start.elapsed();
    c!(ta_free(q));
    took
}

fn port_adjust_case(timeout: f64, offset_ms: i64) -> Duration {
    let mut q = port::mp_dispatch_create();
    port::mp_dispatch_set_wakeup_fn(&mut q, Some(Box::new(|| {})));
    let q = Arc::new(q);
    let q2 = Arc::clone(&q);
    port::mp_dispatch_enqueue(&q, key(0, 0), move || {
        let until = threads::mp_time_ns().saturating_add(offset_ms.saturating_mul(1_000_000));
        port::mp_dispatch_adjust_timeout(&q2, until);
    });
    let start = Instant::now();
    port::mp_dispatch_queue_process(&q, timeout);
    start.elapsed()
}

/// A pointer handed to another thread.
#[derive(Clone, Copy)]
struct Shared(*mut c_void);

// SAFETY: the pointers name C queues, whose functions lock around every
// access, or test state that is only touched through atomics and mutexes.
unsafe impl Send for Shared {}
// SAFETY: as above.
unsafe impl Sync for Shared {}

impl Shared {
    fn get(self) -> *mut c_void {
        self.0
    }
}

/// The target thread of the handshake test: it enters
/// `mp_dispatch_queue_process` only when asked.
#[derive(Default)]
struct Target {
    asked: Mutex<bool>,
    ask: std::sync::Condvar,
    stop: AtomicBool,
    processed: AtomicUsize,
    onlock: AtomicUsize,
    wakeups: AtomicUsize,
}

impl Target {
    fn wake(&self) {
        self.wakeups.fetch_add(1, Ordering::SeqCst);
        *self.asked.lock().unwrap() = true;
        self.ask.notify_all();
    }

    /// Waits to be asked, then calls `process`; until `stop`.
    fn serve(&self, process: impl Fn()) {
        loop {
            let mut asked = self.asked.lock().unwrap();
            while !*asked && !self.stop.load(Ordering::SeqCst) {
                asked = self.ask.wait(asked).unwrap();
            }
            if self.stop.load(Ordering::SeqCst) {
                return;
            }
            *asked = false;
            drop(asked);
            process();
            self.processed.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn finish(&self) {
        self.stop.store(true, Ordering::SeqCst);
        self.ask.notify_all();
    }
}

extern "C" fn c_target_wake(ctx: *mut c_void) {
    // SAFETY: ctx is the Target of the running round, alive across it.
    unsafe { &*ctx.cast::<Target>() }.wake();
}

extern "C" fn c_target_onlock(ctx: *mut c_void) {
    // SAFETY: as in c_target_wake.
    unsafe { &*ctx.cast::<Target>() }
        .onlock
        .fetch_add(1, Ordering::SeqCst);
}

/// (onlock calls, whether the wakeup callback ran, process calls that
/// returned while the lock was held). The target may still be in a
/// process call from an earlier wakeup when the next lock starts, so the
/// number of wakeups is not fixed.
type Handshake = (usize, bool, usize);

fn handshake_c(locks: usize) -> Handshake {
    let target = Arc::new(Target::default());
    let q = Shared(c!(mp_dispatch_create(std::ptr::null_mut())));
    let ctx = Shared(Arc::as_ptr(&target).cast_mut().cast());
    c!(mp_dispatch_set_wakeup_fn(
        q.get(),
        Some(c_target_wake),
        ctx.get()
    ));
    c!(mp_dispatch_set_onlock_fn(
        q.get(),
        Some(c_target_onlock),
        ctx.get()
    ));
    let mut during = 0;
    std::thread::scope(|s| {
        let t = Arc::clone(&target);
        s.spawn(move || t.serve(|| c!(mp_dispatch_queue_process(q.get(), 0.0))));
        for _ in 0..locks {
            c!(mp_dispatch_lock(q.get()));
            // The target is held inside mp_dispatch_queue_process.
            let before = target.processed.load(Ordering::SeqCst);
            std::thread::yield_now();
            during += target.processed.load(Ordering::SeqCst) - before;
            c!(mp_dispatch_unlock(q.get()));
        }
        target.finish();
    });
    c!(ta_free(q.get()));
    (
        target.onlock.load(Ordering::SeqCst),
        target.wakeups.load(Ordering::SeqCst) >= 1,
        during,
    )
}

fn handshake_port(locks: usize) -> Handshake {
    let target = Arc::new(Target::default());
    let mut q = port::mp_dispatch_create();
    let t = Arc::clone(&target);
    port::mp_dispatch_set_wakeup_fn(&mut q, Some(Box::new(move || t.wake())));
    let t = Arc::clone(&target);
    port::mp_dispatch_set_onlock_fn(
        &mut q,
        Some(Box::new(move || {
            t.onlock.fetch_add(1, Ordering::SeqCst);
        })),
    );
    let q = Arc::new(q);
    let mut during = 0;
    std::thread::scope(|s| {
        let t = Arc::clone(&target);
        let q2 = Arc::clone(&q);
        s.spawn(move || t.serve(|| port::mp_dispatch_queue_process(&q2, 0.0)));
        for _ in 0..locks {
            port::mp_dispatch_lock(&q);
            let before = target.processed.load(Ordering::SeqCst);
            std::thread::yield_now();
            during += target.processed.load(Ordering::SeqCst) - before;
            port::mp_dispatch_unlock(&q);
        }
        target.finish();
    });
    (
        target.onlock.load(Ordering::SeqCst),
        target.wakeups.load(Ordering::SeqCst) >= 1,
        during,
    )
}

#[test]
fn dispatch_lock_handshake() {
    with_timeout("dispatch_lock_handshake", 120, || {
        run(
            "mp_dispatch_set_onlock_fn, mp_dispatch_lock, mp_dispatch_unlock (handshake)",
            432,
            200,
            |rng| {
                let locks = 1 + rng.below(5);
                let want = handshake_c(locks);
                let got = handshake_port(locks);
                assert_eq!(want, got);
                assert_eq!(want, (locks, true, 0));
            },
        );
    });
}

/// What a stress round observed.
#[derive(Debug, PartialEq, Eq)]
struct Outcome {
    /// Every plain item ran exactly once.
    all_ran_once: bool,
    /// Plain items of each sender ran in the order they were queued.
    in_order: bool,
    /// Callbacks that ran on another thread than a processing one, or
    /// while a lock was held.
    violations: usize,
    /// `mp_dispatch_run` calls that returned after their callback.
    runs_done: usize,
    /// Every sender's notify item ran at least once.
    notified: bool,
}

const SENDERS: usize = 3;
const OPS: usize = 60;

/// State shared by the stress threads of one side.
struct Stress {
    ran: Mutex<Vec<(usize, usize)>>,
    in_lock: AtomicBool,
    /// Threads that call `mp_dispatch_queue_process`.
    processors: Mutex<Vec<std::thread::ThreadId>>,
    violations: AtomicUsize,
    runs_done: AtomicUsize,
    notified: [AtomicUsize; SENDERS],
}

impl Stress {
    fn new() -> Self {
        Self {
            ran: Mutex::new(Vec::new()),
            in_lock: AtomicBool::new(false),
            processors: Mutex::new(Vec::new()),
            violations: AtomicUsize::new(0),
            runs_done: AtomicUsize::new(0),
            notified: std::array::from_fn(|_| AtomicUsize::new(0)),
        }
    }

    fn callback(&self) {
        let me = std::thread::current().id();
        if self.in_lock.load(Ordering::SeqCst) || !self.processors.lock().unwrap().contains(&me) {
            self.violations.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn outcome(&self, queued: &[Vec<usize>]) -> Outcome {
        let ran = self.ran.lock().unwrap().clone();
        let mut all_ran_once = true;
        let mut in_order = true;
        for (s, items) in queued.iter().enumerate() {
            let mine: Vec<usize> = ran
                .iter()
                .filter(|(rs, _)| *rs == s)
                .map(|&(_, i)| i)
                .collect();
            let mut sorted = mine.clone();
            sorted.sort_unstable();
            all_ran_once &= sorted == *items;
            in_order &= mine == *items;
        }
        Outcome {
            all_ran_once,
            in_order,
            violations: self.violations.load(Ordering::SeqCst),
            runs_done: self.runs_done.load(Ordering::SeqCst),
            notified: self.notified.iter().all(|n| n.load(Ordering::SeqCst) > 0),
        }
    }
}

/// Data of one C callback.
struct StressItem {
    stress: *const Stress,
    sender: usize,
    index: usize,
}

extern "C" fn c_stress_plain(data: *mut c_void) {
    // SAFETY: data is a leaked Box<StressItem>, freed here; its Stress
    // outlives the queue.
    let item = unsafe { Box::from_raw(data.cast::<StressItem>()) };
    // SAFETY: see above.
    let stress = unsafe { &*item.stress };
    stress.callback();
    stress.ran.lock().unwrap().push((item.sender, item.index));
}

extern "C" fn c_stress_run(data: *mut c_void) {
    // SAFETY: data is a StressItem on the stack of the mp_dispatch_run
    // caller, which waits for this call.
    let item = unsafe { &*data.cast::<StressItem>() };
    // SAFETY: as in c_stress_plain.
    unsafe { &*item.stress }.callback();
}

extern "C" fn c_stress_notify(data: *mut c_void) {
    // SAFETY: data is an element of a [StressItem; SENDERS] that outlives
    // the queue.
    let item = unsafe { &*data.cast::<StressItem>() };
    // SAFETY: as in c_stress_plain.
    let stress = unsafe { &*item.stress };
    stress.callback();
    stress.notified[item.sender].fetch_add(1, Ordering::SeqCst);
}

fn sender_ops(seed: u64, sender: usize) -> Vec<u8> {
    let mut rng = Rng(seed ^ (sender as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15));
    (0..OPS).map(|_| rng.below(4) as u8).collect()
}

fn queued_plain(seed: u64) -> Vec<Vec<usize>> {
    (0..SENDERS)
        .map(|s| {
            sender_ops(seed, s)
                .iter()
                .enumerate()
                .filter(|&(_, &op)| op == 0)
                .map(|(i, _)| i)
                .collect()
        })
        .collect()
}

fn stress_c(seed: u64) -> Outcome {
    let stress = Arc::new(Stress::new());
    let sp: *const Stress = Arc::as_ptr(&stress);
    let notify_items: Vec<StressItem> = (0..SENDERS)
        .map(|sender| StressItem {
            stress: sp,
            sender,
            index: 0,
        })
        .collect();
    let q = Shared(c!(mp_dispatch_create(std::ptr::null_mut())));
    let stop = AtomicBool::new(false);
    let sps = Shared(sp.cast_mut().cast());
    std::thread::scope(|s| {
        let st = Arc::clone(&stress);
        let stop = &stop;
        s.spawn(move || {
            st.processors
                .lock()
                .unwrap()
                .push(std::thread::current().id());
            while !stop.load(Ordering::SeqCst) {
                c!(mp_dispatch_queue_process(q.get(), 1.0));
            }
        });
        let senders: Vec<_> = (0..SENDERS)
            .map(|sender| {
                let st = Arc::clone(&stress);
                let notify = Shared((&raw const notify_items[sender]).cast_mut().cast());
                s.spawn(move || {
                    for (index, op) in sender_ops(seed, sender).into_iter().enumerate() {
                        let item = StressItem {
                            stress: sps.get().cast_const().cast(),
                            sender,
                            index,
                        };
                        match op {
                            0 => {
                                let data = Box::into_raw(Box::new(item));
                                c!(mp_dispatch_enqueue(q.get(), c_stress_plain, data.cast()));
                            }
                            1 => {
                                let mut item = item;
                                c!(mp_dispatch_run(
                                    q.get(),
                                    c_stress_run,
                                    (&raw mut item).cast()
                                ));
                                st.runs_done.fetch_add(1, Ordering::SeqCst);
                            }
                            2 => {
                                c!(mp_dispatch_lock(q.get()));
                                st.in_lock.store(true, Ordering::SeqCst);
                                std::thread::yield_now();
                                st.in_lock.store(false, Ordering::SeqCst);
                                c!(mp_dispatch_unlock(q.get()));
                            }
                            _ => c!(mp_dispatch_enqueue_notify(
                                q.get(),
                                c_stress_notify,
                                notify.get()
                            )),
                        }
                    }
                })
            })
            .collect();
        for t in senders {
            t.join().unwrap();
        }
        stop.store(true, Ordering::SeqCst);
        c!(mp_dispatch_interrupt(q.get()));
    });
    // Whatever is left runs here.
    stress
        .processors
        .lock()
        .unwrap()
        .push(std::thread::current().id());
    c!(mp_dispatch_queue_process(q.get(), 0.0));
    c!(ta_free(q.get()));
    stress.outcome(&queued_plain(seed))
}

fn stress_port(seed: u64) -> Outcome {
    let stress = Arc::new(Stress::new());
    let q = Arc::new(port::mp_dispatch_create());
    let stop = AtomicBool::new(false);
    std::thread::scope(|s| {
        let st = Arc::clone(&stress);
        let stop = &stop;
        let q2 = Arc::clone(&q);
        s.spawn(move || {
            st.processors
                .lock()
                .unwrap()
                .push(std::thread::current().id());
            while !stop.load(Ordering::SeqCst) {
                port::mp_dispatch_queue_process(&q2, 1.0);
            }
        });
        let senders: Vec<_> = (0..SENDERS)
            .map(|sender| {
                let st = Arc::clone(&stress);
                let q = Arc::clone(&q);
                s.spawn(move || {
                    for (index, op) in sender_ops(seed, sender).into_iter().enumerate() {
                        let st2 = Arc::clone(&st);
                        match op {
                            0 => port::mp_dispatch_enqueue(
                                &q,
                                key(0, sender * OPS + index),
                                move || {
                                    st2.callback();
                                    st2.ran.lock().unwrap().push((sender, index));
                                },
                            ),
                            1 => {
                                port::mp_dispatch_run(&q, key(1, 0), move || st2.callback());
                                st.runs_done.fetch_add(1, Ordering::SeqCst);
                            }
                            2 => {
                                port::mp_dispatch_lock(&q);
                                st.in_lock.store(true, Ordering::SeqCst);
                                std::thread::yield_now();
                                st.in_lock.store(false, Ordering::SeqCst);
                                port::mp_dispatch_unlock(&q);
                            }
                            _ => port::mp_dispatch_enqueue_notify(&q, key(2, sender), move || {
                                st2.callback();
                                st2.notified[sender].fetch_add(1, Ordering::SeqCst);
                            }),
                        }
                    }
                })
            })
            .collect();
        for t in senders {
            t.join().unwrap();
        }
        stop.store(true, Ordering::SeqCst);
        port::mp_dispatch_interrupt(&q);
    });
    stress
        .processors
        .lock()
        .unwrap()
        .push(std::thread::current().id());
    port::mp_dispatch_queue_process(&q, 0.0);
    stress.outcome(&queued_plain(seed))
}

#[test]
fn dispatch_stress() {
    c!(mp_time_init());
    with_timeout("dispatch_stress", 300, || {
        run(
            "mp_dispatch_run, mp_dispatch_lock, mp_dispatch_unlock (stress, 4 threads)",
            433,
            300,
            |rng| {
                let seed = rng.next_u64();
                let want = stress_c(seed);
                let got = stress_port(seed);
                assert_eq!(want, got, "seed {seed}");
                let runs: usize = (0..SENDERS)
                    .map(|s| {
                        sender_ops(seed, s)
                            .into_iter()
                            .filter(|&op| op == 1)
                            .count()
                    })
                    .sum();
                let notifies = (0..SENDERS).all(|s| sender_ops(seed, s).contains(&3));
                assert_eq!(
                    got,
                    Outcome {
                        all_ran_once: true,
                        in_order: true,
                        violations: 0,
                        runs_done: runs,
                        notified: notifies,
                    }
                );
            },
        );
    });
}
