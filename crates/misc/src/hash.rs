//! Port of `misc/hash.c`.
//!
//! The C hashes with `av_hash_*` of libavutil, which knows many algorithms
//! by name. mpv itself asks for "MD5" and "SHA256" only, and the port carries
//! those two.

use crate::bstr::bstr_xappend;
use crate::clib::strcmp;

const MD5_SHIFTS: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9,
    14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15,
    21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
];

const MD5_K: [u32; 64] = [
    0xd76a_a478,
    0xe8c7_b756,
    0x2420_70db,
    0xc1bd_ceee,
    0xf57c_0faf,
    0x4787_c62a,
    0xa830_4613,
    0xfd46_9501,
    0x6980_98d8,
    0x8b44_f7af,
    0xffff_5bb1,
    0x895c_d7be,
    0x6b90_1122,
    0xfd98_7193,
    0xa679_438e,
    0x49b4_0821,
    0xf61e_2562,
    0xc040_b340,
    0x265e_5a51,
    0xe9b6_c7aa,
    0xd62f_105d,
    0x0244_1453,
    0xd8a1_e681,
    0xe7d3_fbc8,
    0x21e1_cde6,
    0xc337_07d6,
    0xf4d5_0d87,
    0x455a_14ed,
    0xa9e3_e905,
    0xfcef_a3f8,
    0x676f_02d9,
    0x8d2a_4c8a,
    0xfffa_3942,
    0x8771_f681,
    0x6d9d_6122,
    0xfde5_380c,
    0xa4be_ea44,
    0x4bde_cfa9,
    0xf6bb_4b60,
    0xbebf_bc70,
    0x289b_7ec6,
    0xeaa1_27fa,
    0xd4ef_3085,
    0x0488_1d05,
    0xd9d4_d039,
    0xe6db_99e5,
    0x1fa2_7cf8,
    0xc4ac_5665,
    0xf429_2244,
    0x432a_ff97,
    0xab94_23a7,
    0xfc93_a039,
    0x655b_59c3,
    0x8f0c_cc92,
    0xffef_f47d,
    0x8584_5dd1,
    0x6fa8_7e4f,
    0xfe2c_e6e0,
    0xa301_4314,
    0x4e08_11a1,
    0xf753_7e82,
    0xbd3a_f235,
    0x2ad7_d2bb,
    0xeb86_d391,
];

const SHA256_K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// The message followed by the padding both algorithms use: 0x80, zeros up
/// to 8 bytes short of a 64-byte block, then the length in bits.
fn pad(data: &[u8], big_endian_length: bool) -> Vec<u8> {
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    let bits = (data.len() as u64).wrapping_mul(8);
    let mut i = 0;
    while i < 8 {
        let shift = if big_endian_length { 56 - 8 * i } else { 8 * i };
        msg.push((bits >> shift) as u8);
        i += 1;
    }
    msg
}

fn md5_block(state: &mut [u32; 4], block: &[u8]) {
    let mut m = [0_u32; 16];
    let mut i = 0;
    while i < 16 {
        m[i] = block[4 * i] as u32
            | (block[4 * i + 1] as u32) << 8
            | (block[4 * i + 2] as u32) << 16
            | (block[4 * i + 3] as u32) << 24;
        i += 1;
    }
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    i = 0;
    while i < 64 {
        let (f, g) = if i < 16 {
            ((b & c) | (!b & d), i)
        } else if i < 32 {
            ((d & b) | (!d & c), (5 * i + 1) % 16)
        } else if i < 48 {
            (b ^ c ^ d, (3 * i + 5) % 16)
        } else {
            (c ^ (b | !d), (7 * i) % 16)
        };
        let sum = f.wrapping_add(a).wrapping_add(MD5_K[i]).wrapping_add(m[g]);
        a = d;
        d = c;
        c = b;
        b = b.wrapping_add(sum.rotate_left(MD5_SHIFTS[i]));
        i += 1;
    }
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
}

/// MD5 of `data`.
#[must_use]
pub fn md5(data: &[u8]) -> [u8; 16] {
    let msg = pad(data, false);
    let mut state: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];
    let mut off = 0;
    while off < msg.len() {
        md5_block(&mut state, &msg[off..off + 64]);
        off += 64;
    }
    let mut out = [0_u8; 16];
    let mut i = 0;
    while i < 16 {
        out[i] = (state[i / 4] >> (8 * (i % 4))) as u8;
        i += 1;
    }
    out
}

fn sha256_block(state: &mut [u32; 8], block: &[u8]) {
    let mut w = [0_u32; 64];
    let mut i = 0;
    while i < 16 {
        w[i] = (block[4 * i] as u32) << 24
            | (block[4 * i + 1] as u32) << 16
            | (block[4 * i + 2] as u32) << 8
            | block[4 * i + 3] as u32;
        i += 1;
    }
    while i < 64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
        i += 1;
    }
    let mut v = *state;
    i = 0;
    while i < 64 {
        let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
        let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
        let t1 = v[7]
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(SHA256_K[i])
            .wrapping_add(w[i]);
        let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
        let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
        let t2 = s0.wrapping_add(maj);
        v[7] = v[6];
        v[6] = v[5];
        v[5] = v[4];
        v[4] = v[3].wrapping_add(t1);
        v[3] = v[2];
        v[2] = v[1];
        v[1] = v[0];
        v[0] = t1.wrapping_add(t2);
        i += 1;
    }
    i = 0;
    while i < 8 {
        state[i] = state[i].wrapping_add(v[i]);
        i += 1;
    }
}

/// SHA-256 of `data`.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let msg = pad(data, true);
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut off = 0;
    while off < msg.len() {
        sha256_block(&mut state, &msg[off..off + 64]);
        off += 64;
    }
    let mut out = [0_u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (state[i / 4] >> (24 - 8 * (i % 4))) as u8;
        i += 1;
    }
    out
}

/// The uppercase hexadecimal digit for `v < 16`, as `%02X` writes it.
fn upper_hex_digit(v: u8) -> u8 {
    if v < 10 { b'0' + v } else { b'A' + (v - 10) }
}

/// `mp_hash_to_bstr`: the hash of `data` under the algorithm named by the C
/// string `algorithm`, as uppercase hexadecimal.
///
/// # Panics
///
/// If `algorithm` is neither `"MD5"` nor `"SHA256"`. The C fails
/// `mp_require` for a name libavutil does not know; it knows more names
/// than these two and matches them without regard to case.
#[must_use]
pub fn mp_hash_to_bstr(data: &[u8], algorithm: &[u8]) -> Vec<u8> {
    let md5_hash;
    let sha256_hash;
    let digest: &[u8] = if strcmp(algorithm, b"MD5") == 0 {
        md5_hash = md5(data);
        &md5_hash
    } else {
        assert!(strcmp(algorithm, b"SHA256") == 0);
        sha256_hash = sha256(data);
        &sha256_hash
    };

    let mut ret = Vec::new();
    let mut n = 0;
    while n < digest.len() {
        bstr_xappend(
            &mut ret,
            &[
                upper_hex_digit(digest[n] >> 4),
                upper_hex_digit(digest[n] & 15),
            ],
        );
        n += 1;
    }
    ret
}
