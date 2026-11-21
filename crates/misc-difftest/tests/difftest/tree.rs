//! `json.c` and `node.c`.

use std::ptr::null_mut;

use rmpv_misc::node::{self, MpvNode};
use rmpv_misc::{clib, json};
use rmpv_misc_difftest::ffi::Bstr;
use rmpv_misc_difftest::input::{self, Rng, cstring};
use rmpv_misc_difftest::node_wire::{random_double, random_node, to_wire};
use rmpv_misc_difftest::{c, take_malloced};

use crate::{N, run};

/// JSON-like text: real JSON from the port's writer, the parser's
/// extensions, numbers at the edges, and random damage.
fn json_text(rng: &mut Rng) -> Vec<u8> {
    let mut out = Vec::new();
    match rng.below(6) {
        0 => out.extend_from_slice(&input::integer_text(rng)),
        1 => out.extend_from_slice(&input::float_text(rng)),
        2 => out.extend_from_slice(rng.pick(&[
            &b"null"[..],
            b"true",
            b"false",
            b"nul",
            b"truex",
            b"fals",
            b"-",
            b"[-]",
            b"[-,-]",
            b"[1,]",
            b"[,]",
            b"{a=1}",
            b"{a :1}",
            b"{a b:1}",
            b"{_a1:[]}",
            b"{1:2}",
            b"{\"k\"=null,}",
            b"[[[[[[]]]]]]",
            b"\"a\\u00e9\\n\"",
            b"\"\\x41\\x00z\"",
            b"\"\\ud83d\\ude00\"",
            b"\"\\q\"",
            b"\"abc",
            b"\"a\\\"b\"",
            b"\"a\\\\\"",
            b"\"a\\\\\\\"\"",
            b"1e-400",
            b"99999999999999999999e999",
            b"-nan(99999999999999999999)",
            b"-inf",
            b"-0x1p3",
            b"0b101",
            b"0x",
            b"[1 2]",
            b"{\"a\":}",
            b"{a:1,b:[true,false,null],c:{d:\"e\"}}",
            b" \t\r\n[ 1 , 2 ] ",
            b"\"\\u-001\"",
            b"\"\\\"",
        ])),
        _ => {
            let node = random_node(rng, 3, false);
            let _ = if rng.chance(2) {
                json::json_write(&mut out, &node)
            } else {
                json::json_write_pretty(&mut out, &node)
            };
        }
    }
    for _ in 0..rng.below(3) {
        if out.is_empty() {
            break;
        }
        let i = rng.below(out.len());
        match rng.below(5) {
            0 => {
                out.remove(i);
            }
            1 => out[i] = rng.pick(b"[]{},:=\"\\ \t\n\r-+.eE019xbu_ntf"),
            2 => out.insert(i, rng.pick(b"[]{},:=\"\\ \t\n\r-+.eE019xbu_ntf")),
            3 => out.insert(i, input::byte(rng)),
            _ => out.truncate(i),
        }
    }
    out
}

fn depth(rng: &mut Rng) -> i32 {
    match rng.below(6) {
        0 => rng.pick(&[0, -1, i32::MIN, 1, 2]),
        1 => 1 + rng.below(4) as i32,
        _ => 100,
    }
}

/// One of the C parsing entry points: `json_parse`, or `read_id`, `read_str`
/// or `read_sub` through the shim.
fn c_parse(
    which: Option<i32>,
    buf: &mut [u8],
    pos: &mut usize,
    max_depth: i32,
) -> (i32, Option<Vec<u8>>) {
    let (mut dump, mut dump_len) = (null_mut(), 0);
    let r = match which {
        None => c!(dt_json_parse(
            buf.as_mut_ptr().cast(),
            pos,
            max_depth,
            &raw mut dump,
            &raw mut dump_len
        )),
        Some(w) => c!(dt_read(
            w,
            buf.as_mut_ptr().cast(),
            pos,
            max_depth,
            &raw mut dump,
            &raw mut dump_len
        )),
    };
    (
        r,
        if dump.is_null() {
            None
        } else {
            Some(take_malloced(dump, dump_len))
        },
    )
}

