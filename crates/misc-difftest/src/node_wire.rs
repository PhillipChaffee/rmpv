//! The byte format `shim/dt_node.h` uses for `mpv_node` trees, for the port's
//! [`MpvNode`].

use rmpv_misc::node::{MpvNode, MpvNodeList, node_format};

use crate::input::{self, Rng};

fn put_bytes(out: &mut Vec<u8>, s: &[u8]) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s);
}

fn put_list(out: &mut Vec<u8>, list: &MpvNodeList, map: bool) {
    out.extend_from_slice(&(list.values.len() as u32).to_le_bytes());
    for (i, value) in list.values.iter().enumerate() {
        if map {
            put_bytes(out, &list.keys[i]);
        }
        dump(out, value);
    }
}

/// Appends the wire form of `node`.
pub fn dump(out: &mut Vec<u8>, node: &MpvNode) {
    out.push(node_format(node) as u8);
    match node {
        MpvNode::None => {}
        MpvNode::String(s) | MpvNode::ByteArray(s) => put_bytes(out, s),
        MpvNode::Flag(v) => out.extend_from_slice(&v.to_le_bytes()),
        MpvNode::Int64(v) => out.extend_from_slice(&v.to_le_bytes()),
        MpvNode::Double(v) => out.extend_from_slice(&v.to_le_bytes()),
        MpvNode::NodeArray(list) => put_list(out, list, false),
        MpvNode::NodeMap(list) => put_list(out, list, true),
    }
}

/// The wire form of `node`.
#[must_use]
pub fn to_wire(node: &MpvNode) -> Vec<u8> {
    let mut out = Vec::new();
    dump(&mut out, node);
    out
}

/// A random tree at most `depth` levels deep. Strings hold no NUL, as in C.
/// `byte_arrays` allows byte-array leaves below the top level.
pub fn random_node(rng: &mut Rng, depth: u32, byte_arrays: bool) -> MpvNode {
    let leaf_kinds = if byte_arrays { 6 } else { 5 };
    let kind = if depth == 0 {
        rng.below(leaf_kinds)
    } else {
        rng.below(leaf_kinds + 3)
    };
    match kind {
        0 => MpvNode::None,
        1 => MpvNode::String(input::text(rng, 8)),
        2 => MpvNode::Flag(rng.pick(&[0, 1, 1, 0, 2, -1])),
        3 => MpvNode::Int64(match rng.below(4) {
            0 => rng.pick(&[i64::MIN, i64::MAX, 0, -1, 1]),
            1 => rng.next_u64() as i64,
            _ => rng.below(2000) as i64 - 1000,
        }),
        4 => MpvNode::Double(random_double(rng)),
        5 if byte_arrays => MpvNode::ByteArray(input::bytes(rng, 6)),
        k => {
            let mut list = MpvNodeList::default();
            let map = k % 2 == 0;
            for _ in 0..rng.below(4) {
                if map {
                    list.keys.push(input::text(rng, 5));
                }
                list.values.push(random_node(rng, depth - 1, byte_arrays));
            }
            if map {
                MpvNode::NodeMap(list)
            } else {
                MpvNode::NodeArray(list)
            }
        }
    }
}

/// The bits of a `double` that is often a special value.
pub fn random_double(rng: &mut Rng) -> u64 {
    match rng.below(8) {
        0 => rng.pick(&[
            0.0_f64.to_bits(),
            (-0.0_f64).to_bits(),
            f64::INFINITY.to_bits(),
            f64::NEG_INFINITY.to_bits(),
            f64::NAN.to_bits(),
            f64::NAN.to_bits() | (1 << 63),
            0x7ff0_0000_0000_0001,
            f64::MAX.to_bits(),
            f64::MIN_POSITIVE.to_bits(),
            1,
        ]),
        1 | 2 => rng.next_u64(),
        3 => (rng.below(2000) as f64 / 8.0 - 100.0).to_bits(),
        4 => (rng.next_u64() as f64 / 1e3).to_bits(),
        _ => f64::from_bits(rng.next_u64()).rem_euclid(1e6).to_bits(),
    }
}
