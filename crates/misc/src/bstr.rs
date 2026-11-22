//! Port of `misc/bstr.c` and the inline helpers of `misc/bstr.h`.
//!
//! A `bstr` view is a `&[u8]`. The C keeps positions in `int` while lengths
//! are `size_t`, and its header says callers must keep lengths within `int`.
//! The port keeps the `int` variables as `i32` and writes each implicit C
//! conversion as an `as` cast, so both sides agree wherever the C is defined.

use crate::cfloat::strtod;
use crate::clib::{cstr, memchr, memcmp, memrchr, strchr, strncasecmp, strndup, strtoll};
use crate::common::{mp_append_utf8_bstr, mp_log2};
use crate::cprintf::{self, CArg, PrintfError};
use crate::cscanf::{self, ScanArg, ScanfError};
use crate::ctype::{mp_isspace, mp_tolower};

pub const BSTR_DECODE_OUT_OF_RANGE: i32 = -1;
pub const BSTR_DECODE_TRUNCATED_SEQUENCE: i32 = -2;
pub const BSTR_DECODE_OVERLONG_ENCODING: i32 = -3;

/// `WHITESPACE`.
pub const WHITESPACE: &[u8] = b" \x0c\n\r\t\x0b";

/// `bstrdup0`: a C string copy, so it ends at the first NUL in `s`.
#[must_use]
pub fn bstrdup0(s: &[u8]) -> Vec<u8> {
    strndup(s, s.len())
}

/// `bstrto0`. With no NULL views in the port this is `bstrdup0`.
#[must_use]
pub fn bstrto0(s: &[u8]) -> Vec<u8> {
    bstrdup0(s)
}

/// `bstrdup`.
#[must_use]
pub fn bstrdup(s: &[u8]) -> Vec<u8> {
    s.to_vec()
}

/// `bstr0`: the content of a C string as a view.
#[must_use]
pub fn bstr0(s: &[u8]) -> &[u8] {
    cstr(s)
}

/// `bstrcmp`. Only the sign of a nonzero result is defined; see
/// [`memcmp`].
#[must_use]
pub fn bstrcmp(str1: &[u8], str2: &[u8]) -> i32 {
    let mut ret = 0;
    if !str1.is_empty() && !str2.is_empty() {
        ret = memcmp(str1, str2, str1.len().min(str2.len()));
    }

    if ret == 0 {
        if str1.len() == str2.len() {
            return 0;
        } else if str1.len() > str2.len() {
            return 1;
        }
        return -1;
    }
    ret
}

/// `bstrcasecmp`. `strncasecmp` stops at a NUL byte, so two strings that
/// agree up to an embedded NUL compare by length alone.
#[must_use]
pub fn bstrcasecmp(str1: &[u8], str2: &[u8]) -> i32 {
    let mut ret = 0;
    if !str1.is_empty() && !str2.is_empty() {
        ret = strncasecmp(str1, str2, str1.len().min(str2.len()));
    }

    if ret == 0 {
        if str1.len() == str2.len() {
            return 0;
        } else if str1.len() > str2.len() {
            return 1;
        }
        return -1;
    }
    ret
}

/// `bstrchr`. `memchr` converts `c` to `unsigned char`.
#[must_use]
pub fn bstrchr(s: &[u8], c: i32) -> i32 {
    if s.is_empty() {
        return -1;
    }
    match memchr(s, c as u8, s.len()) {
        Some(pos) => pos as i32,
        None => -1,
    }
}

/// `bstrrchr`, the `memrchr` variant this build compiles.
#[must_use]
pub fn bstrrchr(s: &[u8], c: i32) -> i32 {
    if s.is_empty() {
        return -1;
    }
    match memrchr(s, c as u8, s.len()) {
        Some(pos) => pos as i32,
        None => -1,
    }
}

/// `bstrcspn`. `reject` is a C string; a NUL byte in `s` always stops the
/// span because `strchr` finds the terminator.
#[must_use]
pub fn bstrcspn(s: &[u8], reject: &[u8]) -> i32 {
    let mut i: i32 = 0;
    while (i as usize) < s.len() {
        if strchr(reject, s[i as usize]).is_some() {
            break;
        }
        i += 1;
    }
    i
}

