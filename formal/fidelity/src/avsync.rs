//! Generates random operation scripts for the audio and video targets, which
//! check the A/V sync arithmetic of `player/audio.c` and `player/video.c`.
//!
//! A `double` crosses the script as the 16 hex digits of its bit pattern and a
//! `float` as 8, so mpv and the model start from the identical value.

use crate::playloop::{NEGATIVE_NAN, NOPTS, SPECIAL, chance, hex};
use crate::rng::Rng;
use std::fmt::Write as _;

/// `MP_PTS_ROUNDING_TOLERANCE`. The `consts` operation checks the model's
/// value against mpv's own.
const TOLERANCE: f64 = 0.001 * 3.0 + 0.0001;

/// Gains in dB, as the `float` options and replaygain tags hold them.
const GAINS: [f32; 22] = [
    0.0,
    0.0,
    -0.0,
    1.0,
    -1.0,
    6.0,
    -6.0,
    3.5,
    -7.25,
    20.0,
    -20.0,
    89.0,
    0.1,
    -0.1,
    100.0,
    -100.0,
    1000.0,
    f32::MAX,
    f32::MIN_POSITIVE,
    f32::INFINITY,
    f32::NEG_INFINITY,
    f32::NAN,
];

/// Volumes in percent.
const VOLUMES: [f32; 14] = [
    0.0,
    -0.0,
    100.0,
    100.0,
    50.0,
    33.3,
    130.0,
    1000.0,
    1.0,
    1e-20,
    -5.0,
    f32::NEG_INFINITY,
    f32::INFINITY,
    f32::NAN,
];

/// Replaygain peaks; clipping prevention divides by them.
const PEAKS: [f32; 13] = [
    0.0,
    -0.0,
    1.0,
    1.0,
    0.5,
    0.988,
    0.25,
    2.0,
    1.5,
    -1.0,
    f32::MIN_POSITIVE,
    f32::INFINITY,
    f32::NAN,
];

/// The `--mc` option: negative means automatic.
const CORRECTIONS: [f32; 12] = [
    -1.0,
    -1.0,
    -1.0,
    0.0,
    -0.0,
    0.004,
    0.01,
    0.5,
    1.0,
    f32::INFINITY,
    f32::NEG_INFINITY,
    f32::NAN,
];

/// A/V differences and delays, with both thresholds the sync code tests (0.3
/// and 0.5 seconds) and the doubles next to them.
const OFFSETS: [f64; 22] = [
    0.0,
    0.0,
    0.0,
    0.0,
    -0.0,
    0.3,
    -0.3,
    0.299_999_999_999_999_93,
    0.300_000_000_000_000_04,
    0.5,
    -0.5,
    0.499_999_999_999_999_94,
    0.500_000_000_000_000_1,
    0.1,
    -0.1,
    0.04,
    0.2,
    1.0,
    -1.0,
    2.0,
    0.01,
    -0.02,
];

/// Frame durations: common frame rates, the same rounded to a millisecond as
/// Matroska stores them, and one below the rounding tolerance.
const FRAME_TIMES: [f64; 14] = [
    1.0 / 24.0,
    1001.0 / 24000.0,
    1001.0 / 24000.0,
    1.0 / 25.0,
    1.0 / 30.0,
    1.0 / 60.0,
    0.04,
    0.041,
    0.042,
    0.033,
    0.034,
    0.016,
    0.017,
    0.001,
];

/// Display refresh intervals. Some equal a frame duration or divide one
/// exactly, so that a speed of exactly 1 comes up.
const VSYNCS: [f64; 14] = [
    1.0 / 60.0,
    1.0 / 60.0,
    1.0 / 59.94,
    1.0 / 50.0,
    1.0 / 144.0,
    1.0 / 75.0,
    1.0 / 24.0,
    1.0 / 30.0,
    1.0 / 64.0,
    0.0,
    -1.0 / 60.0,
    f64::INFINITY,
    f64::NEG_INFINITY,
    f64::NAN,
];

/// Durations whose ratio to a vsync of 1/64 is an exact half-integer, where
/// `rint` and `round` disagree.
const HALVES: [f64; 7] = [
    0.5 / 64.0,
    1.5 / 64.0,
    2.5 / 64.0,
    3.5 / 64.0,
    1.25 / 64.0,
    2.0 / 64.0,
    -2.5 / 64.0,
];

