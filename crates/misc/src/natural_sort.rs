//! Port of `misc/natural_sort.c`.

use crate::clib::cstr_at;
use crate::ctype::{mp_isdigit, mp_tolower};

fn skip_zeros(name: &[u8], start: usize) -> usize {
    let mut i = start;
    while cstr_at(name, i) == b'0' {
        i += 1;
    }
    i
}

fn skip_digits(name: &[u8], start: usize) -> usize {
    let mut i = start;
    while mp_isdigit(cstr_at(name, i)) {
        i += 1;
    }
    i
}

/// The C compares `char` values here. Both runs hold only ASCII digits when
/// this is reached, so the sign of `char` does not matter.
fn cmp_digit_runs(name1: &[u8], start1: usize, end1: usize, name2: &[u8], start2: usize) -> i32 {
    let mut i1 = start1;
    let mut i2 = start2;
    let mut ret = 0;
    while ret == 0 && i1 < end1 {
        let a = cstr_at(name1, i1);
        let b = cstr_at(name2, i2);
        if a < b {
            ret = -1;
        } else if a > b {
            ret = 1;
        }
        i1 += 1;
        i2 += 1;
    }
    ret
}

/// `mp_natural_sort_cmp`. Both arguments are C strings.
#[must_use]
pub fn mp_natural_sort_cmp(name1: &[u8], name2: &[u8]) -> i32 {
    let mut i1 = 0;
    let mut i2 = 0;
    while cstr_at(name1, i1) != 0 && cstr_at(name2, i2) != 0 {
        if mp_isdigit(cstr_at(name1, i1)) && mp_isdigit(cstr_at(name2, i2)) {
            i1 = skip_zeros(name1, i1);
            i2 = skip_zeros(name2, i2);
            let end1 = skip_digits(name1, i1);
            let end2 = skip_digits(name2, i2);
            if end1 - i1 < end2 - i2 {
                return -1;
            }
            if end1 - i1 > end2 - i2 {
                return 1;
            }
            let ret = cmp_digit_runs(name1, i1, end1, name2, i2);
            if ret != 0 {
                return ret;
            }
            i1 = end1;
            i2 = end2;
        } else {
            let a = mp_tolower(cstr_at(name1, i1));
            let b = mp_tolower(cstr_at(name2, i2));
            if a < b {
                return -1;
            }
            if a > b {
                return 1;
            }
            i1 += 1;
            i2 += 1;
        }
    }
    if cstr_at(name2, i2) != 0 {
        return -1;
    }
    if cstr_at(name1, i1) != 0 {
        return 1;
    }
    0
}
