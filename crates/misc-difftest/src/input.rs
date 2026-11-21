//! Test input generation from fixed seeds.

/// splitmix64. It shares nothing with the generator under test except the
/// well-known constants.
#[derive(Debug, Clone)]
pub struct Rng(pub u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A value in `0..n`.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    pub fn chance(&mut self, one_in: usize) -> bool {
        self.below(one_in) == 0
    }

    pub fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }

    /// An `int` that is often near a boundary.
    pub fn int(&mut self) -> i32 {
        match self.below(8) {
            0 => self.pick(&[i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX - 1, i32::MAX]),
            1 | 2 => self.below(41) as i32 - 20,
            3 => self.below(600) as i32 - 300,
            4 => self.next_u64() as i32,
            _ => self.below(12) as i32 - 2,
        }
    }
}

const SPECIAL: &[u8] = b"00001289 \t\n\r\x0b\x0c+-._/\\:\"'[]{}(),=xXbBeEpPnNaAfFiI~@m\x1b%#";

/// One byte, biased towards the values the C under test branches on.
pub fn byte(rng: &mut Rng) -> u8 {
    match rng.below(16) {
        0..=5 => rng.pick(SPECIAL),
        6..=8 => b'a' + rng.below(26) as u8,
        9 => b'A' + rng.below(26) as u8,
        10 => b'0' + rng.below(10) as u8,
        // UTF-8 lead and continuation bytes, valid and not.
        11 => rng.pick(&[
            0x80, 0xbf, 0xc0, 0xc2, 0xdf, 0xe0, 0xed, 0xef, 0xf0, 0xf4, 0xf5, 0xff, 0xa0, 0x9f,
            0x90, 0x8f,
        ]),
        12 => 0x80 + rng.below(0x80) as u8,
        13 => rng.below(32) as u8,
        _ => rng.next_u64() as u8,
    }
}

/// Bytes of a length in `0..=max_len`. May hold NUL bytes.
pub fn bytes(rng: &mut Rng, max_len: usize) -> Vec<u8> {
    let len = if rng.chance(8) {
        0
    } else {
        rng.below(max_len + 1)
    };
    (0..len).map(|_| byte(rng)).collect()
}

/// Bytes without a NUL: the content of a C string.
pub fn text(rng: &mut Rng, max_len: usize) -> Vec<u8> {
    let mut v = bytes(rng, max_len);
    for b in &mut v {
        if *b == 0 {
            *b = b'0';
        }
    }
    v
}

/// Well-formed UTF-8 mixed with a few stray bytes.
pub fn utf8ish(rng: &mut Rng, max_chars: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for _ in 0..rng.below(max_chars + 1) {
        match rng.below(12) {
            0 => out.push(byte(rng)),
            1 => out.extend_from_slice(b"\x1b["),
            2 => {
                let cp = rng.pick(&[
                    0x300,
                    0x200d,
                    0xfe0f,
                    0x1f600,
                    0x1f1e6,
                    0x1f3fb,
                    0x1100,
                    0x1160,
                    0x11a8,
                    0xac00,
                    0xac01,
                    0x0e33,
                    0x0903,
                    0x094d,
                    0x0915,
                    0x00ad,
                    0x3000,
                    0xff21,
                    0xd7ff,
                    0xe000,
                    0xffff,
                    0x1_0000,
                    0x0010_ffff,
                    0x7f,
                    0x80,
                    0xa0,
                    0x7ff,
                    0x800,
                    0x2028,
                    0x0600,
                ]);
                push_char(&mut out, cp);
            }
            3 | 4 => push_char(&mut out, rng.below(0x80) as u32),
            5 => push_char(&mut out, 0x80 + rng.below(0x780) as u32),
            6 | 7 => {
                let mut cp = 0x800 + rng.below(0xf800) as u32;
                if (0xd800..0xe000).contains(&cp) {
                    cp = 0x4e00;
                }
                push_char(&mut out, cp);
            }
            8 => push_char(&mut out, 0x1_0000 + rng.below(0x10_0000) as u32),
            _ => out.push(b'a' + rng.below(26) as u8),
        }
    }
    if rng.chance(6) && !out.is_empty() {
        let cut = rng.below(out.len());
        out.truncate(cut);
    }
    out
}

