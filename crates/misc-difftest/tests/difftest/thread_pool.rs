//! `misc/thread_pool.c`.
//!
//! The work items wait on a gate the test opens, so that how many threads
//! the pool starts, which item runs on which thread, and what
//! `mp_thread_pool_run` and `mp_thread_pool_queue` return are fixed by the
//! C's counting rule rather than by the scheduler. The idle timeout of 10
//! seconds is tested once, on both sides at the same time.

use std::ffi::c_void;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

use rmpv_misc::thread_pool::{self as port, MpThreadPool};
use rmpv_misc_difftest::{c, with_timeout};

use crate::run;

/// What the work items of one round record.
#[derive(Default)]
struct Log {
    started: Mutex<Vec<(usize, ThreadId)>>,
    open: Mutex<bool>,
    gate: Condvar,
    finished: AtomicUsize,
}

impl Log {
    fn item(&self, index: usize) {
        self.started
            .lock()
            .unwrap()
            .push((index, std::thread::current().id()));
        let mut open = self.open.lock().unwrap();
        while !*open {
            open = self.gate.wait(open).unwrap();
        }
        drop(open);
        self.finished.fetch_add(1, Ordering::SeqCst);
    }

    fn open(&self) {
        *self.open.lock().unwrap() = true;
        self.gate.notify_all();
    }

    /// Waits until `n` items have started.
    fn wait_started(&self, n: usize) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while self.started.lock().unwrap().len() < n {
            assert!(Instant::now() < deadline, "{n} items never started");
            std::thread::sleep(Duration::from_micros(200));
        }
    }

    fn threads_of(&self, items: &[usize]) -> Vec<ThreadId> {
        let started = self.started.lock().unwrap();
        items
            .iter()
            .map(|i| started.iter().find(|(j, _)| j == i).unwrap().1)
            .collect()
    }
}

struct CItem {
    log: *const Log,
    index: usize,
}

extern "C" fn c_item(data: *mut c_void) {
    // SAFETY: data is a leaked Box<CItem>, freed here; its Log outlives the
    // pool, which is freed (and so joined) before the Log.
    let item = unsafe { Box::from_raw(data.cast::<CItem>()) };
    // SAFETY: see above.
    unsafe { &*item.log }.item(item.index);
}

/// One side of a pool under test.
enum Pool {
    C(*mut c_void, Arc<Log>),
    Port(MpThreadPool, Arc<Log>),
}

impl Pool {
    fn new(c_side: bool, init: i32, min: i32, max: i32) -> Option<Self> {
        let log = Arc::new(Log::default());
        if c_side {
            let p = c!(mp_thread_pool_create(std::ptr::null_mut(), init, min, max));
            (!p.is_null()).then_some(Self::C(p, log))
        } else {
            port::mp_thread_pool_create(init, min, max).map(|p| Self::Port(p, log))
        }
    }

    fn log(&self) -> &Arc<Log> {
        match self {
            Self::C(_, log) | Self::Port(_, log) => log,
        }
    }

    /// `mp_thread_pool_queue` (`run == false`) or `mp_thread_pool_run`.
    fn add(&self, index: usize, run: bool) -> bool {
        match self {
            Self::C(p, log) => {
                let data = Box::into_raw(Box::new(CItem {
                    log: Arc::as_ptr(log),
                    index,
                }));
                let ok = if run {
                    c!(mp_thread_pool_run(*p, c_item, data.cast()))
                } else {
                    c!(mp_thread_pool_queue(*p, c_item, data.cast()))
                };
                if !ok {
                    // SAFETY: the pool did not take the item.
                    drop(unsafe { Box::from_raw(data) });
                }
                ok
            }
            Self::Port(p, log) => {
                let log = Arc::clone(log);
                let work: port::Work = Box::new(move || log.item(index));
                let r = if run {
                    port::mp_thread_pool_run(p, work)
                } else {
                    port::mp_thread_pool_queue(p, work)
                };
                r.is_ok()
            }
        }
    }

    /// Frees the pool, which waits for its work.
    fn free(self) -> Arc<Log> {
        match self {
            Self::C(p, log) => {
                c!(ta_free(p));
                log
            }
            Self::Port(p, log) => {
                drop(p);
                log
            }
        }
    }
}

/// One thread runs everything, in the order it was queued.
#[test]
fn thread_pool_fifo() {
    with_timeout("thread_pool_fifo", 120, || {
        run(
            "mp_thread_pool_queue (one thread, order)",
            440,
            200,
            |rng| {
                let n = 1 + rng.below(40);
                let mut seen = Vec::new();
                for c_side in [true, false] {
                    let pool = Pool::new(c_side, 1, 1, 1).unwrap();
                    pool.log().open();
                    for i in 0..n {
                        assert!(pool.add(i, rng.chance(2)));
                    }
                    let log = pool.free();
                    let started = log.started.lock().unwrap().clone();
                    let order: Vec<usize> = started.iter().map(|&(i, _)| i).collect();
                    let first = started[0].1;
                    let one_thread = started.iter().all(|&(_, t)| t == first);
                    seen.push((order, one_thread, log.finished.load(Ordering::SeqCst)));
                }
                assert_eq!(seen[0], seen[1]);
                assert_eq!(seen[0], ((0..n).collect(), true, n));
            },
        );
    });
}

