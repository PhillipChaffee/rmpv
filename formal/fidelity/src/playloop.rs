//! Generates random operation scripts for the playloop target.
//!
//! A float crosses the script as the hex digits of its bit pattern, so mpv and
//! the model start from the identical value.

use crate::rng::Rng;
use std::fmt::Write as _;

/// `MP_NOPTS_VALUE`. The `nopts` operation checks it against mpv's own.
const NOPTS: f64 = -9_223_372_036_854_775_808.0;

/// Times at the edges of every comparison the functions make: the sentinel and
/// its neighbours, both zeros, the smallest positive double, two values one
/// bit apart, infinities, and NaNs with either sign.
const SPECIAL: [f64; 24] = [
    NOPTS,
    NOPTS,
    -9_223_372_036_854_777_856.0,
    -9_223_372_036_854_774_784.0,
    f64::NEG_INFINITY,
    -1e19,
    -10.0,
    -1.0,
    -0.5,
    -0.0,
    0.0,
    0.0,
    5e-324,
    0.1,
    0.5,
    1.0,
    2.5,
    10.0,
    10.000_000_000_000_002,
    30.0,
    100.0,
    1e18,
    f64::INFINITY,
    f64::NAN,
];

/// A NaN with the sign bit set and a payload, to show whether either survives.
const NEGATIVE_NAN: u64 = 0xFFF8_0000_0000_0ABC;

/// File sizes and positions, with values no double represents exactly.
const OFFSETS: [i64; 12] = [
    -1,
    0,
    0,
    1,
    3,
    50,
    100,
    200,
    1 << 20,
    (1 << 53) + 1,
    i64::MAX - 1,
    i64::MAX,
];

/// The harness holds at most eight chapters.
const MOST_CHAPTERS: i64 = 5;

/// What the script has set, so later values can coincide with it.
#[derive(Debug, Default)]
struct State {
    duration: f64,
    range: (f64, f64),
    chapters: Vec<f64>,
}

fn chance(rng: &mut Rng, percent: i64) -> bool {
    rng.range(0, 99) < percent
}

/// A time. Often one the state already holds, so that a position equals the
/// length, a chapter start, or a range end.
fn time(rng: &mut Rng, state: &State) -> f64 {
    let kind = rng.range(0, 99);
    if kind < 40 {
        rng.pick(&SPECIAL).unwrap_or(0.0)
    } else if kind < 65 {
        // Sevenths are inexact, so sums and quotients of them round.
        f64::from(i32::try_from(rng.range(-70, 700)).unwrap_or(0)) / 7.0
    } else if kind < 93 {
        let mut related = vec![state.duration, state.range.0, state.range.1];
        related.extend(&state.chapters);
        rng.pick(&related).unwrap_or(0.0)
    } else if kind < 96 {
        f64::from_bits(NEGATIVE_NAN)
    } else {
        f64::from_bits(rng.next())
    }
}

fn hex(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn setter(rng: &mut Rng, state: &mut State, kind: i64, out: &mut String) {
    // Writing to a String cannot fail.
    let _ = match kind {
        0 if chance(rng, 20) => writeln!(out, "nodemux"),
        0 => {
            state.duration = if chance(rng, 15) {
                -1.0
            } else {
                time(rng, state)
            };
            let start = if chance(rng, 30) {
                0.0
            } else {
                time(rng, state)
            };
            let size = rng.pick(&OFFSETS).unwrap_or(0);
            let position = rng.pick(&OFFSETS).unwrap_or(0);
            writeln!(
                out,
                "demux {} {} {size} {position}",
                hex(state.duration),
                hex(start)
            )
        }
        1 => {
            let frames = rng.pick(&[-1, 0, 0, 1, 3, 10, 1000]).unwrap_or(0);
            writeln!(out, "opts {} {frames}", rng.range(0, 1))
        }
        2 => {
            let pts = if chance(rng, 40) {
                NOPTS
            } else {
                time(rng, state)
            };
            let direction = rng.pick(&[1, 1, 1, 1, -1, -1, -1, 0, 2]).unwrap_or(1);
            let frames = rng.pick(&[-1, 0, 1, 2, 3, 7, 10, 2000]).unwrap_or(0);
            writeln!(
                out,
                "time {} {direction} {} {frames}",
                hex(pts),
                hex(time(rng, state))
            )
        }
        3 => {
            let count = if chance(rng, 15) {
                0
            } else {
                rng.range(1, MOST_CHAPTERS)
            };
            let mut chapters: Vec<f64> = (0..count).map(|_| time(rng, state)).collect();
            // mpv keeps chapters in time order; an unsorted list checks that
            // the model stops at the same chapter anyway.
            if chance(rng, 75) {
                chapters.sort_by(f64::total_cmp);
            }
            let _ = write!(out, "chapters {count}");
            for chapter in &chapters {
                let _ = write!(out, " {}", hex(*chapter));
            }
            state.chapters = chapters;
            writeln!(out)
        }
        4 => {
            let count = i64::try_from(state.chapters.len()).unwrap_or(0);
            writeln!(
                out,
                "chapseek {} {}",
                rng.range(-3, count + 1),
                i64::from(chance(rng, 30))
            )
        }
        5 => writeln!(out, "cache {}", rng.range(-1, 100)),
        _ => {
            state.range = (time(rng, state), time(rng, state));
            writeln!(out, "range {} {}", hex(state.range.0), hex(state.range.1))
        }
    };
}

fn query(rng: &mut Rng, state: &State, out: &mut String) {
    let count = i64::try_from(state.chapters.len()).unwrap_or(0);
    // Writing to a String cannot fail.
    let _ = match rng.range(0, 11) {
        0 => writeln!(out, "length"),
        1 => writeln!(out, "start {}", rng.range(-2, 2)),
        2 => writeln!(out, "current"),
        3 | 4 => writeln!(out, "playback"),
        5 | 6 => writeln!(out, "ratio 0"),
        7 | 8 => writeln!(out, "ratio 1"),
        9 => writeln!(out, "chapter"),
        10 => writeln!(out, "chapstart {}", rng.range(-3, count + 1)),
        _ if chance(rng, 50) => writeln!(out, "chapcount"),
        _ => writeln!(out, "cachepct"),
    };
}

/// `sequences` independent runs of `operations` operations, each after a `reset`.
pub(crate) fn script(rng: &mut Rng, sequences: usize, operations: usize) -> String {
    let mut out = String::from("nopts\n");
    for _ in 0..sequences {
        out.push_str("reset\n");
        let mut state = State::default();
        // Most sequences fill in everything first, so queries rarely see the
        // all-zero state a reset leaves.
        let mut pending: Vec<i64> = (0..7).filter(|_| chance(rng, 85)).collect();
        for _ in 0..operations {
            if let Some(kind) = pending.pop() {
                setter(rng, &mut state, kind, &mut out);
            } else if chance(rng, 35) {
                let kind = rng.range(0, 6);
                setter(rng, &mut state, kind, &mut out);
            } else {
                query(rng, &state, &mut out);
            }
        }
    }
    out
}
