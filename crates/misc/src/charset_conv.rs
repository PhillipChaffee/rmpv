//! Port of `misc/charset_conv.c` as built without iconv and uchardet.
//!
//! The C logs what it decides; the port does not.

use crate::bstr::{
    bstr_case_startswith, bstr_sanitize_utf8_latin1, bstr_startswith0, bstr_validate_utf8, bstr0,
};
use crate::clib::{cstr_at, strcasecmp};

pub const MP_ICONV_VERBOSE: i32 = 1;
pub const MP_ICONV_ALLOW_CUTOFF: i32 = 2;
pub const MP_NO_LATIN1_FALLBACK: i32 = 8;

/// `mp_charset_is_utf8` on a C string. NULL in C is the empty string here;
/// both give false.
#[must_use]
pub fn mp_charset_is_utf8(user_cp: &[u8]) -> bool {
    strcasecmp(user_cp, b"utf8") == 0 || strcasecmp(user_cp, b"utf-8") == 0
}

/// `mp_charset_is_utf16` on a C string.
#[must_use]
pub fn mp_charset_is_utf16(user_cp: &[u8]) -> bool {
    let s = bstr0(user_cp);
    bstr_case_startswith(s, b"utf16") || bstr_case_startswith(s, b"utf-16")
}

// Strings the functions below return. They are statics because Aeneas
// treats a byte-string literal as a local value, which cannot be returned by
// reference.
static BOM_UTF8: [u8; 3] = [0xEF, 0xBB, 0xBF];
static BOM_UTF16LE: [u8; 2] = [0xFF, 0xFE];
static BOM_UTF16BE: [u8; 2] = [0xFE, 0xFF];
static ENC_UTF8: [u8; 5] = *b"utf-8";
static ENC_UTF16LE: [u8; 8] = *b"utf-16le";
static ENC_UTF16BE: [u8; 8] = *b"utf-16be";
static ENC_UTF8_BROKEN: [u8; 12] = *b"UTF-8-BROKEN";

/// `utf_bom[n]`.
fn utf_bom(n: usize) -> &'static [u8] {
    if n == 0 {
        &BOM_UTF8
    } else if n == 1 {
        &BOM_UTF16LE
    } else {
        &BOM_UTF16BE
    }
}

/// `utf_enc[n]`.
fn utf_enc(n: usize) -> &'static [u8] {
    if n == 0 {
        &ENC_UTF8
    } else if n == 1 {
        &ENC_UTF16LE
    } else {
        &ENC_UTF16BE
    }
}

/// `ms_bom_guess`: the encoding a byte order mark at the start of `buf`
/// names.
#[must_use]
pub fn ms_bom_guess(buf: &[u8]) -> Option<&'static [u8]> {
    let mut n = 0;
    while n < 3 {
        if bstr_startswith0(buf, utf_bom(n)) {
            return Some(utf_enc(n));
        }
        n += 1;
    }
    None
}

/// `mp_charset_guess`: the character set to read `buf` in, given the user's
/// choice `user_cp`, a C string that must not be NULL. The result is `user_cp`,
/// its tail after a leading `+`, or a constant.
#[must_use]
pub fn mp_charset_guess<'a>(buf: &[u8], user_cp: &'a [u8], flags: i32) -> &'a [u8] {
    let user_cp = bstr0(user_cp);
    if cstr_at(user_cp, 0) == b'+' {
        return &user_cp[1..];
    }

    if let Some(bom_cp) = ms_bom_guess(buf) {
        return bom_cp;
    }

    let r = bstr_validate_utf8(buf);
    if r >= 0 || (r > -8 && (flags & MP_ICONV_ALLOW_CUTOFF) != 0) {
        return &ENC_UTF8;
    }

    if strcasecmp(user_cp, b"auto") == 0 {
        return &ENC_UTF8_BROKEN;
    }
    user_cp
}

/// `mp_iconv_to_utf8`. Without iconv the C only ever returns its input or
/// the Latin-1 repair of it, whatever `cp` is. `None` stands for the input
/// itself.
#[must_use]
pub fn mp_iconv_to_utf8(buf: &[u8], cp: &[u8], flags: i32) -> Option<Vec<u8>> {
    let _ = cp;
    if (flags & MP_NO_LATIN1_FALLBACK) != 0 {
        None
    } else {
        bstr_sanitize_utf8_latin1(buf)
    }
}