/// `bstrspn`. `accept` is a C string; a NUL byte in `s` is always accepted.
#[must_use]
pub fn bstrspn(s: &[u8], accept: &[u8]) -> i32 {
    let mut i: i32 = 0;
    while (i as usize) < s.len() {
        if strchr(accept, s[i as usize]).is_none() {
            break;
        }
        i += 1;
    }
    i
}

/// `bstr_find`. An empty needle is found at 0, except in an empty haystack.
#[must_use]
pub fn bstr_find(haystack: &[u8], needle: &[u8]) -> i32 {
    let mut i: i32 = 0;
    while (i as usize) < haystack.len() {
        if bstr_startswith(bstr_splice(haystack, i, haystack.len() as i32), needle) {
            return i;
        }
        i += 1;
    }
    -1
}

/// `bstr_lstrip`.
#[must_use]
pub fn bstr_lstrip(s: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < s.len() && mp_isspace(s[i]) {
        i += 1;
    }
    &s[i..]
}

/// `bstr_strip`.
#[must_use]
pub fn bstr_strip(s: &[u8]) -> &[u8] {
    let s = bstr_lstrip(s);
    let mut len = s.len();
    while len != 0 && mp_isspace(s[len - 1]) {
        len -= 1;
    }
    &s[..len]
}

/// `bstr_split`: skips leading bytes in the C string `sep`, then returns the
/// span up to the next one and, as `rest`, everything from that byte on.
#[must_use]
pub fn bstr_split<'a>(s: &'a [u8], sep: &[u8]) -> (&'a [u8], &'a [u8]) {
    let mut start: i32 = 0;
    while (start as usize) < s.len() {
        if strchr(sep, s[start as usize]).is_none() {
            break;
        }
        start += 1;
    }
    let s = bstr_cut(s, start);
    let end = bstrcspn(s, sep);
    let rest = bstr_cut(s, end);
    (bstr_splice(s, 0, end), rest)
}

/// `bstr_split_tok`: returns (found, left, right). `tok` is a C string.
#[must_use]
pub fn bstr_split_tok<'a>(s: &'a [u8], tok: &[u8]) -> (bool, &'a [u8], &'a [u8]) {
    let bsep = bstr0(tok);
    let mut pos = bstr_find(s, bsep);
    if pos < 0 {
        pos = s.len() as i32;
    }
    let out_left = bstr_splice(s, 0, pos);
    let out_right = bstr_cut(s, (pos as usize).wrapping_add(bsep.len()) as i32);
    (pos as usize != s.len(), out_left, out_right)
}

/// `bstr_splice`.
///
/// `MPMIN(end, str.len)` compares in `size_t`, so an `end` that is still
/// negative after adding the length selects the whole length, not 0.
#[must_use]
pub fn bstr_splice(s: &[u8], start: i32, end: i32) -> &[u8] {
    if s.is_empty() {
        return s;
    }
    let mut start = start;
    let mut end = end;

    if start < 0 {
        start = (start as usize).wrapping_add(s.len()) as i32;
    }
    if end < 0 {
        end = (end as usize).wrapping_add(s.len()) as i32;
    }
    end = if end as usize > s.len() {
        s.len() as i32
    } else {
        end
    };
    start = if start > 0 { start } else { 0 };
    end = if end > start { end } else { start };
    if start as usize > s.len() {
        // The C forms `str.start + start` beyond the buffer here, with
        // length 0. The port returns the empty view at the end instead.
        return &s[s.len()..];
    }
    &s[start as usize..end as usize]
}

/// `bstrtoll`: returns the value and the rest of the string after it.
///
/// Only the first 50 bytes after leading whitespace are read, and a NUL byte
/// ends the number. For a base that `strtoll` rejects the C reads an
/// uninitialized `endptr`; the port returns 0 and the stripped string.
#[must_use]
pub fn bstrtoll(s: &[u8], base: i32) -> (i64, &[u8]) {
    let s = bstr_lstrip(s);
    let len = s.len().min(50);
    let r = strtoll(&s[..len], base);
    (r.value, bstr_cut(s, r.consumed as i32))
}