const MAX_CHANGES: [f64; 10] = [
    0.01,
    0.01,
    0.0,
    0.001,
    0.25,
    0.5,
    1.0,
    -1.0,
    f64::INFINITY,
    f64::NAN,
];

/// Playback speeds and speed factors.
const SPEEDS: [f64; 12] = [
    1.0,
    1.0,
    1.0,
    0.5,
    2.0,
    1.001,
    0.999,
    1.25,
    0.0,
    -1.0,
    f64::INFINITY,
    f64::NAN,
];

/// The `video-sync-max-*-change` options, in percent.
const PERCENTS: [f64; 10] = [
    1.0,
    1.0,
    0.125,
    0.125,
    5.0,
    100.0,
    0.0,
    -1.0,
    f64::INFINITY,
    f64::NAN,
];

/// How far a past frame's duration is from the others: nothing, within the
/// rounding tolerance, exactly on it, and just outside.
const JITTERS: [f64; 14] = [
    0.0, 0.0, 0.0, 0.001, -0.001, 0.0005, -0.0005, 0.003, -0.003, 0.003_05, -0.003_05, TOLERANCE,
    -TOLERANCE, 0.0032,
];

fn hex32(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn pick(rng: &mut Rng, items: &[f64]) -> f64 {
    rng.pick(items).unwrap_or(0.0)
}

fn small(rng: &mut Rng, low: i64, high: i64) -> i32 {
    i32::try_from(rng.range(low, high)).unwrap_or(0)
}

/// Any double: mostly the edge cases, sometimes random bits.
fn wild(rng: &mut Rng) -> f64 {
    let kind = rng.range(0, 99);
    if kind < 70 {
        pick(rng, &SPECIAL)
    } else if kind < 80 {
        f64::from_bits(NEGATIVE_NAN)
    } else {
        f64::from_bits(rng.next())
    }
}

/// A value from `pool`, an inexact fraction of the same size, or any double.
fn value(rng: &mut Rng, pool: &[f64]) -> f64 {
    let kind = rng.range(0, 99);
    if kind < 55 {
        pick(rng, pool)
    } else if kind < 85 {
        // Seventieths are inexact, so sums and products of them round.
        f64::from(small(rng, -40, 40)) / 70.0
    } else {
        wild(rng)
    }
}

fn value32(rng: &mut Rng, pool: &[f32]) -> f32 {
    let kind = rng.range(0, 99);
    if kind < 55 {
        rng.pick(pool).unwrap_or(0.0)
    } else if kind < 85 {
        f32::from(i16::try_from(rng.range(-420, 420)).unwrap_or(0)) / 7.0
    } else {
        f32::from_bits(u32::try_from(rng.next() >> 32).unwrap_or(0))
    }
}

fn flag(rng: &mut Rng, percent: i64) -> i64 {
    i64::from(chance(rng, percent))
}

/// Sets the inputs of the audio clock, which both targets have: the output
/// chain for kind 0, else the output with its speed and delay.
fn clock_setter(rng: &mut Rng, kind: i64, out: &mut String) {
    // Writing to a String cannot fail.
    let _ = if kind == 0 {
        if chance(rng, 20) {
            writeln!(out, "nochain")
        } else {
            let pts = if chance(rng, 30) {
                NOPTS
            } else {
                value(rng, &OFFSETS)
            };
            writeln!(out, "chain {} {}", hex(pts), flag(rng, 70))
        }
    } else {
        let speed = value(rng, &SPEEDS);
        let delay = value(rng, &OFFSETS);
        writeln!(out, "ao {} {} {}", flag(rng, 70), hex(speed), hex(delay))
    };
}

fn audio_setter(rng: &mut Rng, kind: i64, out: &mut String) {
    // Writing to a String cannot fail.
    let _ = match kind {
        0 | 1 => {
            clock_setter(rng, kind, out);
            Ok(())
        }
        2 => writeln!(
            out,
            "vol {} {} {}",
            hex32(value32(rng, &VOLUMES)),
            hex32(value32(rng, &GAINS)),
            flag(rng, 15)
        ),
        3 => {
            let mode = rng.pick(&[0, 0, 1, 1, 1, 2, 2, 3, -1]).unwrap_or(0);
            let fallback = if chance(rng, 40) {
                0.0
            } else {
                value32(rng, &GAINS)
            };
            writeln!(
                out,
                "rgopts {mode} {} {} {}",
                hex32(value32(rng, &GAINS)),
                flag(rng, 40),
                hex32(fallback)
            )
        }
        _ if chance(rng, 15) => writeln!(out, "notrack"),
        _ if chance(rng, 15) => writeln!(out, "norg"),
        _ => writeln!(
            out,
            "rg {} {} {} {}",
            hex32(value32(rng, &GAINS)),
            hex32(value32(rng, &PEAKS)),
            hex32(value32(rng, &GAINS)),
            hex32(value32(rng, &PEAKS))
        ),
    };
}

fn audio_query(rng: &mut Rng, out: &mut String) {
    // Writing to a String cannot fail.
    let _ = match rng.range(0, 9) {
        0..=2 => {
            let db = match rng.range(0, 2) {
                0 => wild(rng),
                1 => f64::from(small(rng, -420, 420)) / 7.0,
                _ => f64::from(value32(rng, &GAINS)),
            };
            writeln!(out, "dbgain {}", hex(db))
        }
        3 | 4 => writeln!(out, "replaygain"),
        5 | 6 => writeln!(out, "gain"),
        7 => writeln!(out, "written"),
        _ => writeln!(out, "playing"),
    };
}

/// `sequences` independent runs of `operations` operations on the audio
/// target, each after a `reset`.
pub(crate) fn audio_script(rng: &mut Rng, sequences: usize, operations: usize) -> String {
    let mut out = String::from("nopts\n");
    for _ in 0..sequences {
        out.push_str("reset\n");
        // Most sequences fill in everything first, so queries rarely see the
        // all-zero state a reset leaves.
        let mut pending: Vec<i64> = (0..5).filter(|_| chance(rng, 85)).collect();
        for _ in 0..operations {
            if let Some(kind) = pending.pop() {
                audio_setter(rng, kind, &mut out);
            } else if chance(rng, 35) {
                let kind = rng.range(0, 4);
                audio_setter(rng, kind, &mut out);
            } else {
                audio_query(rng, &mut out);
            }
        }
    }
    out
}

/// A playback status: mostly `STATUS_PLAYING`, which is 2.
fn status(rng: &mut Rng) -> i64 {
    rng.pick(&[2, 2, 2, 2, 2, 2, 0, 1, 3, 4]).unwrap_or(2)
}

/// A frame history without a pattern, for the functions that average
/// `approx_duration`.
fn past(rng: &mut Rng, out: &mut String) -> i64 {
    let count = rng.pick(&[0, 1, 1, 2, 3, 5, 8]).unwrap_or(0);
    let base = pick(rng, &FRAME_TIMES);
    // Writing to a String cannot fail.
    let _ = write!(out, "past {count}");
    for _ in 0..count {
        let approx = match rng.range(0, 9) {
            0..=4 => base,
            5 => 0.0,
            6 => pick(rng, &[-1.0, -0.0, f64::NAN, f64::INFINITY]),
            7 => pick(rng, &VSYNCS),
            _ => value(rng, &FRAME_TIMES),
        };
        let duration = value(rng, &FRAME_TIMES);
        let _ = write!(out, " {} {}", hex(duration), hex(approx));
    }
    let _ = writeln!(out);
    count
}

/// An output chain, the next frames and a frame history shaped like playback:
/// frames about `base` apart, with timestamp rounding and at most one outlier.
/// These are the inputs of `calculate_frame_duration`.
fn frame_scenario(rng: &mut Rng, out: &mut String) {
    let base = pick(rng, &FRAME_TIMES);
    let fps = match rng.range(0, 9) {
        0..=5 => 1.0 / base,
        6 => pick(rng, &[24.0, 25.0, 30.0, 60.0, 24000.0 / 1001.0]),
        7 => pick(rng, &[0.0, -0.0, -1.0]),
        8 => pick(rng, &[f64::INFINITY, f64::NAN]),
        _ => wild(rng),
    };
    // Writing to a String cannot fail.
    let _ = writeln!(out, "vo {} {}", flag(rng, 10), hex(fps));

    // Equal timestamps give a duration of 0, which puts a past duration of
    // exactly the tolerance on the edge of the outlier test.
    let edge = chance(rng, 6);
    let pts0 = match rng.range(0, 9) {
        0 => NOPTS,
        1 => wild(rng),
        _ => f64::from(small(rng, 0, 1000)) * base,
    };
    let gap = match rng.range(0, 9) {
        _ if edge => 0.0,
        0..=3 => base + pick(rng, &JITTERS),
        4 => 0.0,
        5 => -base,
        6 => 2.0 * base,
        7 => (base * 1000.0).round() / 1000.0,
        8 => (base * 1000.0).floor() / 1000.0,
        _ => wild(rng),
    };
    let (pts0, pts1) = if chance(rng, 4) {
        // A known first timestamp below an unknown second one.
        let below = [f64::NEG_INFINITY, -1e19, -9_223_372_036_854_777_856.0];
        (pick(rng, &below), NOPTS)
    } else if chance(rng, 8) {
        (pts0, NOPTS)
    } else {
        (pts0, pts0 + gap)
    };
    let count = rng.pick(&[1, 2, 2, 2, 2, 3]).unwrap_or(1);
    let _ = write!(out, "next {count}");
    for pts in [pts0, pts1, pts1 + gap].iter().take(count) {
        let _ = write!(out, " {}", hex(*pts));
    }
    let _ = writeln!(out);

    // The demuxer's frame time is only taken with 16 similar frames or with
    // all but 4, so the lengths around 21 matter.
    let count = match rng.range(0, 9) {
        0..=2 => rng.range(1, 5),
        3..=6 => rng.range(6, 20),
        _ => rng.range(21, 24),
    };
    let outlier_at = (!chance(rng, 35)).then(|| rng.range(1, count));
    let pattern = rng.range(0, 3);
    let _ = write!(out, "past {count}");
    for index in 0..count {
        let similar = match pattern {
            _ if edge => TOLERANCE,
            0 => gap,
            1 => base,
            2 => base + pick(rng, &JITTERS),
            // Millisecond timestamps: each duration is rounded down or up.
            _ => ((base * 1000.0).floor() + f64::from(small(rng, 0, 1))) / 1000.0,
        };
        let duration = if Some(index) == outlier_at {
            let outliers = [
                0.0,
                -1.0,
                f64::NAN,
                base * 2.0,
                base / 2.0,
                base + TOLERANCE,
                base - TOLERANCE,
                TOLERANCE,
                wild(rng),
            ];
            pick(rng, &outliers)
        } else if index == 0 && chance(rng, 50) {
            // The function overwrites the newest frame without reading it.
            wild(rng)
        } else {
            similar
        };
        let approx = if chance(rng, 80) { base } else { wild(rng) };
        let _ = write!(out, " {} {}", hex(duration), hex(approx));
    }
    let _ = writeln!(out);
}

/// `frames_ready` says whether the preconditions of `calculate_frame_duration`
/// hold: an output chain and at least one past and one next frame.
fn video_setter(rng: &mut Rng, frames_ready: &mut bool, kind: i64, out: &mut String) {
    // Writing to a String cannot fail.
    let _ = match kind {
        0 | 1 => {
            clock_setter(rng, kind, out);
            Ok(())
        }
        2 => writeln!(out, "status {} {}", status(rng), status(rng)),
        3 => {
            let correction = value32(rng, &CORRECTIONS);
            writeln!(
                out,
                "sync {} {} {} {} {}",
                hex(value(rng, &OFFSETS)),
                hex32(correction),
                hex(value(rng, &OFFSETS)),
                hex(value(rng, &OFFSETS)),
                flag(rng, 30)
            )
        }
        4 => {
            let pts = if chance(rng, 20) {
                NOPTS
            } else {
                value(rng, &OFFSETS)
            };
            writeln!(
                out,
                "avstate {} {} {} {}",
                hex(value(rng, &OFFSETS)),
                hex(pts),
                hex(value(rng, &OFFSETS)),
                flag(rng, 30)
            )
        }
        5 if chance(rng, 25) => {
            *frames_ready = false;
            writeln!(out, "novo")
        }
        5 => {
            let fps = value(rng, &[24.0, 25.0, 30.0, 60.0, 0.0]);
            writeln!(out, "vo {} {}", flag(rng, 25), hex(fps))
        }
        6 => {
            let factor = rng.pick(&[-1, 0, 1, 1, 2, 5, 5, 10]).unwrap_or(0);
            writeln!(
                out,
                "speed {} {} {} {factor}",
                hex(value(rng, &SPEEDS)),
                hex(value(rng, &PERCENTS)),
                hex(value(rng, &PERCENTS))
            )
        }
        7 => {
            // 1 and 4 are the two modes that resample audio.
            let mode = rng.pick(&[1, 1, 1, 4, 4, 4, 0, 2, 3, 5, 8]).unwrap_or(0);
            let compensation = value(rng, &[0.0, 0.0, 0.001, -0.001, 0.000_5, 0.000_01]);
            writeln!(
                out,
                "dsync {mode} {} {} {} {}",
                hex(value(rng, &SPEEDS)),
                hex(value(rng, &SPEEDS)),
                hex(compensation),
                hex(value(rng, &OFFSETS))
            )
        }
        8 => writeln!(out, "fmt {} {}", rng.range(0, 3), rng.range(0, 3)),
        9 => {
            *frames_ready &= past(rng, out) > 0;
            Ok(())
        }
        _ => {
            frame_scenario(rng, out);
            *frames_ready = true;
            Ok(())
        }
    };
}

/// The arguments of `calc_best_speed`.
fn best_speed(rng: &mut Rng, out: &mut String) {
    let (vsync, frame) = match rng.range(0, 9) {
        // An exact half-integer number of vsyncs per frame.
        0..=2 => (1.0 / 64.0, pick(rng, &HALVES)),
        3..=5 => (pick(rng, &VSYNCS), pick(rng, &FRAME_TIMES)),
        6 | 7 => (pick(rng, &VSYNCS), pick(rng, &VSYNCS)),
        _ => (wild(rng), wild(rng)),
    };
    let max_change = if chance(rng, 85) {
        pick(rng, &MAX_CHANGES)
    } else {
        wild(rng)
    };
    let max_factor = rng.pick(&[-1, 0, 1, 1, 2, 5, 5, 10]).unwrap_or(0);
    // Writing to a String cannot fail.
    let _ = writeln!(
        out,
        "bestspeed {} {} {} {max_factor}",
        hex(vsync),
        hex(frame),
        hex(max_change)
    );
}

fn video_query(rng: &mut Rng, frames_ready: &mut bool, out: &mut String) {
    // Writing to a String cannot fail.
    let _ = match rng.range(0, 21) {
        0..=2 => {
            let frame_time = value(rng, &FRAME_TIMES);
            writeln!(
                out,
                "adjsync {} {}",
                hex(value(rng, &OFFSETS)),
                hex(frame_time)
            )
        }
        3..=5 => writeln!(out, "avdiff {}", hex(value(rng, &OFFSETS))),
        6 => writeln!(out, "avgdur"),
        7..=9 => {
            best_speed(rng, out);
            Ok(())
        }
        10 | 11 => writeln!(out, "findspeed {}", hex(value(rng, &VSYNCS))),
        12 => writeln!(out, "spdif"),
        13..=15 => {
            let duration = if chance(rng, 70) {
                pick(rng, &FRAME_TIMES)
            } else {
                value(rng, &[0.0, -1.0, -0.5, 1.0])
            };
            writeln!(out, "drift {}", hex(duration))
        }
        16 => {
            let x = match rng.range(0, 3) {
                // Ties, which round to even.
                0 => f64::from(small(rng, -21, 21)) / 2.0,
                1 => f64::from(small(rng, -70, 70)) / 7.0,
                2 => pick(
                    rng,
                    &[
                        0.499_999_999_999_999_94,
                        -0.499_999_999_999_999_94,
                        4_503_599_627_370_495.5,
                        -4_503_599_627_370_495.5,
                        4_503_599_627_370_497.0,
                        -1e-300,
                        1e-300,
                    ],
                ),
                _ => wild(rng),
            };
            writeln!(out, "rint {}", hex(x))
        }
        _ => {
            if !*frames_ready || chance(rng, 70) {
                frame_scenario(rng, out);
                *frames_ready = true;
            }
            writeln!(out, "framedur")
        }
    };
}

/// `sequences` independent runs of about `operations` operations on the video
/// target, each after a `reset`.
pub(crate) fn video_script(rng: &mut Rng, sequences: usize, operations: usize) -> String {
    let mut out = String::from("consts\n");
    for _ in 0..sequences {
        out.push_str("reset\n");
        let mut frames_ready = false;
        // Most sequences fill in everything first, so queries rarely see the
        // all-zero state a reset leaves.
        let mut pending: Vec<i64> = (0..10).filter(|_| chance(rng, 85)).collect();
        for _ in 0..operations {
            if let Some(kind) = pending.pop() {
                video_setter(rng, &mut frames_ready, kind, &mut out);
            } else if chance(rng, 30) {
                let kind = rng.range(0, 10);
                video_setter(rng, &mut frames_ready, kind, &mut out);
            } else {
                video_query(rng, &mut frames_ready, &mut out);
            }
        }
    }
    out
}
