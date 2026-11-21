//! Port of `misc/node.c`: building and comparing `mpv_node` trees.

use crate::bstr::{bstr_equals0, bstr0, bstrdup0, bstrto0};
use crate::clib::{double_eq, memcmp, strcmp};

/// `mpv_format` values.
pub const MPV_FORMAT_NONE: i32 = 0;
pub const MPV_FORMAT_STRING: i32 = 1;
pub const MPV_FORMAT_OSD_STRING: i32 = 2;
pub const MPV_FORMAT_FLAG: i32 = 3;
pub const MPV_FORMAT_INT64: i32 = 4;
pub const MPV_FORMAT_DOUBLE: i32 = 5;
pub const MPV_FORMAT_NODE: i32 = 6;
pub const MPV_FORMAT_NODE_ARRAY: i32 = 7;
pub const MPV_FORMAT_NODE_MAP: i32 = 8;
pub const MPV_FORMAT_BYTE_ARRAY: i32 = 9;

/// `mpv_node_list`: the values of an array or map, and for a map the key of
/// each value. `num` in C is `values.len()`.
#[cfg_attr(not(aeneas), derive(Debug, Clone, PartialEq, Eq, Default))]
pub struct MpvNodeList {
    pub values: Vec<MpvNode>,
    pub keys: Vec<Vec<u8>>,
}

/// `mpv_node`: the format tag and the union member it selects.
///
/// A `double` is held as its bit pattern, so that the type and the code that
/// only moves values around stay free of floating point. Strings are C
/// strings and hold no NUL byte.
#[cfg_attr(not(aeneas), derive(Debug, Clone, PartialEq, Eq, Default))]
pub enum MpvNode {
    #[cfg_attr(not(aeneas), default)]
    None,
    String(Vec<u8>),
    Flag(i32),
    Int64(i64),
    Double(u64),
    NodeArray(MpvNodeList),
    NodeMap(MpvNodeList),
    ByteArray(Vec<u8>),
}

/// The `format` field of a node.
#[must_use]
pub fn node_format(n: &MpvNode) -> i32 {
    match n {
        MpvNode::None => MPV_FORMAT_NONE,
        MpvNode::String(_) => MPV_FORMAT_STRING,
        MpvNode::Flag(_) => MPV_FORMAT_FLAG,
        MpvNode::Int64(_) => MPV_FORMAT_INT64,
        MpvNode::Double(_) => MPV_FORMAT_DOUBLE,
        MpvNode::NodeArray(_) => MPV_FORMAT_NODE_ARRAY,
        MpvNode::NodeMap(_) => MPV_FORMAT_NODE_MAP,
        MpvNode::ByteArray(_) => MPV_FORMAT_BYTE_ARRAY,
    }
}

/// `node_init`: sets `dst` to the zero value of `format`.
///
/// # Panics
///
/// Where the C fails `mp_assert`: a `format` that needs manual
/// initialization, or a `parent` that is neither a map nor an array.
pub fn node_init(dst: &mut MpvNode, format: i32, parent: Option<&MpvNode>) {
    assert!(
        format == MPV_FORMAT_NODE_MAP
            || format == MPV_FORMAT_NODE_ARRAY
            || format == MPV_FORMAT_FLAG
            || format == MPV_FORMAT_INT64
            || format == MPV_FORMAT_DOUBLE
            || format == MPV_FORMAT_BYTE_ARRAY
            || format == MPV_FORMAT_NONE
    );

    if let Some(parent) = parent {
        let parent_format = node_format(parent);
        assert!(parent_format == MPV_FORMAT_NODE_MAP || parent_format == MPV_FORMAT_NODE_ARRAY);
    }

    *dst = if format == MPV_FORMAT_NODE_MAP {
        MpvNode::NodeMap(MpvNodeList {
            values: Vec::new(),
            keys: Vec::new(),
        })
    } else if format == MPV_FORMAT_NODE_ARRAY {
        MpvNode::NodeArray(MpvNodeList {
            values: Vec::new(),
            keys: Vec::new(),
        })
    } else if format == MPV_FORMAT_FLAG {
        MpvNode::Flag(0)
    } else if format == MPV_FORMAT_INT64 {
        MpvNode::Int64(0)
    } else if format == MPV_FORMAT_DOUBLE {
        MpvNode::Double(0)
    } else if format == MPV_FORMAT_BYTE_ARRAY {
        MpvNode::ByteArray(Vec::new())
    } else {
        MpvNode::None
    };
}