fn check_parse(
    name: &str,
    seed: u64,
    which: Option<i32>,
    port: impl Fn(&mut MpvNode, &mut [u8], &mut usize, i32) -> i32,
) {
    run(name, seed, N, |rng| {
        let text = if which == Some(0) && rng.chance(2) {
            let mut id = input::text(rng, 5);
            id.extend_from_slice(rng.pick(&[&b" "[..], b":", b" :", b"", b"=1"]));
            id
        } else {
            json_text(rng)
        };
        let shown = String::from_utf8_lossy(&text).into_owned();
        let max_depth = depth(rng);
        let mut c_buf = cstring(&text);
        let mut c_pos = 0;
        let (want, want_dump) = c_parse(which, &mut c_buf, &mut c_pos, max_depth);
        let mut r_buf = text.clone();
        let mut r_pos = 0;
        let mut dst = MpvNode::None;
        let got = port(&mut dst, &mut r_buf, &mut r_pos, max_depth);
        assert_eq!((want, c_pos), (got, r_pos), "{shown:?} depth {max_depth}");
        // What the parser wrote into its input.
        assert_eq!(&c_buf[..text.len()], r_buf, "{shown:?} depth {max_depth}");
        if got >= 0 {
            assert_eq!(
                want_dump,
                Some(to_wire(&dst)),
                "{shown:?} depth {max_depth}: {dst:?}"
            );
        }
    });
}

#[test]
fn json_parse() {
    check_parse("json_parse", 200, None, json::json_parse);
}

#[test]
fn read_id() {
    check_parse("read_id", 201, Some(0), |dst, src, pos, _| {
        json::read_id(dst, src, pos)
    });
}

#[test]
fn read_str() {
    check_parse("read_str", 202, Some(1), |dst, src, pos, _| {
        json::read_str(dst, src, pos)
    });
}

#[test]
fn read_sub() {
    check_parse("read_sub", 203, Some(2), json::read_sub);
}

#[test]
fn eat() {
    run("eat_c", 204, N, |rng| {
        let text = input::bytes(rng, 6);
        let ch = if text.is_empty() || rng.chance(3) {
            input::byte(rng).max(1)
        } else {
            text[0].max(1)
        };
        let mut buf = cstring(&text);
        let (mut c_pos, mut r_pos) = (0, 0);
        let want = c!(dt_eat_c(
            buf.as_mut_ptr().cast(),
            &raw mut c_pos,
            ch as std::ffi::c_char
        ));
        assert_eq!(
            (want, c_pos),
            (json::eat_c(&text, &mut r_pos, ch), r_pos),
            "{text:?} {ch}"
        );
    });
    run("eat_ws", 205, N, |rng| {
        let mut text = Vec::new();
        for _ in 0..rng.below(5) {
            text.push(rng.pick(b" \t\n\r \t\n\r\x0b\x0c\0a"));
        }
        text.extend_from_slice(&input::bytes(rng, 3));
        let mut buf = cstring(&text);
        let mut r_pos = 0;
        json::eat_ws(&text, &mut r_pos);
        assert_eq!(c!(dt_eat_ws(buf.as_mut_ptr().cast(), 0)), r_pos, "{text:?}");
        let mut r_pos = 0;
        json::json_skip_whitespace(&text, &mut r_pos);
        assert_eq!(
            c!(dt_json_skip_whitespace(buf.as_mut_ptr().cast(), 0)),
            r_pos,
            "{text:?}"
        );
    });
    println!("difftest json_skip_whitespace: {N} inputs agree");
}

fn c_write(mode: i32, node: &MpvNode, prefix: &[u8], indent: i32) -> (i32, Vec<u8>) {
    let wire = to_wire(node);
    let prefix0 = cstring(prefix);
    let (mut out, mut out_len) = (null_mut(), 0);
    let r = c!(dt_json_write(
        mode,
        wire.as_ptr(),
        prefix0.as_ptr().cast(),
        indent,
        &raw mut out,
        &raw mut out_len
    ));
    (r, take_malloced(out, out_len))
}

fn indent(rng: &mut Rng) -> i32 {
    match rng.below(4) {
        0 => rng.pick(&[-1, 0, 1, i32::MIN, -2]),
        1 => rng.below(4) as i32,
        _ => rng.pick(&[-1, 0]),
    }
}

#[test]
fn json_append() {
    run("json_append", 206, N, |rng| {
        let node = random_node(rng, 3, true);
        let prefix = input::text(rng, 3);
        let indent = indent(rng);
        let (want, want_out) = c_write(0, &node, &prefix, indent);
        let mut out = prefix;
        let got = json::json_append(&mut out, &node, indent);
        assert_eq!((want, want_out), (got, out), "{node:?} indent {indent}");
    });
}

