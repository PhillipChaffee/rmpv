//! Port of `misc/json.c`: mpv's JSON parser and writer.
//!
//! The parser reads a NUL-terminated buffer through a `char **` cursor and
//! writes NUL bytes into it. Here the buffer is `src` and the cursor is the
//! index `pos`; the same bytes are overwritten, so the buffer ends up as the
//! C leaves it. Strings in the result are copies where the C points into the
//! buffer.

use crate::bstr::{bstr_xappend, bstr0};
use crate::clib::{cstr_at, format_d, format_f, isfinite, strlen, strncmp, strtod, strtoll};
use crate::common::mp_append_escaped_string;
use crate::ctype::{mp_isalnum, mp_isalpha};
use crate::node::{MpvNode, MpvNodeList};

/// `eat_c`.
pub fn eat_c(src: &[u8], pos: &mut usize, c: u8) -> bool {
    if cstr_at(src, *pos) == c {
        *pos += 1;
        return true;
    }
    false
}

/// `eat_ws`.
pub fn eat_ws(src: &[u8], pos: &mut usize) {
    loop {
        let c = cstr_at(src, *pos);
        if c != b' ' && c != b'\t' && c != b'\n' && c != b'\r' {
            return;
        }
        *pos += 1;
    }
}

/// `json_skip_whitespace`.
pub fn json_skip_whitespace(src: &[u8], pos: &mut usize) {
    eat_ws(src, pos);
}

/// `read_id`: an unquoted object key.
pub fn read_id(dst: &mut MpvNode, src: &mut [u8], pos: &mut usize) -> i32 {
    let start = *pos;
    if !mp_isalpha(cstr_at(src, *pos)) && cstr_at(src, *pos) != b'_' {
        return -1;
    }
    while mp_isalnum(cstr_at(src, *pos)) || cstr_at(src, *pos) == b'_' {
        *pos += 1;
    }
    let end = *pos;
    if cstr_at(src, *pos) == b' ' {
        // The C terminates the key in place instead of copying it.
        src[*pos] = 0;
        *pos += 1;
    }
    *dst = MpvNode::String(src[start..end].to_vec());
    0
}

/// The scan loop of `read_str`: the index of the byte that ends the literal.
fn find_str_end(src: &[u8], start: usize) -> (usize, bool) {
    let mut cur = start;
    let mut has_escapes = false;
    while cstr_at(src, cur) != 0 && cstr_at(src, cur) != b'"' {
        if cstr_at(src, cur) == b'\\' {
            has_escapes = true;
            // Skip \" and \\, the latter so that \\" ends the string.
            if cstr_at(src, cur + 1) == b'"' || cstr_at(src, cur + 1) == b'\\' {
                cur += 1;
            }
        }
        cur += 1;
    }
    (cur, has_escapes)
}

/// `read_str`: a quoted string.
pub fn read_str(dst: &mut MpvNode, src: &mut [u8], pos: &mut usize) -> i32 {
    if !eat_c(src, pos, b'"') {
        return -1;
    }
    let start = *pos;
    let (cur, has_escapes) = find_str_end(src, start);
    if cstr_at(src, cur) != b'"' {
        return -1;
    }
    src[cur] = 0;
    *pos = cur + 1;
    let mut string = src[start..cur].to_vec();
    if has_escapes {
        let mut unescaped = Vec::new();
        if mp_append_escaped_string(&mut unescaped, &src[start..cur]).is_none() {
            return -1;
        }
        // The node holds a C string, so an escaped NUL ends it.
        string = bstr0(&unescaped).to_vec();
    }
    *dst = MpvNode::String(string);
    0
}