/// `node_array_add`: appends a new entry of `format` and returns its index,
/// where the C returns a pointer to it.
///
/// # Panics
///
/// If `dst` is not an array, where the C fails `mp_assert`.
pub fn node_array_add(dst: &mut MpvNode, format: i32) -> usize {
    assert!(node_format(dst) == MPV_FORMAT_NODE_ARRAY);
    let mut entry = MpvNode::None;
    node_init(&mut entry, format, Some(dst));
    match dst {
        MpvNode::NodeArray(list) => {
            list.values.push(entry);
            list.values.len() - 1
        }
        _ => 0,
    }
}

/// `node_map_add` with a C string key. Returns the index of the new entry.
///
/// # Panics
///
/// If `dst` is not a map, where the C fails `mp_assert`.
pub fn node_map_add(dst: &mut MpvNode, key: &[u8], format: i32) -> usize {
    node_map_badd(dst, bstr0(key), format)
}

/// `node_map_badd`: appends a new entry of `format` under `key` without
/// looking for an existing one, and returns its index. The stored key is a C
/// string, so it ends at the first NUL byte in `key`.
///
/// # Panics
///
/// If `dst` is not a map, where the C fails `mp_assert`.
pub fn node_map_badd(dst: &mut MpvNode, key: &[u8], format: i32) -> usize {
    assert!(node_format(dst) == MPV_FORMAT_NODE_MAP);
    let mut entry = MpvNode::None;
    node_init(&mut entry, format, Some(dst));
    match dst {
        MpvNode::NodeMap(list) => {
            list.keys.push(bstrdup0(key));
            list.values.push(entry);
            list.values.len() - 1
        }
        _ => 0,
    }
}

/// Stores `value` in entry `index` of a map or array, as the C does through
/// the pointer that the `add` functions return.
fn node_set(dst: &mut MpvNode, index: usize, value: MpvNode) {
    match dst {
        MpvNode::NodeArray(list) | MpvNode::NodeMap(list) => list.values[index] = value,
        _ => {}
    }
}

/// `node_map_add_string`. `val` is a C string.
///
/// # Panics
///
/// If `dst` is not a map.
pub fn node_map_add_string(dst: &mut MpvNode, key: &[u8], val: &[u8]) {
    let entry = node_map_add(dst, key, MPV_FORMAT_NONE);
    node_set(dst, entry, MpvNode::String(bstr0(val).to_vec()));
}

/// `node_map_add_bstr`. The stored value ends at the first NUL in `val`.
///
/// # Panics
///
/// If `dst` is not a map.
pub fn node_map_add_bstr(dst: &mut MpvNode, key: &[u8], val: &[u8]) {
    let entry = node_map_add(dst, key, MPV_FORMAT_NONE);
    node_set(dst, entry, MpvNode::String(bstrto0(val)));
}

/// `node_map_add_int64`.
///
/// # Panics
///
/// If `dst` is not a map.
pub fn node_map_add_int64(dst: &mut MpvNode, key: &[u8], v: i64) {
    let entry = node_map_add(dst, key, MPV_FORMAT_INT64);
    node_set(dst, entry, MpvNode::Int64(v));
}

/// `node_map_add_double`. `v` is the bit pattern of the `double`.
///
/// # Panics
///
/// If `dst` is not a map.
pub fn node_map_add_double(dst: &mut MpvNode, key: &[u8], v: u64) {
    let entry = node_map_add(dst, key, MPV_FORMAT_DOUBLE);
    node_set(dst, entry, MpvNode::Double(v));
}

