//! Ports of the functions in mpv's `common/common.c` that `misc/` calls.

use crate::bstr::{bstr_cut, bstr_splice, bstr_xappend, bstrtoll};

/// `mp_log2`: the index of the highest set bit, 0 for 0.
#[must_use]
pub fn mp_log2(v: u32) -> u32 {
    if v == 0 { 0 } else { 31 - v.leading_zeros() }
}

/// `mp_round_next_power_of_2`. Returns 0 above 2^31.
#[must_use]
pub fn mp_round_next_power_of_2(v: u32) -> u32 {
    if v == 0 {
        return 1;
    }
    if v & (v - 1) == 0 {
        return v;
    }
    let l = mp_log2(v) + 1;
    if l == 32 { 0 } else { 1 << l }
}

/// `mp_append_utf8_bstr`.
///
/// This is `PUT_UTF8` of libavutil, which also encodes values that are not
/// code points, in up to 6 bytes. For a value of 2^31 or more the macro
/// shifts a 32-bit value by 36, which C leaves undefined; the port's shift
/// overflows there, which panics in a debug build.
pub fn mp_append_utf8_bstr(buf: &mut Vec<u8>, codepoint: u32) {
    let mut data = [0_u8; 8];
    let mut n = 0;
    if codepoint < 0x80 {
        data[n] = codepoint as u8;
        n += 1;
    } else {
        let bytes = (mp_log2(codepoint | 1) + 4) / 5;
        let mut shift = (bytes - 1) * 6;
        data[n] = ((256 - (256 >> bytes)) | (codepoint >> shift)) as u8;
        n += 1;
        while shift >= 6 {
            shift -= 6;
            data[n] = (0x80 | ((codepoint >> shift) & 0x3f)) as u8;
            n += 1;
        }
    }
    bstr_xappend(buf, &data[..n]);
}

/// `mp_parse_escape`: one escape sequence, without its backslash, from the
/// front of `code`. Returns whether it was valid and the rest of `code`.
pub fn mp_parse_escape<'a>(dst: &mut Vec<u8>, code: &'a [u8]) -> (bool, &'a [u8]) {
    if code.is_empty() {
        return (false, code);
    }
    let replace: u8 = match code[0] {
        b'"' => b'"',
        b'\\' => b'\\',
        b'/' => b'/',
        b'b' => 0x08,
        b'f' => 0x0c,
        b'n' => b'\n',
        b'r' => b'\r',
        b't' => b'\t',
        b'e' => 0x1b,
        b'\'' => b'\'',
        _ => 0,
    };
    if replace != 0 {
        bstr_xappend(dst, &[replace]);
        return (true, bstr_cut(code, 1));
    }
    if code[0] == b'x' && code.len() >= 3 {
        let (value, num) = bstrtoll(bstr_splice(code, 1, 3), 16);
        let c = value as u8;
        if !num.is_empty() {
            return (false, code);
        }
        bstr_xappend(dst, &[c]);
        return (true, bstr_cut(code, 3));
    }
    if code[0] == b'u' && code.len() >= 5 {
        let mut code = code;
        let (value, num) = bstrtoll(bstr_splice(code, 1, 5), 16);
        let mut c = value as u32;
        if !num.is_empty() || c > 0x0010_ffff {
            return (false, code);
        }
        if c >= 0xd800 && c <= 0xdbff {
            if code.len() < 5 + 6 || code[5] != b'\\' || code[6] != b'u' {
                return (false, code);
            }
            code = bstr_cut(code, 5 + 1);
            let (value2, num2) = bstrtoll(bstr_splice(code, 1, 5), 16);
            let c2 = value2 as u32;
            if !num2.is_empty() || c2 < 0xdc00 || c2 > 0xdfff {
                return (false, code);
            }
            c = ((c - 0xd800) << 10) + 0x10000 + (c2 - 0xdc00);
        }
        mp_append_utf8_bstr(dst, c);
        return (true, bstr_cut(code, 5));
    }
    (false, code)
}

/// `mp_append_escaped_string` for a `dst` that starts out empty.
///
/// That is the only way `misc/` calls it. Unescapes `src` up to its first
/// `"` or its end, and returns the rest of `src`, or `None` for a broken
/// escape.
pub fn mp_append_escaped_string<'a>(dst: &mut Vec<u8>, src: &'a [u8]) -> Option<&'a [u8]> {
    let mut t = src;
    let mut cur = 0;
    loop {
        if cur >= t.len() || t[cur] == b'"' {
            let rest = bstr_cut(t, cur as i32);
            bstr_xappend(dst, bstr_splice(t, 0, cur as i32));
            return Some(rest);
        } else if t[cur] == b'\\' {
            bstr_xappend(dst, bstr_splice(t, 0, cur as i32));
            t = bstr_cut(t, cur as i32 + 1);
            cur = 0;
            let (ok, rest) = mp_parse_escape(dst, t);
            if !ok {
                return None;
            }
            t = rest;
        } else {
            cur += 1;
        }
    }
}