/// `bstrtod`: returns the value and the rest of the string after it. Only
/// the first 100 bytes after leading whitespace are read.
#[must_use]
pub fn bstrtod(s: &[u8]) -> (f64, &[u8]) {
    let s = bstr_lstrip(s);
    let len = s.len().min(100);
    let r = strtod(&s[..len]);
    (f64::from_bits(r.bits), bstr_cut(s, r.consumed as i32))
}

/// `bstr_splitchar`: returns the text up to and including the first `c`, and
/// the rest after it. Despite mpv's comment, the delimiter stays in the
/// returned part.
#[must_use]
pub fn bstr_splitchar(s: &[u8], c: u8) -> (&[u8], &[u8]) {
    // The C passes a `char`, which sign-extends; `memchr` then keeps the low
    // byte, so the round trip is the identity on the byte.
    let mut pos = bstrchr(s, c as i8 as i32);
    if pos < 0 {
        pos = s.len() as i32;
    }
    let rest = bstr_cut(s, pos + 1);
    (bstr_splice(s, 0, pos + 1), rest)
}

/// `bstr_getline`.
#[must_use]
pub fn bstr_getline(s: &[u8]) -> (&[u8], &[u8]) {
    bstr_splitchar(s, b'\n')
}

/// `bstr_strip_linebreaks`.
#[must_use]
pub fn bstr_strip_linebreaks(s: &[u8]) -> &[u8] {
    if bstr_endswith0(s, b"\r\n") {
        bstr_splice(s, 0, s.len().wrapping_sub(2) as i32)
    } else if bstr_endswith0(s, b"\n") {
        bstr_splice(s, 0, s.len().wrapping_sub(1) as i32)
    } else {
        s
    }
}

/// `bstr_eatstart`: returns whether `s` starts with `prefix`, and `s` without
/// it if so, else `s` itself.
#[must_use]
pub fn bstr_eatstart<'a>(s: &'a [u8], prefix: &[u8]) -> (bool, &'a [u8]) {
    if !bstr_startswith(s, prefix) {
        return (false, s);
    }
    (true, bstr_cut(s, prefix.len() as i32))
}

/// `bstr_eatend`.
#[must_use]
pub fn bstr_eatend<'a>(s: &'a [u8], prefix: &[u8]) -> (bool, &'a [u8]) {
    if !bstr_endswith(s, prefix) {
        return (false, s);
    }
    (true, &s[..s.len() - prefix.len()])
}

/// `bstr_lower`.
pub fn bstr_lower(s: &mut [u8]) {
    let mut i: i32 = 0;
    while (i as usize) < s.len() {
        s[i as usize] = mp_tolower(s[i as usize]);
        i += 1;
    }
}