/// What a growth round observed: the results of the adds, how many items
/// were running before the gate opened, on how many threads, and how many
/// items finished in the end.
type Growth = (Vec<bool>, usize, usize, usize);

fn growth(c_side: bool, init: i32, min: i32, max: i32, adds: &[bool]) -> Option<Growth> {
    let pool = Pool::new(c_side, init, min, max)?;
    let results: Vec<bool> = adds
        .iter()
        .enumerate()
        .map(|(i, &run)| pool.add(i, run))
        .collect();
    let accepted = results.iter().filter(|&&ok| ok).count();
    // As many items run at once as the pool has threads.
    let threads = max.max(init) as usize;
    let running = accepted.min(threads);
    pool.log().wait_started(running);
    // Give items that should not start a chance to, wrongly.
    std::thread::sleep(Duration::from_millis(2));
    let started: Vec<usize> = pool
        .log()
        .started
        .lock()
        .unwrap()
        .iter()
        .map(|&(i, _)| i)
        .collect();
    let mut ids = pool.log().threads_of(&started);
    ids.sort_unstable_by_key(|t| format!("{t:?}"));
    ids.dedup();
    pool.log().open();
    let log = pool.free();
    let finished = log.finished.load(Ordering::SeqCst);
    Some((results, started.len(), ids.len(), finished))
}

/// Threads are started while the work outnumbers them, up to the maximum;
/// `mp_thread_pool_run` queues as `mp_thread_pool_queue` does once the
/// maximum is reached (see the report's suspected mpv defects).
#[test]
fn thread_pool_growth() {
    with_timeout("thread_pool_growth", 300, || {
        run(
            "mp_thread_pool_create, add_thread, thread_pool_add, mp_thread_pool_queue, \
             mp_thread_pool_run, worker_thread, thread_pool_dtor (growth)",
            441,
            300,
            |rng| {
                let max = 1 + rng.below(4) as i32;
                let min = rng.below(max as usize + 1) as i32;
                let init = rng.below(min as usize + 1) as i32;
                let adds: Vec<bool> = (0..rng.below(7)).map(|_| rng.chance(2)).collect();
                let want = growth(true, init, min, max, &adds);
                let got = growth(false, init, min, max, &adds);
                assert_eq!(want, got, "init {init} min {min} max {max} adds {adds:?}");
                let (results, started, threads, finished) = got.unwrap();
                assert!(results.iter().all(|&ok| ok));
                let running = adds.len().min(max as usize);
                assert_eq!((started, threads, finished), (running, running, adds.len()));
            },
        );
    });
}

/// After 10 seconds without work a thread above the minimum ends itself;
/// the next work then needs a new thread.
#[test]
fn thread_pool_idle_timeout() {
    with_timeout("thread_pool_idle_timeout", 120, || {
        let c_side = std::thread::spawn(|| idle_timeout(true));
        let port_side = idle_timeout(false);
        let c_result = c_side.join().unwrap();
        assert_eq!(c_result, port_side);
        // min 1, max 2: one of the two threads ends, so of the next two
        // items one runs on the old thread and one on a new one. min 0,
        // max 1: the only thread ends, so the next item has a new one.
        assert_eq!(port_side, (1, true));
        println!("difftest worker_thread idle timeout: 2 scenarios agree");
    });
}

/// (new threads among the two late items with min 1, max 2; whether the
/// late item with min 0, max 1 ran on a new thread)
fn idle_timeout(c_side: bool) -> (usize, bool) {
    let two = Pool::new(c_side, 0, 1, 2).unwrap();
    let one = Pool::new(c_side, 0, 0, 1).unwrap();
    // The first two items hold their threads until both run, so that
    // each runs on its own thread and the test knows both threads.
    assert!(two.add(0, false));
    assert!(two.add(1, false));
    two.log().wait_started(2);
    two.log().open();
    one.log().open();
    assert!(one.add(0, false));
    one.log().wait_started(1);
    let early_two = two.log().threads_of(&[0, 1]);
    let early_one = one.log().threads_of(&[0]);
    std::thread::sleep(Duration::from_millis(10_700));
    *two.log().open.lock().unwrap() = false;
    assert!(two.add(2, false));
    assert!(two.add(3, false));
    two.log().wait_started(4);
    assert!(one.add(1, false));
    one.log().wait_started(2);
    let late_two = two.log().threads_of(&[2, 3]);
    let late_one = one.log().threads_of(&[1]);
    two.log().open();
    two.free();
    one.free();
    let new_two = late_two.iter().filter(|t| !early_two.contains(t)).count();
    (new_two, !early_one.contains(&late_one[0]))
}