/// `read_sub`: an array or object.
pub fn read_sub(dst: &mut MpvNode, src: &mut [u8], pos: &mut usize, max_depth: i32) -> i32 {
    let is_arr = eat_c(src, pos, b'[');
    let is_obj = !is_arr && eat_c(src, pos, b'{');
    if !is_arr && !is_obj {
        return -1;
    }
    let term = if is_obj { b'}' } else { b']' };
    let mut list = MpvNodeList {
        values: Vec::new(),
        keys: Vec::new(),
    };
    loop {
        eat_ws(src, pos);
        if eat_c(src, pos, term) {
            break;
        }
        if !list.values.is_empty() && !eat_c(src, pos, b',') {
            return -1;
        }
        eat_ws(src, pos);
        // A trailing "," is allowed.
        if eat_c(src, pos, term) {
            break;
        }
        if is_obj {
            let mut key_node = MpvNode::None;
            // Keys may be unquoted.
            if read_id(&mut key_node, src, pos) < 0 && read_str(&mut key_node, src, pos) < 0 {
                return -1;
            }
            eat_ws(src, pos);
            // "=" is allowed in place of ":".
            if !eat_c(src, pos, b':') && !eat_c(src, pos, b'=') {
                return -1;
            }
            eat_ws(src, pos);
            if let MpvNode::String(key) = key_node {
                list.keys.push(key);
            }
        }
        let mut value = MpvNode::None;
        if json_parse(&mut value, src, pos, max_depth) < 0 {
            return -1;
        }
        list.values.push(value);
    }
    *dst = if is_obj {
        MpvNode::NodeMap(list)
    } else {
        MpvNode::NodeArray(list)
    };
    0
}

/// The number branch of `json_parse`.
///
/// A number is an integer if `strtoll` gets at least as far as `strtod`. A
/// range error makes either parser count as having read nothing, so
/// `1e-400` is the integer 1 followed by `e-400`.
fn parse_number(dst: &mut MpvNode, src: &[u8], pos: &mut usize) -> i32 {
    let text = &src[(*pos).min(src.len())..];
    let int = strtoll(text, 0);
    let nsrci = if int.erange { 0 } else { int.consumed };
    let float = strtod(text);
    let nsrcf = if float.erange { 0 } else { float.consumed };
    if nsrci >= nsrcf {
        *pos += nsrci;
        *dst = MpvNode::Int64(int.value);
        return 0;
    }
    if nsrcf > 0 && isfinite(float.bits) {
        *pos += nsrcf;
        *dst = MpvNode::Double(float.bits);
        return 0;
    }
    -1
}

/// `json_parse`: parses one JSON value from the C string in `src`.
///
/// Parsing starts at `*pos`. Returns 0 and sets `*dst`, with `*pos` after
/// the value, or returns -1. `src` is modified either way. `max_depth`
/// bounds nesting.
pub fn json_parse(dst: &mut MpvNode, src: &mut [u8], pos: &mut usize, max_depth: i32) -> i32 {
    let max_depth = max_depth.wrapping_sub(1);
    if max_depth < 0 {
        return -1;
    }

    eat_ws(src, pos);

    let c = cstr_at(src, *pos);
    let here = &src[(*pos).min(src.len())..];
    if c == 0 {
        return -1;
    }
    if c == b'n' && strncmp(here, b"null", 4) == 0 {
        *pos += 4;
        *dst = MpvNode::None;
        return 0;
    } else if c == b't' && strncmp(here, b"true", 4) == 0 {
        *pos += 4;
        *dst = MpvNode::Flag(1);
        return 0;
    } else if c == b'f' && strncmp(here, b"false", 5) == 0 {
        *pos += 5;
        *dst = MpvNode::Flag(0);
        return 0;
    } else if c == b'"' {
        return read_str(dst, src, pos);
    } else if c == b'[' || c == b'{' {
        return read_sub(dst, src, pos, max_depth);
    } else if c == b'-' || (c >= b'0' && c <= b'9') {
        return parse_number(dst, src, pos);
    }
    -1
}

/// `special_escape`: the letter that escapes a control character, or 0.
fn special_escape(c: u8) -> u8 {
    match c {
        0x08 => b'b',
        0x0c => b'f',
        b'\n' => b'n',
        b'\r' => b'r',
        b'\t' => b't',
        _ => 0,
    }
}

/// The lowercase hexadecimal digit for `v < 16`.
fn hex_digit(v: u8) -> u8 {
    if v < 10 { b'0' + v } else { b'a' + (v - 10) }
}

/// The inner loop of `write_json_str`: the index of the next byte at or
/// after `start` that cannot be copied as it is.
fn scan_plain(s: &[u8], start: usize) -> usize {
    let mut cur = start;
    while cstr_at(s, cur) >= 32 && cstr_at(s, cur) != b'"' && cstr_at(s, cur) != b'\\' {
        cur += 1;
    }
    cur
}