/// `bstr_sscanf`: `sscanf` on a NUL-terminated copy of `s`, so the input
/// ends at the first NUL byte of `s`. The pointer arguments are `args`.
///
/// # Errors
///
/// The port's refusals of what it does not model; see
/// [`cscanf`](crate::cscanf).
pub fn bstr_sscanf(s: &[u8], fmt: &[u8], args: &mut [ScanArg<'_>]) -> Result<i32, ScanfError> {
    cscanf::vsscanf(s, fmt, args)
}

/// `bstr_parse_utf8_code_length`.
#[must_use]
pub fn bstr_parse_utf8_code_length(b: u8) -> i32 {
    if b < 128 {
        return 1;
    }
    let bytes = 7 - mp_log2((b ^ 255) as u32) as i32;
    if bytes >= 2 && bytes <= 4 { bytes } else { -1 }
}

/// `bstr_decode_partial_utf8`: returns the code point or a negative
/// `BSTR_DECODE_*` code, and the string after the bytes it consumed. Bytes
/// are consumed on errors too.
#[must_use]
pub fn bstr_decode_partial_utf8(s: &[u8]) -> (i32, &[u8]) {
    if s.is_empty() {
        return (-1, s);
    }
    let mut codepoint = s[0] as u32;
    let mut pos = 1;
    if codepoint >= 128 {
        let bytes = bstr_parse_utf8_code_length(codepoint as u8);
        if bytes < 1 {
            return (BSTR_DECODE_OUT_OF_RANGE, &s[pos..]);
        }
        codepoint &= 127 >> bytes;
        let mut n = 1;
        while n < bytes {
            if pos == s.len() {
                return (BSTR_DECODE_TRUNCATED_SEQUENCE, &s[pos..]);
            }
            let tmp = s[pos] as u32;
            if (tmp & 0xC0) != 0x80 {
                return (BSTR_DECODE_TRUNCATED_SEQUENCE, &s[pos..]);
            }
            codepoint = (codepoint << 6) | (tmp & !0xC0);
            pos += 1;
            n += 1;
        }
        if codepoint > 0x0010_FFFF || (codepoint >= 0xD800 && codepoint <= 0xDFFF) {
            return (BSTR_DECODE_OUT_OF_RANGE, &s[pos..]);
        }

        let min: u32 = if bytes == 2 {
            0x80
        } else {
            1 << (5 * bytes - 4)
        };
        if codepoint < min {
            return (BSTR_DECODE_OVERLONG_ENCODING, &s[pos..]);
        }
    }
    (codepoint as i32, &s[pos..])
}

/// `bstr_decode_utf8`: the code point at the start of `s` and the string
/// after it, or a negative `BSTR_DECODE_*` code and `s` itself.
///
/// The C writes the rest through `out_next` and leaves it alone on an
/// error. Its callers either pass their own string there or do not read it
/// after an error, which is the same as getting `s` back.
#[must_use]
pub fn bstr_decode_utf8(s: &[u8]) -> (i32, &[u8]) {
    let (res, next) = bstr_decode_partial_utf8(s);
    if res >= 0 {
        return (res, next);
    }
    (res, s)
}

/// `bstr_split_utf8`: the bytes of the first code point and the string
/// after it. On an error the first part is empty and the second is `s`
/// itself, where the C leaves `out_next` alone.
#[must_use]
pub fn bstr_split_utf8(s: &[u8]) -> (&[u8], &[u8]) {
    let (code, rest) = bstr_decode_utf8(s);
    if code < 0 {
        return (&s[..0], s);
    }
    (bstr_splice(s, 0, (s.len() - rest.len()) as i32), rest)
}

/// The inner loop of `bstr_validate_utf8`: 0 to go on to `return -8`, else
/// the value to return.
fn validate_cutoff(s: &[u8], bytes: i32) -> i32 {
    let mut n = 1;
    while n < bytes {
        if n as usize >= s.len() {
            return 0_usize.wrapping_sub((bytes as usize).wrapping_sub(s.len())) as i32;
        }
        let tmp = s[n as usize] as i32;
        if (tmp & 0xC0) != 0x80 {
            break;
        }
        n += 1;
    }
    0
}

/// `bstr_validate_utf8`.
#[must_use]
pub fn bstr_validate_utf8(s: &[u8]) -> i32 {
    let mut s = s;
    while !s.is_empty() {
        let (res, next) = bstr_decode_utf8(s);
        s = next;
        if res < 0 {
            let codepoint = s[0] as u32;
            let bytes = bstr_parse_utf8_code_length(codepoint as u8);
            if bytes > 1 && s.len() < 6 {
                let cutoff = validate_cutoff(s, bytes);
                if cutoff != 0 {
                    return cutoff;
                }
            }
            return -8;
        }
    }
    0
}

/// `bstr_sanitize_utf8_latin1`: `None` where the C returns its input because
/// it was valid UTF-8, else the repaired copy.
#[must_use]
pub fn bstr_sanitize_utf8_latin1(s: &[u8]) -> Option<Vec<u8>> {
    let mut new = Vec::new();
    let mut replaced = false;
    let mut left = s;
    let mut first_ok = 0;
    while !left.is_empty() {
        let (r, next) = bstr_decode_utf8(left);
        left = next;
        if r < 0 {
            let pos = s.len() - left.len();
            bstr_xappend(&mut new, &s[first_ok..pos]);
            mp_append_utf8_bstr(&mut new, left[0] as u32);
            replaced = true;
            left = &left[1..];
            first_ok = pos + 1;
        }
    }
    if !replaced {
        return None;
    }
    if first_ok != s.len() {
        bstr_xappend(&mut new, &s[first_ok..]);
    }
    Some(new)
}

/// `resize_append`: makes room for `append_min` more bytes. The capacity of
/// the `Vec` stands for `talloc_get_size`. Nothing here is observable except
/// the abort on an impossible size.
fn resize_append(s: &mut Vec<u8>, append_min: usize) {
    let mut append_min = append_min;
    let size = s.capacity();
    assert!(s.len() <= size);
    if append_min > size - s.len() {
        if append_min < size {
            append_min = size;
        }
        // The C calls abort() here.
        assert!(size < usize::MAX / 2 && append_min < usize::MAX / 2);
        s.reserve_exact(size + append_min - s.len());
    }
}

/// `bstr_xappend`. The C also keeps a NUL after the data, which a `Vec` does
/// not hold.
pub fn bstr_xappend(s: &mut Vec<u8>, append: &[u8]) {
    if append.is_empty() {
        return;
    }
    resize_append(s, append.len() + 1);
    s.extend_from_slice(append);
}

/// `bstr_xappend0`.
pub fn bstr_xappend0(s: &mut Vec<u8>, append: &[u8]) {
    bstr_xappend(s, bstr0(append));
}

/// `bstr_xappend_asprintf`: appends what `printf(fmt, args...)` writes and
/// returns its length. The variadic arguments are `args`.
///
/// # Errors
///
/// As [`bstr_xappend_vasprintf`].
pub fn bstr_xappend_asprintf(
    s: &mut Vec<u8>,
    fmt: &[u8],
    args: &[CArg<'_>],
) -> Result<i32, PrintfError> {
    bstr_xappend_vasprintf(s, fmt, args)
}

/// `bstr_xappend_vasprintf`. On a formatting error the C returns the
/// negative result of `vsnprintf` and leaves the length of `s` as it was;
/// the port returns the error and leaves `s` as it was.
///
/// # Errors
///
/// [`PrintfError::Failed`] where the C returns -1, and the port's
/// refusals of what it does not model; see [`cprintf`](crate::cprintf).
pub fn bstr_xappend_vasprintf(
    s: &mut Vec<u8>,
    fmt: &[u8],
    args: &[CArg<'_>],
) -> Result<i32, PrintfError> {
    // The C formats into the free space first and again after growing the
    // buffer if it did not fit; both produce the same bytes.
    let text = cprintf::vsnprintf(fmt, args)?;
    let size = text.len();
    let avail = s.capacity() - s.len();
    if avail < 1 || size + 1 > avail {
        resize_append(s, size + 1);
    }
    s.extend_from_slice(&text);
    Ok(size as i32)
}

/// `bstr_case_startswith`.
#[must_use]
pub fn bstr_case_startswith(s: &[u8], prefix: &[u8]) -> bool {
    let start = bstr_splice(s, 0, prefix.len() as i32);
    start.len() == prefix.len() && bstrcasecmp(start, prefix) == 0
}

/// `bstr_case_endswith`. With an empty suffix the C cuts at `-0`, which keeps
/// the whole string, so only an empty string ends with the empty suffix.
#[must_use]
pub fn bstr_case_endswith(s: &[u8], suffix: &[u8]) -> bool {
    let end = bstr_cut(s, 0_usize.wrapping_sub(suffix.len()) as i32);
    end.len() == suffix.len() && bstrcasecmp(end, suffix) == 0
}

/// `h_to_i`.
#[must_use]
pub fn h_to_i(c: u8) -> i32 {
    if c >= b'0' && c <= b'9' {
        return (c - b'0') as i32;
    }
    if c >= b'a' && c <= b'f' {
        return (c - b'a') as i32 + 10;
    }
    if c >= b'A' && c <= b'F' {
        return (c - b'A') as i32 + 10;
    }

    -1
}

/// `bstr_decode_hex`: `None` for an invalid digit. A trailing odd digit is
/// ignored, and whitespace is not skipped whatever mpv's header says.
#[must_use]
pub fn bstr_decode_hex(hex: &[u8]) -> Option<Vec<u8>> {
    let mut hex = hex;
    let mut arr = Vec::with_capacity(hex.len() / 2);

    while hex.len() >= 2 {
        let a = h_to_i(hex[0]);
        let b = h_to_i(hex[1]);
        hex = bstr_splice(hex, 2, hex.len() as i32);

        if a < 0 || b < 0 {
            return None;
        }

        arr.push(((a << 4) | b) as u8);
    }

    Some(arr)
}

/// `bstr_find_in_list0`. A NULL-terminated `char **` is a slice of C strings.
#[must_use]
pub fn bstr_find_in_list0(s: &[u8], strings: &[Vec<u8>], case_sensitive: bool) -> i32 {
    let mut n: i32 = 0;
    while (n as usize) < strings.len() {
        let order = if case_sensitive {
            bstrcmp0(s, &strings[n as usize])
        } else {
            bstrcasecmp0(s, &strings[n as usize])
        };
        if order == 0 {
            return n;
        }
        n += 1;
    }

    -1
}

/// `bstr_in_list0`.
#[must_use]
pub fn bstr_in_list0(s: &[u8], strings: &[Vec<u8>]) -> bool {
    bstr_find_in_list0(s, strings, false) != -1
}

/// `bstr_cut`.
#[must_use]
pub fn bstr_cut(s: &[u8], n: i32) -> &[u8] {
    let mut n = n;
    if n < 0 {
        n = (n as usize).wrapping_add(s.len()) as i32;
        if n < 0 {
            n = 0;
        }
    }
    if n as usize > s.len() {
        n = s.len() as i32;
    }
    &s[n as usize..]
}

/// `bstr_startswith`.
#[must_use]
pub fn bstr_startswith(s: &[u8], prefix: &[u8]) -> bool {
    if s.len() < prefix.len() {
        return false;
    }
    memcmp(s, prefix, prefix.len()) == 0
}

/// `bstr_startswith0`.
#[must_use]
pub fn bstr_startswith0(s: &[u8], prefix: &[u8]) -> bool {
    bstr_startswith(s, bstr0(prefix))
}

/// `bstr_endswith`.
#[must_use]
pub fn bstr_endswith(s: &[u8], suffix: &[u8]) -> bool {
    if s.len() < suffix.len() {
        return false;
    }
    memcmp(&s[s.len() - suffix.len()..], suffix, suffix.len()) == 0
}

/// `bstr_endswith0`.
#[must_use]
pub fn bstr_endswith0(s: &[u8], suffix: &[u8]) -> bool {
    bstr_endswith(s, bstr0(suffix))
}

/// `bstrcmp0`.
#[must_use]
pub fn bstrcmp0(str1: &[u8], str2: &[u8]) -> i32 {
    bstrcmp(str1, bstr0(str2))
}

/// `bstr_equals`. The C also accepts equal start pointers, which says no
/// more than equal content once the lengths match.
#[must_use]
pub fn bstr_equals(str1: &[u8], str2: &[u8]) -> bool {
    if str1.len() != str2.len() {
        return false;
    }

    bstrcmp(str1, str2) == 0
}

/// `bstr_equals0`.
#[must_use]
pub fn bstr_equals0(str1: &[u8], str2: &[u8]) -> bool {
    bstr_equals(str1, bstr0(str2))
}

/// `bstrcasecmp0`.
#[must_use]
pub fn bstrcasecmp0(str1: &[u8], str2: &[u8]) -> i32 {
    bstrcasecmp(str1, bstr0(str2))
}

/// `bstr_find0`.
#[must_use]
pub fn bstr_find0(haystack: &[u8], needle: &[u8]) -> i32 {
    bstr_find(haystack, bstr0(needle))
}

/// `bstr_eatstart0`.
#[must_use]
pub fn bstr_eatstart0<'a>(s: &'a [u8], prefix: &[u8]) -> (bool, &'a [u8]) {
    bstr_eatstart(s, bstr0(prefix))
}

/// `bstr_eatend0`.
#[must_use]
pub fn bstr_eatend0<'a>(s: &'a [u8], prefix: &[u8]) -> (bool, &'a [u8]) {
    bstr_eatend(s, bstr0(prefix))
}