fn push_char(out: &mut Vec<u8>, cp: u32) {
    if let Some(c) = char::from_u32(cp) {
        let mut buf = [0; 4];
        out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    }
}

/// `s` with a terminating NUL, to pass to C as a `char *`. If `s` holds a
/// NUL byte itself, C sees the part before it.
#[must_use]
pub fn cstring(s: &[u8]) -> Vec<u8> {
    let mut v = s.to_vec();
    v.push(0);
    v
}

/// A part of `s`, sometimes with its case changed: a needle that is likely
/// to be found.
pub fn part_of(rng: &mut Rng, s: &[u8]) -> Vec<u8> {
    if s.is_empty() || rng.chance(4) {
        return bytes(rng, 4);
    }
    let a = rng.below(s.len());
    let b = a + rng.below(s.len() - a + 1).min(4);
    let mut v = s[a..b].to_vec();
    if rng.chance(3) {
        for c in &mut v {
            *c = if rng.chance(2) {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            };
        }
    }
    v
}

/// Text shaped like an integer in some base: optional space, sign, prefix,
/// digits and trailing junk, with digit runs up to past the 50 bytes that
/// `bstrtoll` reads.
pub fn integer_text(rng: &mut Rng) -> Vec<u8> {
    let mut out = Vec::new();
    for _ in 0..rng.below(3) {
        if rng.chance(2) {
            out.push(rng.pick(b" \t\n\r\x0b\x0c"));
        }
    }
    match rng.below(5) {
        0 => out.push(b'-'),
        1 => out.push(b'+'),
        _ => {}
    }
    if rng.chance(3) {
        out.extend_from_slice(rng.pick(&[&b"0x"[..], b"0X", b"0b", b"0B", b"0", b"00", b"0o"]));
    }
    match rng.below(8) {
        0 => out.extend_from_slice(rng.pick(&[
            &b"9223372036854775807"[..],
            b"9223372036854775808",
            b"9223372036854775809",
            b"18446744073709551615",
            b"18446744073709551616",
            b"7fffffffffffffff",
            b"8000000000000000",
            b"ffffffffffffffff",
            b"10000000000000000",
            b"777777777777777777777",
            b"1000000000000000000000",
            b"1y2p0ij32e8e7",
            b"1y2p0ij32e8e8",
            b"2147483647",
            b"2147483648",
        ])),
        1 => {
            let d = rng.pick(b"0179fz");
            for _ in 0..40 + rng.below(25) {
                out.push(d);
            }
        }
        _ => {
            let alphabet: &[u8] = rng.pick(&[
                &b"01"[..],
                b"01234567",
                b"0123456789",
                b"0123456789abcdefABCDEF",
                b"0123456789abcdefghijklmnopqrstuvwxyzABCXYZ",
            ]);
            for _ in 0..rng.below(22) {
                out.push(rng.pick(alphabet));
            }
        }
    }
    if rng.chance(2) {
        out.extend_from_slice(&bytes(rng, 4));
    }
    out
}

/// The decimal digits of `m * 5^k`, so that the digits followed by `e-k`
/// are exactly `m * 2^-k`.
fn exact_binary_fraction(m: u64, k: u32) -> Vec<u8> {
    // Little-endian limbs in base 10^9.
    let mut limbs = vec![
        (m % 1_000_000_000) as u32,
        ((m / 1_000_000_000) % 1_000_000_000) as u32,
        (m / 1_000_000_000_000_000_000) as u32,
    ];
    for _ in 0..k {
        let mut carry = 0_u64;
        for limb in &mut limbs {
            let v = u64::from(*limb) * 5 + carry;
            *limb = (v % 1_000_000_000) as u32;
            carry = v / 1_000_000_000;
        }
        if carry != 0 {
            limbs.push(carry as u32);
        }
    }
    while limbs.len() > 1 && limbs.last() == Some(&0) {
        limbs.pop();
    }
    let mut text = limbs.last().map_or_else(String::new, ToString::to_string);
    for limb in limbs.iter().rev().skip(1) {
        let digits = limb.to_string();
        for _ in digits.len()..9 {
            text.push('0');
        }
        text.push_str(&digits);
    }
    text.into_bytes()
}

