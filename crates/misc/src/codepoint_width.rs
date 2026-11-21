//! Port of `misc/codepoint_width.c`: terminal display width of text.

#[path = "codepoint_width_tables.rs"]
mod tables;

use crate::bstr::{
    BSTR_DECODE_TRUNCATED_SEQUENCE, bstr_cut, bstr_decode_partial_utf8, bstr_decode_utf8,
    bstr_eatstart0,
};
use tables::{S_JOIN_RULES, S_STAGE0, S_STAGE1, S_STAGE2, S_STAGE3};

/// `ucdLookup`. The first table has an entry for every `cp >> 11` up to
/// U+10FFFF; a larger argument is out of bounds in the C and panics here.
#[must_use]
pub fn ucd_lookup(cp: u32) -> i32 {
    let s0 = S_STAGE0[(cp >> 11) as usize];
    let s1 = S_STAGE1[(s0 as u32 + ((cp >> 6) & 31)) as usize];
    let s2 = S_STAGE2[(s1 as u32 + ((cp >> 3) & 7)) as usize];
    let s3 = S_STAGE3[(s2 as u32 + (cp & 7)) as usize];
    s3 as i32
}

/// `ucdGraphemeJoins`. `state` must be 0 or 1.
#[must_use]
pub fn ucd_grapheme_joins(state: i32, lead: i32, trail: i32) -> i32 {
    let l = lead & 15;
    let t = trail & 15;
    ((S_JOIN_RULES[state as usize][l as usize] >> (t * 2)) & 3) as i32
}

/// `ucdGraphemeDone`.
#[must_use]
pub fn ucd_grapheme_done(state: i32) -> bool {
    state == 3
}

/// `ucdToCharacterWidth`.
#[must_use]
pub fn ucd_to_character_width(val: i32) -> i32 {
    val >> 6
}

/// `is_valid_partial_sequence`.
#[must_use]
pub fn is_valid_partial_sequence(seq: &[u8]) -> bool {
    if seq.len() < 2 {
        return true;
    }

    if seq[0] == 0xE0 {
        return seq[1] >= 0xA0;
    } else if seq[0] == 0xED {
        return seq[1] <= 0x9F;
    } else if seq[0] == 0xF0 {
        return seq[1] >= 0x90;
    } else if seq[0] == 0xF4 {
        return seq[1] <= 0x8F;
    }
    true
}

/// The `while` loop of `term_disp_width` that skips the parameter bytes of
/// an escape sequence.
fn skip_escape_parameters(s: &[u8]) -> &[u8] {
    let mut s = s;
    while !s.is_empty() && !((s[0] >= b'@' && s[0] <= b'~') || s[0] == b'm') {
        s = bstr_cut(s, 1);
    }
    s
}

/// The grapheme-cluster loop of `term_disp_width`: the summed width of the
/// cluster that starts with `cp`, before clamping, and the string after it.
fn cluster_width(cp: i32, s: &[u8]) -> (i32, &[u8]) {
    const AMBIGUOUS_WIDTH: i32 = 1;
    let mut cp = cp;
    let mut s = s;
    let mut current_width = 0;
    let mut state = 0;
    loop {
        let lead = ucd_lookup(cp as u32);
        let mut w = ucd_to_character_width(lead);
        if w == 3 {
            w = AMBIGUOUS_WIDTH;
        }

        // U+FE0F turns an emoji wide; the caller clamps the sum to 2.
        if cp == 0xFE0F {
            w = 2;
        }

        current_width += w;

        if s.is_empty() {
            break;
        }

        let (next_cp, cluster_end) = bstr_decode_utf8(s);
        cp = next_cp;
        if cp < 0 {
            break;
        }
        let trail = ucd_lookup(cp as u32);

        state = ucd_grapheme_joins(state, lead, trail);
        if ucd_grapheme_done(state) {
            break;
        }

        s = cluster_end;
    }
    (current_width, s)
}

/// The part of the `term_disp_width` loop body before its `next:` label:
/// the width of the next item, the string after it, and whether the item was
/// a truncated sequence, which the caller may narrow to width 1. `prev_pos`
/// is updated as the C updates it, which is not on every path, and a
/// carriage return resets `width`.
fn next_item_width<'a>(
    s: &'a [u8],
    total: usize,
    width: &mut i32,
    prev_pos: &mut usize,
) -> (i32, &'a [u8], bool) {
    let (escape, s) = bstr_eatstart0(s, b"\x1b[");
    if escape {
        let s = skip_escape_parameters(s);
        return (0, bstr_cut(s, 1), false);
    }

    *prev_pos = total - s.len();
    let (cp, s) = bstr_decode_partial_utf8(s);

    // An ill-formed sequence shows as one U+FFFD per maximal subpart.
    if cp < 0 {
        let partial_start = *prev_pos;
        let partial_len = total - s.len() - partial_start;
        return (partial_len as i32, s, cp == BSTR_DECODE_TRUNCATED_SEQUENCE);
    }

    if cp == b'\r' as i32 {
        *width = 0;
        return (0, s, false);
    }

    if cp == b'\t' as i32 {
        return (8 - *width % 8, s, false);
    }

    if cp < 0x20 {
        return (0, s, false);
    }

    if cp <= 0x7E {
        return (1, s, false);
    }

    let (current_width, s) = cluster_width(cp, s);
    (if current_width > 2 { 2 } else { current_width }, s, false)
}

/// `term_disp_width`: the display width of `str`, stopping once it reaches
/// `max_width`.
///
/// `cut_pos` is the caller's current value of the C out-parameter, as an
/// offset into `s`; the returned offset is where the C would point it. It
/// comes back unchanged when the whole string fits.
///
/// # Panics
///
/// Where the C fails `mp_assert`, which no input reaches.
#[must_use]
pub fn term_disp_width(s: &[u8], max_width: i32, cut_pos: usize) -> (i32, usize) {
    let total = s.len();
    let orig = s;
    let mut s = s;
    let mut cut_pos = cut_pos;
    let mut width = 0;

    let mut prev_pos = 0;
    while !s.is_empty() {
        let (item_width, rest, truncated) = next_item_width(s, total, &mut width, &mut prev_pos);
        s = rest;
        let mut current_width = item_width;
        if truncated {
            let partial = &orig[prev_pos..total - s.len()];
            if is_valid_partial_sequence(partial) {
                current_width = 1;
            }
        }

        if width + current_width > max_width {
            assert!(prev_pos < total);
            cut_pos = prev_pos;
            break;
        }
        width += current_width;
        if width == max_width {
            if !s.is_empty() {
                cut_pos = total - s.len();
            }
            break;
        }
    }

    (width, cut_pos)
}
