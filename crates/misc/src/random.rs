//! Port of `misc/random.c`: xoshiro256** seeded with splitmix64.
//!
//! At the pinned commit the generator state is already an explicit
//! `mp_rand_state` value passed by pointer, so the port has no hidden state
//! either.

use crate::common::mp_round_next_power_of_2;
use crate::os;

/// `mp_rand_state`.
#[cfg_attr(not(aeneas), derive(Debug, Clone, Copy, PartialEq, Eq))]
pub struct MpRandState {
    pub v: [u64; 4],
}

/// `rotl_u64`. Like the C, this is only defined for `0 < k < 64`.
#[must_use]
pub fn rotl_u64(x: u64, k: i32) -> u64 {
    (x << k) | (x >> (64 - k))
}

/// `splitmix64`.
pub fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// The part of `mp_rand_seed` that runs for a known seed.
#[must_use]
pub fn mp_rand_seed_from(seed: u64) -> MpRandState {
    let mut seed = seed;
    let mut ret = MpRandState { v: [0; 4] };
    ret.v[0] = seed;
    let mut i = 1;
    while i < 4 {
        ret.v[i] = splitmix64(&mut seed);
        i += 1;
    }
    ret
}

/// `mp_rand_seed`. A seed of 0 asks for a state from the operating system.
#[must_use]
pub fn mp_rand_seed(seed: u64) -> MpRandState {
    if seed == 0 {
        if let Some(v) = os::random_state() {
            return MpRandState { v };
        }
        return mp_rand_seed_from(os::clock_seed());
    }
    mp_rand_seed_from(seed)
}

/// `mp_rand_next`.
pub fn mp_rand_next(s: &mut MpRandState) -> u64 {
    let result = rotl_u64(s.v[1].wrapping_mul(5), 7).wrapping_mul(9);
    let t = s.v[1] << 17;

    s.v[2] ^= s.v[0];
    s.v[3] ^= s.v[1];
    s.v[1] ^= s.v[2];
    s.v[0] ^= s.v[3];
    s.v[2] ^= t;
    s.v[3] = rotl_u64(s.v[3], 45);

    result
}

/// `mp_rand_next_double`: uniform in `[0, 1)`.
pub fn mp_rand_next_double(s: &mut MpRandState) -> f64 {
    // 2^-53, the C's 0x1.0p-53.
    const SCALE: f64 = 1.0 / 9_007_199_254_740_992.0;
    (mp_rand_next(s) >> 11) as f64 * SCALE
}

/// `mp_rand_in_range32`.
///
/// # Panics
///
/// If `min >= max`, where the C fails `mp_assert`.
pub fn mp_rand_in_range32(s: &mut MpRandState, min: u32, max: u32) -> u32 {
    assert!(min < max);
    let range = max - min;
    let mask = mp_round_next_power_of_2(range).wrapping_sub(1);
    let mut ret = mp_rand_next(s) as u32 & mask;
    while ret >= range {
        ret = mp_rand_next(s) as u32 & mask;
    }
    min.wrapping_add(ret)
}