/// Text shaped like what `strtod` reads: decimal and hexadecimal numbers,
/// infinities and NaNs, with values at the edges of the `double` range.
pub fn float_text(rng: &mut Rng) -> Vec<u8> {
    let mut out = Vec::new();
    if rng.chance(6) {
        out.push(rng.pick(b" \t\n"));
    }
    match rng.below(5) {
        0 => out.push(b'-'),
        1 => out.push(b'+'),
        _ => {}
    }
    match rng.below(14) {
        0 => out.extend_from_slice(rng.pick(&[
            &b"inf"[..], b"INF", b"Infinity", b"infinit", b"infinityx", b"nan", b"NaN", b"nan()",
            b"nan(123)", b"nan(0x7ff)", b"nan(abc_9)", b"nan(0b11)", b"nan(", b"nan(1", b"nan(1 )",
            b"nan(99999999999999999999)", b"nan(99999999999999999999x)", b"nan(0x99999999999999999999)", b"nan(_)", b"nan(+5)", b"nan(0x8000000000001)", b"nan(-1)", b"nan(017)", b"nanx", b"in", b"na", b".", b".e5", b"e5",
            b"0x", b"0x.", b"0xp1", b"0x.p1", b"0xg", b"0x1p", b"0x1p+", b"0x1.p1", b"0x.1p1", b"1e", b"1e+", b"1.e3",
        ])),
        1 => out.extend_from_slice(rng.pick(&[
            &b"1.7976931348623157e308"[..],
            b"1.7976931348623158e308",
            b"1.7976931348623159e308",
            b"179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497791",
            b"179769313486231580793728971405303415079934132710037826936173778980444968292764750946649017977587207096330286416692887910946555547851940402630657488671505820681908902000708383676273854845817711531764475730270069855571366959622842914819860834936475292719074168444365510704342711559699508093042880177904174497792",
            b"2.2250738585072014e-308",
            b"2.2250738585072011e-308",
            b"2.2250738585072012e-308",
            b"2.225073858507201e-308",
            b"2.22507385850720138e-308",
            b"4.9406564584124654e-324",
            b"2.4703282292062327e-324",
            b"2.4703282292062328e-324",
            b"2.4703282292062329e-324",
            b"5e-324",
            b"3e-324",
            b"2e-324",
            b"1e-400",
            b"1e400",
            b"0e400",
            b"0.0e-99999",
            b"1e99999999999999999999",
            b"1e-99999999999999999999",
            b"0x1p-1074",
            b"0x1p-1075",
            b"0x1.8p-1075",
            b"0x1.0000000000001p-1075",
            b"0x0.fffffffffffffp-1022",
            b"0x0.fffffffffffff8p-1022",
            b"0x0.fffffffffffffcp-1022",
            b"0x0.fffffffffffffbp-1022",
            b"0x0.fffffffffffff7fp-1022",
            b"0x1.fffffffffffffp1023",
            b"0x1.fffffffffffff8p1023",
            b"0x1.fffffffffffff7p1023",
            b"0x1p1024",
            b"0x1.00000000000008p0",
            b"0x1.000000000000080001p0",
            b"0x1.00000000000018p0",
            b"0x123456789abcdef01234p-40",
            b"0x.00000000000000000001p80",
            b"0x1p-99999999999999999999",
            b"0x1p99999999999999999999",
            b"0x0p99999999999999999999",
            b"4.4501477170144023e-308",
            b"8.98846567431158e307",
        ])),
        2 | 3 => {
            out.extend_from_slice(b"0x");
            for _ in 0..rng.below(20) {
                out.push(rng.pick(b"0123456789abcdefABCDEF"));
            }
            if rng.chance(2) {
                out.push(b'.');
                for _ in 0..rng.below(20) {
                    out.push(rng.pick(b"0123456789abcdef08f"));
                }
            }
            if rng.chance(2) {
                out.push(rng.pick(b"pP"));
                match rng.below(4) {
                    0 => out.push(b'-'),
                    1 => out.push(b'+'),
                    _ => {}
                }
                let exp = match rng.below(4) {
                    0 => rng.below(10),
                    1 => 1000 + rng.below(100),
                    _ => rng.below(1200),
                };
                out.extend_from_slice(exp.to_string().as_bytes());
            }
        }
        4 => {
            // More than the 100 bytes bstrtod reads.
            let d = rng.pick(b"0159");
            for _ in 0..90 + rng.below(30) {
                out.push(d);
            }
            if rng.chance(2) {
                out.extend_from_slice(b"e-5");
            }
        }
        12 => {
            // The exact decimal expansion of m * 2^-k for a subnormal or
            // barely normal value, sometimes off by one in the last place:
            // glibc reports ERANGE for a tiny result only if it is inexact.
            let (m, k) = match rng.below(6) {
                0 => (1 + rng.below(8) as u64, 1074),
                1 => (rng.next_u64() >> 12, 1074),
                2 => ((1 << 54) - 1, 1076),
                3 => ((1 << 53) - 1, 1075),
                4 => ((1 << 52) - rng.below(2) as u64, 1074),
                _ => ((rng.next_u64() >> 11) | 1, 1074 + rng.below(4) as u32),
            };
            let mut digits = exact_binary_fraction(m, k);
            if rng.chance(2) {
                let last = digits.len() - 1;
                digits[last] = if digits[last] == b'5' { rng.pick(b"46") } else { b'5' };
            }
            out.extend_from_slice(&digits);
            out.extend_from_slice(format!("e-{k}").as_bytes());
        }
        5 => {
            // Around the smallest numbers, where glibc reports ERANGE.
            let digits = 1 + rng.below(18);
            for i in 0..digits {
                out.push(rng.pick(b"0123456789"));
                if i == 0 {
                    out.push(b'.');
                }
            }
            out.extend_from_slice(format!("e-{}", 300 + rng.below(30)).as_bytes());
        }
        _ => {
            for _ in 0..rng.below(20) {
                out.push(rng.pick(b"0123456789"));
            }
            if rng.chance(2) {
                out.push(b'.');
                for _ in 0..rng.below(20) {
                    out.push(rng.pick(b"0123456789"));
                }
            }
            if rng.chance(2) {
                out.push(rng.pick(b"eE"));
                match rng.below(4) {
                    0 => out.push(b'-'),
                    1 => out.push(b'+'),
                    _ => {}
                }
                let exp = match rng.below(4) {
                    0 => rng.below(10),
                    1 => 290 + rng.below(40),
                    _ => rng.below(400),
                };
                out.extend_from_slice(exp.to_string().as_bytes());
            }
        }
    }
    if rng.chance(3) {
        out.extend_from_slice(&bytes(rng, 3));
    }
    out
}

/// `a`, or something that shares a prefix with it.
pub fn near(rng: &mut Rng, a: &[u8]) -> Vec<u8> {
    let mut b = a.to_vec();
    match rng.below(6) {
        0 => {}
        1 if !b.is_empty() => {
            let i = rng.below(b.len());
            b[i] = byte(rng);
        }
        2 if !b.is_empty() => {
            let i = rng.below(b.len());
            b[i] = if b[i].is_ascii_lowercase() {
                b[i].to_ascii_uppercase()
            } else {
                b[i].to_ascii_lowercase()
            };
        }
        3 => {
            let cut = rng.below(b.len() + 1);
            b.truncate(cut);
        }
        4 => b.extend_from_slice(&bytes(rng, 3)),
        _ => b = bytes(rng, 12),
    }
    b
}
