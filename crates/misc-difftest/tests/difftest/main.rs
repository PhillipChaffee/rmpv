//! Differential tests: each test runs one mpv C function and its port on the
//! same generated inputs and compares every result and output.
//!
//! Each test prints `difftest <function>: <n> inputs agree`. A function is
//! named for the C function, so the lines can be matched with mpv's source.

#![cfg(mpv_build)]

mod bstr;
mod clib;
mod fs;
mod printf;
mod small;
mod threads;
mod tree;

use rmpv_misc_difftest::input::Rng;

/// Inputs per function, unless a test says why it runs fewer.
pub const N: usize = 100_000;

/// Runs `case` on `count` inputs drawn from a generator seeded with `seed`.
pub fn run(name: &str, seed: u64, count: usize, mut case: impl FnMut(&mut Rng)) {
    let mut rng = Rng(seed);
    for _ in 0..count {
        case(&mut rng);
    }
    println!("difftest {name}: {count} inputs agree");
}
