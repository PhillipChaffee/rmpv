//! Stand-ins for the C library routines that Aeneas is not asked to
//! translate: `strtod`, and the two `printf` conversions `misc/` uses.
//!
//! Their signatures have no floating point: a `double` crosses as its bit
//! pattern. Charon is told to keep this module opaque, so in Lean these are
//! declarations without a body, as a C library call is on the C side.

use core::cmp::Ordering;

use crate::clib::{Strtod, cstr_at, digit_value, isspace, scan_integer, tolower};

const SIGN_BIT: u64 = 1 << 63;
const INF_BITS: u64 = 0x7ff0_0000_0000_0000;

const QNAN_BITS: u64 = 0x7ff8_0000_0000_0000;
const MIN_NORMAL_BITS: u64 = 1 << 52;

fn is_xdigit(c: u8) -> bool {
    digit_value(c) < 16
}

fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn starts_with_ci(s: &[u8], at: usize, word: &[u8]) -> bool {
    let mut i = 0;
    while i < word.len() {
        if tolower(cstr_at(s, at + i)) != word[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// An unsigned integer of any size, least significant limb first.
struct Big(Vec<u32>);

impl Big {
    fn from_u64(v: u64) -> Self {
        Self(vec![v as u32, (v >> 32) as u32])
    }

    fn mul_add_small(&mut self, mul: u32, add: u32) {
        let mut carry = add as u64;
        for limb in &mut self.0 {
            let v = *limb as u64 * mul as u64 + carry;
            *limb = v as u32;
            carry = v >> 32;
        }
        if carry != 0 {
            self.0.push(carry as u32);
        }
    }

    fn mul_pow(&mut self, base: u32, mut exp: u64) {
        // The largest powers of 2 and 10 that fit a limb multiplication.
        let (chunk, chunk_exp) = if base == 2 {
            (1 << 31, 31)
        } else {
            (1_000_000_000, 9)
        };
        while exp >= chunk_exp {
            self.mul_add_small(chunk, 0);
            exp -= chunk_exp;
        }
        while exp > 0 {
            self.mul_add_small(base, 0);
            exp -= 1;
        }
    }

    fn cmp(&self, other: &Self) -> Ordering {
        let len = self.0.len().max(other.0.len());
        for i in (0..len).rev() {
            let a = self.0.get(i).copied().unwrap_or(0);
            let b = other.0.get(i).copied().unwrap_or(0);
            if a != b {
                return a.cmp(&b);
            }
        }
        Ordering::Equal
    }
}

/// Compares `digits * 10^e10` with `m * 2^e2` exactly.
fn cmp_decimal_binary(digits: &[u8], e10: i64, m: u64, e2: i64) -> Ordering {
    let mut lhs = Big(vec![0]);
    for &d in digits {
        lhs.mul_add_small(10, (d - b'0') as u32);
    }
    let mut rhs = Big::from_u64(m);
    if e10 >= 0 {
        lhs.mul_pow(10, e10.unsigned_abs());
    } else {
        rhs.mul_pow(10, e10.unsigned_abs());
    }
    if e2 >= 0 {
        rhs.mul_pow(2, e2.unsigned_abs());
    } else {
        lhs.mul_pow(2, e2.unsigned_abs());
    }
    lhs.cmp(&rhs)
}

/// Rounds `m * 2^e2`, plus a little more if `sticky`, to the nearest
/// `double`, ties to even. Returns the bits without sign and whether glibc
/// sets `ERANGE`: on overflow, and on a result that is both inexact and tiny
/// after rounding.
fn round_binary(m: u64, e2: i64, sticky: bool) -> (u64, bool) {
    if m == 0 {
        return (0, false);
    }
    let lz = m.leading_zeros();
    let m = m << lz;
    let e2 = e2.saturating_sub(lz as i64);
    let top = e2.saturating_add(63);
    if top > 1023 {
        return (INF_BITS, true);
    }
    // 53 bits with an unbounded exponent: the result for a normal number, and
    // the test for tininess after rounding otherwise.
    let rem = m & 0x7ff;
    let mut mant = m >> 11;
    if rem > 0x400 || (rem == 0x400 && (sticky || mant & 1 == 1)) {
        mant += 1;
    }
    let carried = mant == 1 << 53;
    if top >= -1022 {
        let top = if carried { top + 1 } else { top };
        if top > 1023 {
            return (INF_BITS, true);
        }
        let frac = if carried { 0 } else { mant & ((1 << 52) - 1) };
        return ((((top + 1023) as u64) << 52) | frac, false);
    }
    let tiny = !(carried && top == -1023);
    let shift = (-1074_i64).saturating_sub(e2);
    let (mut sub, rem, half) = if shift >= 65 {
        (0, 1, 2)
    } else if shift == 64 {
        (0, m, 1 << 63)
    } else {
        (m >> shift, m & ((1 << shift) - 1), 1 << (shift - 1))
    };
    let inexact = rem != 0 || sticky;
    if rem > half || (rem == half && (sticky || sub & 1 == 1)) {
        sub += 1;
    }
    (sub, tiny && inexact)
}

/// A hexadecimal mantissa: up to 64 significant bits, the power of two that
/// scales them, whether nonzero bits were dropped, the index after the last
/// digit, and how many digits there were.
fn scan_hex_mantissa(s: &[u8], start: usize) -> (u64, i64, bool, usize, usize) {
    let mut i = start;
    let mut m: u64 = 0;
    let mut e2: i64 = 0;
    let mut sticky = false;
    let mut digits = 0;
    let mut in_fraction = false;
    loop {
        let c = cstr_at(s, i);
        if c == b'.' && !in_fraction {
            in_fraction = true;
            i += 1;
            continue;
        }
        if !is_xdigit(c) {
            break;
        }
        let d = digit_value(c) as u64;
        digits += 1;
        if m >> 60 == 0 {
            m = (m << 4) | d;
            if in_fraction {
                e2 -= 4;
            }
        } else {
            if d != 0 {
                sticky = true;
            }
            if !in_fraction {
                e2 = e2.saturating_add(4);
            }
        }
        i += 1;
    }
    // A lone '.' after the digits belongs to the number, as in "0x1.".
    (m, e2, sticky, i, digits)
}

/// An exponent: `marker`, an optional sign and at least one decimal digit.
/// Returns the saturated value and the index after it, or 0 and `at` if the
/// text at `at` is not an exponent.
fn scan_exponent(s: &[u8], at: usize, marker: u8) -> (i64, usize) {
    if tolower(cstr_at(s, at)) != marker {
        return (0, at);
    }
    let mut i = at + 1;
    let mut negative = false;
    if cstr_at(s, i) == b'-' {
        negative = true;
        i += 1;
    } else if cstr_at(s, i) == b'+' {
        i += 1;
    }
    if !is_digit(cstr_at(s, i)) {
        return (0, at);
    }
    let mut value: i64 = 0;
    while is_digit(cstr_at(s, i)) {
        value = value
            .saturating_mul(10)
            .saturating_add((cstr_at(s, i) - b'0') as i64);
        i += 1;
    }
    // Far outside any exponent a double can have, but safe to add to.
    let value = value.min(1 << 40);
    (if negative { -value } else { value }, i)
}

/// The decimal digits of a mantissa with leading zeros dropped, the power of
/// ten that scales them, and the index after the last digit.
fn scan_decimal_mantissa(s: &[u8], start: usize) -> (Vec<u8>, i64, usize) {
    let mut i = start;
    let mut digits = Vec::new();
    let mut e10: i64 = 0;
    let mut in_fraction = false;
    loop {
        let c = cstr_at(s, i);
        if c == b'.' && !in_fraction {
            in_fraction = true;
            i += 1;
            continue;
        }
        if !is_digit(c) {
            break;
        }
        if c != b'0' || !digits.is_empty() {
            digits.push(c);
        }
        if in_fraction {
            e10 -= 1;
        }
        i += 1;
    }
    (digits, e10, i)
}

/// Correctly rounded value of `digits * 10^e10` and glibc's `ERANGE` verdict.
fn round_decimal(digits: &[u8], e10: i64) -> (u64, bool) {
    if digits.is_empty() {
        return (0, false);
    }
    let mut text = String::with_capacity(digits.len() + 24);
    for &d in digits {
        text.push(char::from(d));
    }
    text.push('e');
    text.push_str(&e10.to_string());
    let bits = text.parse::<f64>().map_or(0, f64::to_bits);
    if bits == INF_BITS {
        return (bits, true);
    }
    if bits == 0 {
        return (bits, true);
    }
    if bits < MIN_NORMAL_BITS {
        let exact = cmp_decimal_binary(digits, e10, bits, -1074) == Ordering::Equal;
        return (bits, !exact);
    }
    if bits == MIN_NORMAL_BITS {
        // Values in [2^-1022 - 2^-1075, 2^-1022 - 2^-1076) round to the
        // smallest normal number here but stay tiny with an unbounded
        // exponent, which is what glibc tests on x86.
        let below = cmp_decimal_binary(digits, e10, (1 << 54) - 1, -1076) == Ordering::Less;
        return (bits, below);
    }
    (bits, false)
}

/// The payload of `nan(...)`, as glibc reads it with `strtoull` in base 0,
/// and whether that call overflowed. glibc does not clear `errno` after
/// it, so an overflowing payload makes the whole `strtod` report `ERANGE`.
fn nan_payload(s: &[u8], start: usize, end: usize) -> (Option<u64>, bool) {
    let seq = &s[start..end];
    let (negative, acc, consumed, overflow) = scan_integer(seq, 0, false);
    if consumed != seq.len() {
        return (None, overflow);
    }
    let value = if overflow {
        u64::MAX
    } else if negative {
        acc.wrapping_neg()
    } else {
        acc
    };
    (Some(value), overflow)
}

/// `strtod` as glibc behaves on the C string `s` in the "C" locale with the
/// default rounding mode.
#[must_use]
pub fn strtod(s: &[u8]) -> Strtod {
    let mut i = 0;
    while isspace(cstr_at(s, i)) {
        i += 1;
    }
    let mut sign = 0;
    if cstr_at(s, i) == b'-' {
        sign = SIGN_BIT;
        i += 1;
    } else if cstr_at(s, i) == b'+' {
        i += 1;
    }
    let c = cstr_at(s, i);
    let starts_number = is_digit(c) || (c == b'.' && is_digit(cstr_at(s, i + 1)));
    if !starts_number {
        if starts_with_ci(s, i, b"inf") {
            let end = if starts_with_ci(s, i + 3, b"inity") {
                i + 8
            } else {
                i + 3
            };
            return Strtod {
                bits: sign | INF_BITS,
                consumed: end,
                erange: false,
            };
        }
        if starts_with_ci(s, i, b"nan") {
            let mut end = i + 3;
            let mut bits = QNAN_BITS;
            let mut erange = false;
            if cstr_at(s, end) == b'(' {
                let mut j = end + 1;
                while digit_value(cstr_at(s, j)) < 36 || cstr_at(s, j) == b'_' {
                    j += 1;
                }
                if cstr_at(s, j) == b')' {
                    let (payload, overflow) = nan_payload(s, end + 1, j);
                    if let Some(payload) = payload {
                        bits |= payload & ((1 << 51) - 1);
                    }
                    erange = overflow;
                    end = j + 1;
                }
            }
            return Strtod {
                bits: sign | bits,
                consumed: end,
                erange,
            };
        }
        return Strtod {
            bits: 0,
            consumed: 0,
            erange: false,
        };
    }
    if c == b'0' && tolower(cstr_at(s, i + 1)) == b'x' {
        let (m, e2, sticky, end, digits) = scan_hex_mantissa(s, i + 2);
        if digits == 0 {
            // "0x" with no hexadecimal digit is the number 0, ending before
            // the 'x'.
            return Strtod {
                bits: sign,
                consumed: i + 1,
                erange: false,
            };
        }
        let (exp, end) = scan_exponent(s, end, b'p');
        let (bits, erange) = round_binary(m, e2.saturating_add(exp), sticky);
        return Strtod {
            bits: sign | bits,
            consumed: end,
            erange,
        };
    }
    let (digits, e10, end) = scan_decimal_mantissa(s, i);
    let (exp, end) = scan_exponent(s, end, b'e');
    let (bits, erange) = round_decimal(&digits, e10.saturating_add(exp));
    Strtod {
        bits: sign | bits,
        consumed: end,
        erange,
    }
}

/// What `printf("%f")` writes for the `double` with these bits.
#[must_use]
pub fn format_f(bits: u64) -> Vec<u8> {
    let negative = bits & SIGN_BIT != 0;
    let magnitude = bits & !SIGN_BIT;
    let body = if magnitude == INF_BITS {
        "inf".to_owned()
    } else if magnitude > INF_BITS {
        "nan".to_owned()
    } else {
        format!("{:.6}", f64::from_bits(magnitude))
    };
    let mut out = Vec::with_capacity(body.len() + 1);
    if negative {
        out.push(b'-');
    }
    out.extend_from_slice(body.as_bytes());
    out
}

/// What `printf("%" PRId64)` writes.
#[must_use]
pub fn format_d(v: i64) -> Vec<u8> {
    let mut digits = [0_u8; 20];
    let mut n = 0;
    let mut rest = v.unsigned_abs();
    loop {
        digits[n] = b'0' + (rest % 10) as u8;
        rest /= 10;
        n += 1;
        if rest == 0 {
            break;
        }
    }
    let mut out = Vec::with_capacity(n + 1);
    if v < 0 {
        out.push(b'-');
    }
    while n > 0 {
        n -= 1;
        out.push(digits[n]);
    }
    out
}