/// `node_map_add_flag`.
///
/// # Panics
///
/// If `dst` is not a map.
pub fn node_map_add_flag(dst: &mut MpvNode, key: &[u8], v: bool) {
    let entry = node_map_add(dst, key, MPV_FORMAT_FLAG);
    node_set(dst, entry, MpvNode::Flag(v as i32));
}

/// `node_map_get` with a C string key.
#[must_use]
pub fn node_map_get<'a>(src: &'a MpvNode, key: &[u8]) -> Option<&'a MpvNode> {
    node_map_bget(src, bstr0(key))
}

/// `node_map_bget`: the first entry of a map whose key equals `key`.
#[must_use]
pub fn node_map_bget<'a>(src: &'a MpvNode, key: &[u8]) -> Option<&'a MpvNode> {
    let MpvNode::NodeMap(list) = src else {
        return None;
    };

    let mut i = 0;
    while i < list.values.len() {
        if bstr_equals0(key, &list.keys[i]) {
            return Some(&list.values[i]);
        }
        i += 1;
    }

    None
}

/// The entry loop of `equal_mpv_value` for arrays and maps.
fn equal_lists(l_a: &MpvNodeList, l_b: &MpvNodeList, is_map: bool) -> bool {
    let mut n = 0;
    while n < l_a.values.len() {
        if is_map && strcmp(&l_a.keys[n], &l_b.keys[n]) != 0 {
            return false;
        }
        if !equal_mpv_node(&l_a.values[n], &l_b.values[n]) {
            return false;
        }
        n += 1;
    }
    true
}

/// `equal_mpv_value`.
///
/// The C takes two pointers to the union and the format that says which
/// member to read; here the two nodes carry their own tags and `format`
/// must agree with them. Maps compare in order, as in the C.
///
/// # Panics
///
/// If `format` is not a defined format, where the C reaches
/// `MP_ASSERT_UNREACHABLE`, or does not match the nodes, where the C would
/// read the wrong union member.
#[must_use]
pub fn equal_mpv_value(a: &MpvNode, b: &MpvNode, format: i32) -> bool {
    if format == MPV_FORMAT_NODE {
        return equal_mpv_node(a, b);
    }
    match (a, b) {
        (MpvNode::None, MpvNode::None) if format == MPV_FORMAT_NONE => true,
        (MpvNode::String(s_a), MpvNode::String(s_b))
            if format == MPV_FORMAT_STRING || format == MPV_FORMAT_OSD_STRING =>
        {
            strcmp(s_a, s_b) == 0
        }
        (MpvNode::Flag(f_a), MpvNode::Flag(f_b)) if format == MPV_FORMAT_FLAG => f_a == f_b,
        (MpvNode::Int64(i_a), MpvNode::Int64(i_b)) if format == MPV_FORMAT_INT64 => i_a == i_b,
        (MpvNode::Double(d_a), MpvNode::Double(d_b)) if format == MPV_FORMAT_DOUBLE => {
            double_eq(*d_a, *d_b)
        }
        (MpvNode::ByteArray(a_r), MpvNode::ByteArray(b_r)) if format == MPV_FORMAT_BYTE_ARRAY => {
            if a_r.len() != b_r.len() {
                return false;
            }
            memcmp(a_r, b_r, a_r.len()) == 0
        }
        (MpvNode::NodeArray(l_a), MpvNode::NodeArray(l_b)) if format == MPV_FORMAT_NODE_ARRAY => {
            if l_a.values.len() != l_b.values.len() {
                return false;
            }
            equal_lists(l_a, l_b, false)
        }
        (MpvNode::NodeMap(l_a), MpvNode::NodeMap(l_b)) if format == MPV_FORMAT_NODE_MAP => {
            if l_a.values.len() != l_b.values.len() {
                return false;
            }
            equal_lists(l_a, l_b, true)
        }
        _ => unreachable!("format does not match the nodes"),
    }
}

/// `equal_mpv_node`.
#[must_use]
pub fn equal_mpv_node(a: &MpvNode, b: &MpvNode) -> bool {
    if node_format(a) != node_format(b) {
        return false;
    }
    equal_mpv_value(a, b, node_format(a))
}