/// `write_json_str`: appends the C string `s` as a quoted JSON string.
pub fn write_json_str(b: &mut Vec<u8>, s: &[u8]) {
    bstr_xappend(b, b"\"");
    let mut start = 0;
    let mut cur = scan_plain(s, start);
    while cstr_at(s, cur) != 0 {
        let c = cstr_at(s, cur);
        bstr_xappend(b, &s[start..cur]);
        if c == b'"' {
            bstr_xappend(b, b"\\\"");
        } else if c == b'\\' {
            bstr_xappend(b, b"\\\\");
        } else if special_escape(c) != 0 {
            bstr_xappend(b, &[b'\\', special_escape(c)]);
        } else {
            bstr_xappend(
                b,
                &[
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    hex_digit(c >> 4),
                    hex_digit(c & 15),
                ],
            );
        }
        start = cur + 1;
        cur = scan_plain(s, start);
    }
    bstr_xappend(b, &s[start..cur]);
    bstr_xappend(b, b"\"");
}

/// `add_indent`.
pub fn add_indent(b: &mut Vec<u8>, indent: i32) {
    if indent < 0 {
        return;
    }
    bstr_xappend(b, b"\n");
    let mut n = 0;
    while n < indent {
        bstr_xappend(b, b" ");
        n += 1;
    }
}

/// The entry loop of `json_append` for arrays and objects, from index `n`.
fn append_list(b: &mut Vec<u8>, list: &MpvNodeList, is_obj: bool, next_indent: i32, n: usize) {
    if n >= list.values.len() {
        return;
    }
    if n != 0 {
        bstr_xappend(b, b",");
    }
    add_indent(b, next_indent);
    if is_obj {
        write_json_str(b, &list.keys[n]);
        bstr_xappend(b, b":");
    }
    json_append(b, &list.values[n], next_indent);
    append_list(b, list, is_obj, next_indent, n + 1);
}

/// `json_append`: appends `src` as JSON. `indent < 0` writes one line;
/// otherwise each nesting level is indented by one more space.
///
/// At `indent == 0`, the top level of pretty printing, a string is written
/// raw without quotes and a non-finite number without them, as in the C.
/// Returns -1 for a format JSON cannot express.
pub fn json_append(b: &mut Vec<u8>, src: &MpvNode, indent: i32) -> i32 {
    match src {
        MpvNode::None => {
            bstr_xappend(b, b"null");
            0
        }
        MpvNode::Flag(flag) => {
            bstr_xappend(b, if *flag != 0 { b"true" } else { b"false" });
            0
        }
        MpvNode::Int64(v) => {
            bstr_xappend(b, &format_d(*v));
            0
        }
        MpvNode::Double(bits) => {
            let quote: &[u8] = if isfinite(*bits) || indent == 0 {
                b""
            } else {
                b"\""
            };
            bstr_xappend(b, quote);
            bstr_xappend(b, &format_f(*bits));
            bstr_xappend(b, quote);
            0
        }
        MpvNode::String(s) => {
            if indent == 0 {
                bstr_xappend(b, bstr0(s));
            } else {
                write_json_str(b, s);
            }
            0
        }
        MpvNode::NodeArray(list) | MpvNode::NodeMap(list) => {
            let is_obj = matches!(src, MpvNode::NodeMap(_));
            bstr_xappend(b, if is_obj { b"{" } else { b"[" });
            let next_indent = if indent >= 0 { indent + 1 } else { -1 };
            append_list(b, list, is_obj, next_indent, 0);
            add_indent(b, indent);
            bstr_xappend(b, if is_obj { b"}" } else { b"]" });
            0
        }
        MpvNode::ByteArray(_) => -1,
    }
}

/// `json_append_str`: appends to the C string held in `dst`.
pub fn json_append_str(dst: &mut Vec<u8>, src: &MpvNode, indent: i32) -> i32 {
    let len = strlen(dst);
    dst.truncate(len);
    json_append(dst, src, indent)
}

/// `json_write`: appends `src` to `dst` as one line of JSON.
pub fn json_write(dst: &mut Vec<u8>, src: &MpvNode) -> i32 {
    json_append_str(dst, src, -1)
}

/// `json_write_pretty`: appends `src` to `dst` as indented JSON.
pub fn json_write_pretty(dst: &mut Vec<u8>, src: &MpvNode) -> i32 {
    json_append_str(dst, src, 0)
}