#[test]
fn json_write() {
    run("json_write", 207, N, |rng| {
        let node = random_node(rng, 3, true);
        let prefix = input::text(rng, 3);
        let (want, want_out) = c_write(1, &node, &prefix, 0);
        let mut out = prefix;
        let got = json::json_write(&mut out, &node);
        assert_eq!((want, want_out), (got, out), "{node:?}");
    });
}

#[test]
fn json_write_pretty() {
    run("json_write_pretty", 208, N, |rng| {
        let node = random_node(rng, 3, true);
        let prefix = input::text(rng, 3);
        let (want, want_out) = c_write(2, &node, &prefix, 0);
        let mut out = prefix;
        let got = json::json_write_pretty(&mut out, &node);
        assert_eq!((want, want_out), (got, out), "{node:?}");
    });
}

#[test]
fn json_static_writers() {
    run("write_json_str", 209, N, |rng| {
        let s = input::text(rng, 10);
        let prefix = input::text(rng, 3);
        let (mut s0, prefix0) = (cstring(&s), cstring(&prefix));
        let mut len = 0;
        let p = c!(dt_write_json_str(
            prefix0.as_ptr().cast(),
            s0.as_mut_ptr(),
            &raw mut len
        ));
        let mut out = prefix;
        json::write_json_str(&mut out, &s);
        assert_eq!(take_malloced(p, len), out, "{s:?}");
    });
    run("add_indent", 210, N, |rng| {
        let prefix = input::text(rng, 3);
        let prefix0 = cstring(&prefix);
        let indent = if rng.chance(4) {
            rng.pick(&[i32::MIN, -1, 0])
        } else {
            rng.below(20) as i32 - 3
        };
        let mut len = 0;
        let p = c!(dt_add_indent(prefix0.as_ptr().cast(), indent, &raw mut len));
        let mut out = prefix;
        json::add_indent(&mut out, indent);
        assert_eq!(take_malloced(p, len), out, "{indent}");
    });
    run("json_append_str", 211, N, |rng| {
        let node = random_node(rng, 3, true);
        let prefix = input::text(rng, 3);
        let prefix0 = cstring(&prefix);
        let indent = indent(rng);
        let wire = to_wire(&node);
        let (mut r, mut len) = (0, 0);
        let p = c!(dt_json_append_str(
            prefix0.as_ptr().cast(),
            wire.as_ptr(),
            indent,
            &raw mut r,
            &raw mut len
        ));
        let mut out = prefix;
        let got = json::json_append_str(&mut out, &node, indent);
        assert_eq!(
            (r, take_malloced(p, len)),
            (got, out),
            "{node:?} indent {indent}"
        );
    });
}

const INIT_FORMATS: [i32; 7] = [
    node::MPV_FORMAT_NODE_MAP,
    node::MPV_FORMAT_NODE_ARRAY,
    node::MPV_FORMAT_FLAG,
    node::MPV_FORMAT_INT64,
    node::MPV_FORMAT_DOUBLE,
    node::MPV_FORMAT_BYTE_ARRAY,
    node::MPV_FORMAT_NONE,
];

#[test]
fn node_init() {
    // Every format and parent the C's assertions allow: 7 formats, and no
    // parent, a map or an array. 21 inputs are the whole domain.
    let mut count = 0;
    for format in INIT_FORMATS {
        for parent_format in [-1, node::MPV_FORMAT_NODE_MAP, node::MPV_FORMAT_NODE_ARRAY] {
            let mut len = 0;
            let p = c!(dt_node_init(format, parent_format, &raw mut len));
            let mut parent = MpvNode::None;
            if parent_format >= 0 {
                node::node_init(&mut parent, parent_format, None);
            }
            let mut dst = MpvNode::String(b"old".to_vec());
            node::node_init(
                &mut dst,
                format,
                if parent_format >= 0 {
                    Some(&parent)
                } else {
                    None
                },
            );
            assert_eq!(
                take_malloced(p, len),
                to_wire(&dst),
                "{format} {parent_format}"
            );
            count += 1;
        }
    }
    println!("difftest node_init: {count} inputs agree");
}

