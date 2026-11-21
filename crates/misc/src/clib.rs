//! Stand-ins for the C library routines that `misc/` calls.
//!
//! A C string is a `&[u8]` whose content ends at its first NUL byte, or at the
//! end of the slice if it has none. [`cstr_at`] reads the way C reads a
//! NUL-terminated string: the byte at the terminator and beyond is 0.
//!
//! `strtoll` follows glibc 2.39 as the pinned mpv build links it, which means
//! the C23 entry point: it accepts a `0b` prefix. The routines that need
//! floating point or formatting are in [`crate::cfloat`].

/// The byte at index `i` of a C string, 0 at and after its end.
#[must_use]
pub fn cstr_at(s: &[u8], i: usize) -> u8 {
    if i < s.len() { s[i] } else { 0 }
}

/// `strlen`.
#[must_use]
pub fn strlen(s: &[u8]) -> usize {
    let mut i = 0;
    while i < s.len() && s[i] != 0 {
        i += 1;
    }
    i
}

/// The content of a C string, without its terminator.
#[must_use]
pub fn cstr(s: &[u8]) -> &[u8] {
    &s[..strlen(s)]
}

/// `memcmp` over the first `n` bytes of both slices.
///
/// C defines only the sign of the result. glibc's value depends on the CPU
/// and on `n`, so the port returns the difference of the first differing
/// bytes and callers may rely on the sign alone.
#[must_use]
pub fn memcmp(a: &[u8], b: &[u8], n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        if a[i] != b[i] {
            return a[i] as i32 - b[i] as i32;
        }
        i += 1;
    }
    0
}

/// `tolower` in the "C" locale.
#[must_use]
pub fn tolower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' { c + 32 } else { c }
}

