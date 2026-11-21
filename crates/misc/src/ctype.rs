//! Port of `misc/ctype.h`: locale-independent character classes.
//!
//! The C helpers take `char`. Every caller passes a byte, and every range
//! tested lies below 0x80, so the sign of `char` never matters and the port
//! takes `u8`.

#[must_use]
pub fn mp_isprint(c: u8) -> bool {
    c >= 32
}

#[must_use]
pub fn mp_isspace(c: u8) -> bool {
    c == b' ' || c == 0x0c || c == b'\n' || c == b'\r' || c == b'\t' || c == 0x0b
}

#[must_use]
pub fn mp_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

#[must_use]
pub fn mp_islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

#[must_use]
pub fn mp_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

#[must_use]
pub fn mp_isalpha(c: u8) -> bool {
    mp_isupper(c) || mp_islower(c)
}

#[must_use]
pub fn mp_isalnum(c: u8) -> bool {
    mp_isalpha(c) || mp_isdigit(c)
}

#[must_use]
pub fn mp_tolower(c: u8) -> u8 {
    if mp_isupper(c) { c - b'A' + b'a' } else { c }
}

#[must_use]
pub fn mp_toupper(c: u8) -> u8 {
    if mp_islower(c) { c - b'a' + b'A' } else { c }
}