fn random_list(rng: &mut Rng, map: bool) -> MpvNode {
    loop {
        let n = random_node(rng, 2, true);
        if matches!(
            (&n, map),
            (MpvNode::NodeMap(_), true) | (MpvNode::NodeArray(_), false)
        ) {
            return n;
        }
    }
}

fn check_add(name: &str, seed: u64, op: i32) {
    run(name, seed, N, |rng| {
        let mut dst = random_list(rng, op != 0);
        // A C string key for all but node_map_badd, whose key may hold NUL.
        let key = if op == 2 {
            input::bytes(rng, 5)
        } else {
            input::text(rng, 5)
        };
        let key0 = cstring(&key);
        let format = rng.pick(&INIT_FORMATS);
        let sval = if op == 4 {
            input::bytes(rng, 6)
        } else {
            input::text(rng, 6)
        };
        let sval0 = cstring(&sval);
        let ival = if op == 7 {
            rng.below(2) as i64
        } else {
            rng.next_u64() as i64
        };
        let dval = random_double(rng);
        let wire = to_wire(&dst);
        let (mut index, mut len) = (0, 0);
        // The bstr arguments point at NUL-terminated copies, as the shim
        // passes their start as a C string where the function takes one.
        let c_key = Bstr {
            start: key0.as_ptr().cast_mut(),
            len: key.len(),
        };
        let c_sval = Bstr {
            start: sval0.as_ptr().cast_mut(),
            len: sval.len(),
        };
        let p = c!(dt_node_add(
            op,
            wire.as_ptr(),
            c_key,
            format,
            c_sval,
            ival,
            f64::from_bits(dval),
            &raw mut index,
            &raw mut len
        ));
        let got_index = match op {
            0 => node::node_array_add(&mut dst, format) as i32,
            1 => node::node_map_add(&mut dst, &key, format) as i32,
            2 => node::node_map_badd(&mut dst, &key, format) as i32,
            3 => {
                node::node_map_add_string(&mut dst, &key, &sval);
                -1
            }
            4 => {
                node::node_map_add_bstr(&mut dst, &key, &sval);
                -1
            }
            5 => {
                node::node_map_add_int64(&mut dst, &key, ival);
                -1
            }
            6 => {
                node::node_map_add_double(&mut dst, &key, dval);
                -1
            }
            _ => {
                node::node_map_add_flag(&mut dst, &key, ival != 0);
                -1
            }
        };
        assert_eq!(
            (index, take_malloced(p, len)),
            (got_index, to_wire(&dst)),
            "{dst:?} {key:?}"
        );
    });
}

#[test]
fn node_array_add() {
    check_add("node_array_add", 220, 0);
}

#[test]
fn node_map_add() {
    check_add("node_map_add", 221, 1);
}

#[test]
fn node_map_badd() {
    check_add("node_map_badd", 222, 2);
}

#[test]
fn node_map_add_string() {
    check_add("node_map_add_string", 223, 3);
}

#[test]
fn node_map_add_bstr() {
    check_add("node_map_add_bstr", 224, 4);
}

#[test]
fn node_map_add_int64() {
    check_add("node_map_add_int64", 225, 5);
}

#[test]
fn node_map_add_double() {
    check_add("node_map_add_double", 226, 6);
}

#[test]
fn node_map_add_flag() {
    check_add("node_map_add_flag", 227, 7);
}

fn check_get(name: &str, seed: u64, bkey: bool) {
    run(name, seed, N, |rng| {
        let src = if rng.chance(4) {
            random_node(rng, 2, true)
        } else {
            random_list(rng, true)
        };
        let known = match &src {
            MpvNode::NodeMap(list) if !list.keys.is_empty() && !rng.chance(3) => {
                Some(list.keys[rng.below(list.keys.len())].clone())
            }
            _ => None,
        };
        let mut key = match known {
            Some(k) => input::near(rng, &k),
            None => input::text(rng, 5),
        };
        if !bkey {
            key.retain(|&b| b != 0);
        }
        let key0 = cstring(&key);
        let wire = to_wire(&src);
        let c_key = Bstr {
            start: key0.as_ptr().cast_mut(),
            len: key.len(),
        };
        let want = c!(dt_node_map_get(wire.as_ptr(), c_key, i32::from(bkey)));
        let got = if bkey {
            node::node_map_bget(&src, &key)
        } else {
            node::node_map_get(&src, &key)
        };
        let got_index = match (&src, got) {
            (MpvNode::NodeMap(list), Some(found)) => list
                .values
                .iter()
                .position(|v| std::ptr::eq(v, found))
                .map_or(-2, |i| i as i32),
            _ => -1,
        };
        assert_eq!(want, got_index, "{src:?} {key:?}");
    });
}