/// `strncasecmp` in the "C" locale. Like the C routine it stops at a NUL
/// byte, so bytes after an embedded NUL are never compared. Only the sign of
/// the result is defined.
#[must_use]
pub fn strncasecmp(a: &[u8], b: &[u8], n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let c1 = tolower(cstr_at(a, i));
        let c2 = tolower(cstr_at(b, i));
        if c1 != c2 {
            return c1 as i32 - c2 as i32;
        }
        if c1 == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

/// `strcasecmp` in the "C" locale.
#[must_use]
pub fn strcasecmp(a: &[u8], b: &[u8]) -> i32 {
    strncasecmp(a, b, usize::MAX)
}

/// `strncmp`.
#[must_use]
pub fn strncmp(a: &[u8], b: &[u8], n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let c1 = cstr_at(a, i);
        let c2 = cstr_at(b, i);
        if c1 != c2 {
            return c1 as i32 - c2 as i32;
        }
        if c1 == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

/// `strcmp`.
#[must_use]
pub fn strcmp(a: &[u8], b: &[u8]) -> i32 {
    strncmp(a, b, usize::MAX)
}

/// `strchr`: the index of the first `c` in the C string `s`.
///
/// The terminator counts as part of the string, so searching for 0 always
/// succeeds. `misc/` leans on this: a NUL byte is a member of every character
/// set passed to `bstrspn`, `bstrcspn` and `bstr_split`.
#[must_use]
pub fn strchr(s: &[u8], c: u8) -> Option<usize> {
    let mut i = 0;
    loop {
        let cur = cstr_at(s, i);
        if cur == c {
            return Some(i);
        }
        if cur == 0 {
            return None;
        }
        i += 1;
    }
}

/// `memchr` over the first `n` bytes.
#[must_use]
pub fn memchr(s: &[u8], c: u8, n: usize) -> Option<usize> {
    let mut i = 0;
    while i < n {
        if s[i] == c {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// `memrchr` over the first `n` bytes.
#[must_use]
pub fn memrchr(s: &[u8], c: u8, n: usize) -> Option<usize> {
    let mut i = n;
    while i > 0 {
        i -= 1;
        if s[i] == c {
            return Some(i);
        }
    }
    None
}

/// `talloc_strndup`: at most `n` bytes of `s`, cut at the first NUL.
#[must_use]
pub fn strndup(s: &[u8], n: usize) -> Vec<u8> {
    let mut len = 0;
    while len < n && len < s.len() && s[len] != 0 {
        len += 1;
    }
    s[..len].to_vec()
}

/// `isspace` in the "C" locale.
#[must_use]
pub fn isspace(c: u8) -> bool {
    c == b' ' || (c >= 9 && c <= 13)
}

/// The value of `c` as a digit in bases up to 36, or 36 if it is none.
pub(crate) fn digit_value(c: u8) -> u32 {
    if c >= b'0' && c <= b'9' {
        (c - b'0') as u32
    } else if c >= b'a' && c <= b'z' {
        (c - b'a') as u32 + 10
    } else if c >= b'A' && c <= b'Z' {
        (c - b'A') as u32 + 10
    } else {
        36
    }
}

/// What `strtoll` and `strtoull` report: the value, how many bytes of the
/// string the number took (`endptr - nptr`), and whether `errno` was set to
/// `ERANGE`.
#[cfg_attr(not(aeneas), derive(Debug, Clone, Copy, PartialEq, Eq))]
pub struct Strtoll {
    pub value: i64,
    pub consumed: usize,
    pub erange: bool,
}

/// The digits of an integer after sign and prefix: the magnitude, the index
/// after the last digit, and whether the magnitude passed `u64::MAX`.
fn scan_digits(s: &[u8], start: usize, base: u32) -> (u64, usize, bool) {
    let cutoff = u64::MAX / base as u64;
    let cutlim = u64::MAX % base as u64;
    let mut i = start;
    let mut acc: u64 = 0;
    let mut overflow = false;
    loop {
        let d = digit_value(cstr_at(s, i));
        if d >= base {
            break;
        }
        if acc > cutoff || (acc == cutoff && d as u64 > cutlim) {
            overflow = true;
        } else {
            acc = acc * base as u64 + d as u64;
        }
        i += 1;
    }
    (acc, i, overflow)
}

/// The shared front of `strtoll` and `strtoull`: whitespace, sign, prefix and
/// digits. Returns (negative, magnitude, consumed, overflowed `u64`).
pub(crate) fn scan_integer(s: &[u8], base: i32, binary_prefix: bool) -> (bool, u64, usize, bool) {
    let mut i = 0;
    while isspace(cstr_at(s, i)) {
        i += 1;
    }
    let mut negative = false;
    if cstr_at(s, i) == b'-' {
        negative = true;
        i += 1;
    } else if cstr_at(s, i) == b'+' {
        i += 1;
    }
    let mut base = base;
    let mut prefixed = false;
    if cstr_at(s, i) == b'0' {
        let next = cstr_at(s, i + 1);
        if (base == 0 || base == 16) && (next == b'x' || next == b'X') {
            i += 2;
            base = 16;
            prefixed = true;
        } else if binary_prefix && (base == 0 || base == 2) && (next == b'b' || next == b'B') {
            i += 2;
            base = 2;
            prefixed = true;
        } else if base == 0 {
            base = 8;
        }
    } else if base == 0 {
        base = 10;
    }
    let (acc, end, overflow) = scan_digits(s, i, base as u32);
    if end == i {
        // "0x" or "0b" without a digit after it is the number 0, ending
        // before the letter.
        let consumed = if prefixed { i - 1 } else { 0 };
        return (negative, 0, consumed, false);
    }
    (negative, acc, end, overflow)
}

/// `strtoll` as glibc's `__isoc23_strtoll` behaves on the C string `s`.
///
/// For a base outside 0 and 2..=36 glibc returns 0 and leaves `*endptr`
/// unwritten. The port reports 0 bytes consumed for that case.
#[must_use]
pub fn strtoll(s: &[u8], base: i32) -> Strtoll {
    if base < 0 || base == 1 || base > 36 {
        return Strtoll {
            value: 0,
            consumed: 0,
            erange: false,
        };
    }
    let (negative, acc, consumed, overflow) = scan_integer(s, base, true);
    let limit: u64 = if negative { 1 << 63 } else { (1 << 63) - 1 };
    if overflow || acc > limit {
        let value = if negative { i64::MIN } else { i64::MAX };
        return Strtoll {
            value,
            consumed,
            erange: true,
        };
    }
    let value = if negative {
        0_i64.wrapping_sub(acc as i64)
    } else {
        acc as i64
    };
    Strtoll {
        value,
        consumed,
        erange: false,
    }
}

/// What [`crate::cfloat::strtod`] reports: the bit pattern of the `double`, how many bytes
/// the number took, and whether `errno` was set to `ERANGE`.
#[cfg_attr(not(aeneas), derive(Debug, Clone, Copy, PartialEq, Eq))]
pub struct Strtod {
    pub bits: u64,
    pub consumed: usize,
    pub erange: bool,
}

const SIGN_BIT: u64 = 1 << 63;
const INF_BITS: u64 = 0x7ff0_0000_0000_0000;

/// Whether the `double` with these bits is neither infinite nor NaN.
#[must_use]
pub fn isfinite(bits: u64) -> bool {
    bits & INF_BITS != INF_BITS
}

/// `==` on two `double`s given by their bits: NaN equals nothing, and the two
/// zeros are equal.
#[must_use]
pub fn double_eq(a: u64, b: u64) -> bool {
    let a_nan = a & !SIGN_BIT > INF_BITS;
    let b_nan = b & !SIGN_BIT > INF_BITS;
    if a_nan || b_nan {
        return false;
    }
    a == b || (a | b) & !SIGN_BIT == 0
}