#[test]
fn node_map_get() {
    check_get("node_map_get", 228, false);
}

#[test]
fn node_map_bget() {
    check_get("node_map_bget", 229, true);
}

/// Changes one thing somewhere in the tree, or nothing.
fn perturb(rng: &mut Rng, node: &mut MpvNode, byte_arrays: bool) {
    match node {
        MpvNode::NodeArray(list) | MpvNode::NodeMap(list)
            if !list.values.is_empty() && !rng.chance(4) =>
        {
            let i = rng.below(list.values.len());
            if !list.keys.is_empty() && rng.chance(4) {
                list.keys[i] = input::near(rng, &list.keys[i].clone());
                list.keys[i].retain(|&b| b != 0);
            } else if rng.chance(8) {
                list.values.pop();
                list.keys.truncate(list.values.len());
            } else {
                perturb(rng, &mut list.values[i], byte_arrays);
            }
        }
        MpvNode::Double(bits) if rng.chance(2) => {
            // The two zeros are equal; a NaN is not equal to itself.
            *bits ^= rng.pick(&[1 << 63, 1, 0]);
        }
        MpvNode::String(s) if rng.chance(2) => {
            *s = input::near(rng, &s.clone());
            s.retain(|&b| b != 0);
        }
        _ => {
            if rng.chance(3) {
                *node = random_node(rng, 1, byte_arrays);
            }
        }
    }
}

#[test]
fn equal_mpv_node() {
    run("equal_mpv_node", 230, N, |rng| {
        // No byte arrays: for those the C compares the wrong memory (see
        // docs/pilot-misc-rust.md).
        let a = random_node(rng, 3, false);
        let mut b = a.clone();
        if !rng.chance(3) {
            perturb(rng, &mut b, false);
        }
        let (wa, wb) = (to_wire(&a), to_wire(&b));
        assert_eq!(
            c!(dt_node_equal(wa.as_ptr(), wb.as_ptr(), -1)),
            node::equal_mpv_node(&a, &b),
            "{a:?} {b:?}"
        );
    });
}

#[test]
fn equal_mpv_value() {
    run("equal_mpv_value", 231, N, |rng| {
        // The format must be the one both values have, or MPV_FORMAT_NODE.
        // Byte arrays are compared directly but do not occur nested.
        let a = if rng.chance(8) {
            MpvNode::ByteArray(input::bytes(rng, 5))
        } else {
            random_node(rng, 3, false)
        };
        let mut b = a.clone();
        if !rng.chance(3) {
            perturb(rng, &mut b, false);
            if let (MpvNode::ByteArray(x), MpvNode::ByteArray(_)) = (&a, &b)
                && rng.chance(2)
            {
                b = MpvNode::ByteArray(input::near(rng, x));
            }
        }
        let mut format = node::node_format(&a);
        if node::node_format(&b) != format || (rng.chance(4) && !matches!(a, MpvNode::ByteArray(_)))
        {
            format = node::MPV_FORMAT_NODE;
        } else if format == node::MPV_FORMAT_STRING && rng.chance(2) {
            format = node::MPV_FORMAT_OSD_STRING;
        }
        let (wa, wb) = (to_wire(&a), to_wire(&b));
        assert_eq!(
            c!(dt_node_equal(wa.as_ptr(), wb.as_ptr(), format)),
            node::equal_mpv_value(&a, &b, format),
            "{a:?} {b:?} {format}"
        );
    });
}

#[test]
#[expect(
    clippy::float_cmp,
    reason = "the port's double_eq stands for C's == on doubles"
)]
fn double_helpers() {
    run("clib double_eq, isfinite", 232, N, |rng| {
        let a = random_double(rng);
        let b = if rng.chance(2) {
            a ^ rng.pick(&[0, 1 << 63, 1])
        } else {
            random_double(rng)
        };
        assert_eq!(
            f64::from_bits(a) == f64::from_bits(b),
            clib::double_eq(a, b),
            "{a:#x} {b:#x}"
        );
        assert_eq!(f64::from_bits(a).is_finite(), clib::isfinite(a), "{a:#x}");
    });
}
