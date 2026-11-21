//! `misc/bstr.c` and the inline helpers of `misc/bstr.h` and `misc/ctype.h`.

use std::ffi::c_char;
use std::ptr::null_mut;

use rmpv_misc::{bstr as port, common, ctype};
use rmpv_misc_difftest::ffi::Bstr;
use rmpv_misc_difftest::input::{self, Rng, cstring};
use rmpv_misc_difftest::{c, c_view, copy_cstr, copy_out, r_view, same_view, sign};

use crate::{N, run};

fn pair(rng: &mut Rng) -> (Vec<u8>, Vec<u8>) {
    let a = input::bytes(rng, 12);
    let b = input::near(rng, &a);
    (a, b)
}

#[test]
fn bstrcmp() {
    run("bstrcmp", 1, N, |rng| {
        let (a, b) = pair(rng);
        let want = c!(bstrcmp(Bstr::of(&a), Bstr::of(&b)));
        assert_eq!(sign(want), sign(port::bstrcmp(&a, &b)), "{a:?} {b:?}");
    });
}

#[test]
fn bstrcasecmp() {
    run("bstrcasecmp", 2, N, |rng| {
        let (a, b) = pair(rng);
        let want = c!(bstrcasecmp(Bstr::of(&a), Bstr::of(&b)));
        assert_eq!(sign(want), sign(port::bstrcasecmp(&a, &b)), "{a:?} {b:?}");
    });
}

fn needle_char(rng: &mut Rng, s: &[u8]) -> i32 {
    match rng.below(4) {
        0 if !s.is_empty() => i32::from(s[rng.below(s.len())]),
        // memchr keeps only the low byte of the int.
        1 if !s.is_empty() => i32::from(s[rng.below(s.len())]) + 256 * (rng.below(7) as i32 - 3),
        2 => rng.int(),
        _ => i32::from(input::byte(rng)),
    }
}

#[test]
fn bstrchr() {
    run("bstrchr", 3, N, |rng| {
        let s = input::bytes(rng, 12);
        let ch = needle_char(rng, &s);
        assert_eq!(
            c!(bstrchr(Bstr::of(&s), ch)),
            port::bstrchr(&s, ch),
            "{s:?} {ch}"
        );
    });
}

#[test]
fn bstrrchr() {
    run("bstrrchr", 4, N, |rng| {
        let s = input::bytes(rng, 12);
        let ch = needle_char(rng, &s);
        assert_eq!(
            c!(bstrrchr(Bstr::of(&s), ch)),
            port::bstrrchr(&s, ch),
            "{s:?} {ch}"
        );
    });
}

fn char_set(rng: &mut Rng, s: &[u8]) -> Vec<u8> {
    let mut set = input::part_of(rng, s);
    set.extend_from_slice(&input::bytes(rng, 2));
    set
}

#[test]
fn bstrcspn() {
    run("bstrcspn", 5, N, |rng| {
        let s = input::bytes(rng, 12);
        let set = char_set(rng, &s);
        let set0 = cstring(&set);
        assert_eq!(
            c!(bstrcspn(Bstr::of(&s), set0.as_ptr().cast())),
            port::bstrcspn(&s, &set),
            "{s:?} {set:?}"
        );
    });
}

#[test]
fn bstrspn() {
    run("bstrspn", 6, N, |rng| {
        let s = input::bytes(rng, 12);
        let set = char_set(rng, &s);
        let set0 = cstring(&set);
        assert_eq!(
            c!(bstrspn(Bstr::of(&s), set0.as_ptr().cast())),
            port::bstrspn(&s, &set),
            "{s:?} {set:?}"
        );
    });
}

#[test]
fn bstr_find() {
    run("bstr_find", 7, N, |rng| {
        let h = input::bytes(rng, 16);
        let n = input::part_of(rng, &h);
        assert_eq!(
            c!(bstr_find(Bstr::of(&h), Bstr::of(&n))),
            port::bstr_find(&h, &n),
            "{h:?} {n:?}"
        );
    });
}

fn spaced(rng: &mut Rng) -> Vec<u8> {
    let mut s = Vec::new();
    for _ in 0..rng.below(4) {
        s.push(rng.pick(b" \t\n\r\x0b\x0c\0\x1c\xa0"));
    }
    s.extend_from_slice(&input::bytes(rng, 8));
    for _ in 0..rng.below(4) {
        s.push(rng.pick(b" \t\n\r\x0b\x0c\0\x85"));
    }
    s
}

#[test]
fn bstr_lstrip() {
    run("bstr_lstrip", 8, N, |rng| {
        let s = spaced(rng);
        let want = c_view(&s, c!(bstr_lstrip(Bstr::of(&s))));
        assert!(same_view(want, r_view(&s, port::bstr_lstrip(&s))), "{s:?}");
    });
}

#[test]
fn bstr_strip() {
    run("bstr_strip", 9, N, |rng| {
        let s = spaced(rng);
        let want = c_view(&s, c!(bstr_strip(Bstr::of(&s))));
        assert!(same_view(want, r_view(&s, port::bstr_strip(&s))), "{s:?}");
    });
}

#[test]
fn bstr_split() {
    run("bstr_split", 10, N, |rng| {
        let s = input::bytes(rng, 14);
        let sep = char_set(rng, &s);
        let sep0 = cstring(&sep);
        let mut rest = Bstr::NULL;
        let ret = c!(bstr_split(
            Bstr::of(&s),
            sep0.as_ptr().cast(),
            &raw mut rest
        ));
        let (r_ret, r_rest) = port::bstr_split(&s, &sep);
        assert!(
            same_view(c_view(&s, ret), r_view(&s, r_ret)),
            "{s:?} {sep:?}"
        );
        assert!(
            same_view(c_view(&s, rest), r_view(&s, r_rest)),
            "{s:?} {sep:?}"
        );
    });
}

#[test]
fn bstr_split_tok() {
    run("bstr_split_tok", 11, N, |rng| {
        let s = input::bytes(rng, 14);
        let tok = input::part_of(rng, &s);
        let tok0 = cstring(&tok);
        let (mut left, mut right) = (Bstr::NULL, Bstr::NULL);
        let found = c!(bstr_split_tok(
            Bstr::of(&s),
            tok0.as_ptr().cast(),
            &raw mut left,
            &raw mut right
        ));
        let (r_found, r_left, r_right) = port::bstr_split_tok(&s, &tok);
        assert_eq!(found, r_found, "{s:?} {tok:?}");
        assert!(
            same_view(c_view(&s, left), r_view(&s, r_left)),
            "{s:?} {tok:?}"
        );
        assert!(
            same_view(c_view(&s, right), r_view(&s, r_right)),
            "{s:?} {tok:?}"
        );
    });
}

#[test]
fn bstr_splice() {
    run("bstr_splice", 12, N, |rng| {
        let s = input::bytes(rng, 10);
        let (start, end) = (rng.int(), rng.int());
        let want = c_view(&s, c!(bstr_splice(Bstr::of(&s), start, end)));
        let got = r_view(&s, port::bstr_splice(&s, start, end));
        let len = s.len() as i64;
        let from = if start < 0 {
            i64::from(start) + len
        } else {
            i64::from(start)
        };
        if !s.is_empty() && from > len {
            // The C returns a pointer past the end of the buffer with length
            // 0. Only the length can be compared.
            assert_eq!(want.len, got.len, "{s:?} {start} {end}");
        } else {
            assert!(
                same_view(want, got),
                "{s:?} {start} {end}: {want:?} {got:?}"
            );
        }
    });
}

fn base(rng: &mut Rng) -> i32 {
    match rng.below(4) {
        0 => rng.pick(&[0, 2, 8, 10, 16, 36]),
        1 => 2 + rng.below(35) as i32,
        _ => rng.pick(&[0, 10, 16]),
    }
}

#[test]
fn bstrtoll() {
    run("bstrtoll", 13, N, |rng| {
        let s = input::integer_text(rng);
        let base = base(rng);
        let mut rest = Bstr::NULL;
        let want = c!(bstrtoll(Bstr::of(&s), &raw mut rest, base));
        let (got, r_rest) = port::bstrtoll(&s, base);
        assert_eq!(want, got, "{:?} base {base}", String::from_utf8_lossy(&s));
        assert!(
            same_view(c_view(&s, rest), r_view(&s, r_rest)),
            "{:?} base {base}",
            String::from_utf8_lossy(&s)
        );
    });
}

#[test]
fn bstrtod() {
    run("bstrtod", 14, N, |rng| {
        let s = input::float_text(rng);
        let mut rest = Bstr::NULL;
        let want = c!(bstrtod(Bstr::of(&s), &raw mut rest));
        let (got, r_rest) = port::bstrtod(&s);
        assert_eq!(
            want.to_bits(),
            got.to_bits(),
            "{:?}: {want:e} {got:e}",
            String::from_utf8_lossy(&s)
        );
        assert!(
            same_view(c_view(&s, rest), r_view(&s, r_rest)),
            "{:?}",
            String::from_utf8_lossy(&s)
        );
    });
}

#[test]
fn bstr_splitchar() {
    run("bstr_splitchar", 15, N, |rng| {
        let s = input::bytes(rng, 12);
        let ch = if s.is_empty() || rng.chance(3) {
            input::byte(rng)
        } else {
            s[rng.below(s.len())]
        };
        let mut rest = Bstr::NULL;
        let ret = c!(bstr_splitchar(Bstr::of(&s), &raw mut rest, ch as c_char));
        let (r_ret, r_rest) = port::bstr_splitchar(&s, ch);
        assert!(same_view(c_view(&s, ret), r_view(&s, r_ret)), "{s:?} {ch}");
        assert!(
            same_view(c_view(&s, rest), r_view(&s, r_rest)),
            "{s:?} {ch}"
        );
    });
}

#[test]
fn bstr_getline() {
    run("bstr_getline", 16, N, |rng| {
        let s = input::bytes(rng, 12);
        let mut rest = Bstr::NULL;
        let ret = c!(dt_bstr_getline(Bstr::of(&s), &raw mut rest));
        let (r_ret, r_rest) = port::bstr_getline(&s);
        assert!(same_view(c_view(&s, ret), r_view(&s, r_ret)), "{s:?}");
        assert!(same_view(c_view(&s, rest), r_view(&s, r_rest)), "{s:?}");
    });
}

#[test]
fn bstr_strip_linebreaks() {
    run("bstr_strip_linebreaks", 17, N, |rng| {
        let mut s = input::bytes(rng, 6);
        s.extend_from_slice(rng.pick(&[
            &b""[..],
            b"\n",
            b"\r\n",
            b"\r",
            b"\n\r",
            b"\n\n",
            b"\r\r\n",
        ]));
        let want = c_view(&s, c!(bstr_strip_linebreaks(Bstr::of(&s))));
        assert!(
            same_view(want, r_view(&s, port::bstr_strip_linebreaks(&s))),
            "{s:?}"
        );
    });
}

fn affix(rng: &mut Rng, s: &[u8]) -> Vec<u8> {
    match rng.below(4) {
        0 => s[..rng.below(s.len() + 1)].to_vec(),
        1 => s[rng.below(s.len() + 1)..].to_vec(),
        2 => input::near(rng, s),
        _ => input::part_of(rng, s),
    }
}

#[test]
fn bstr_eatstart() {
    run("bstr_eatstart", 18, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        let mut cs = Bstr::of(&s);
        let want = c!(bstr_eatstart(&raw mut cs, Bstr::of(&p)));
        let (got, rs) = port::bstr_eatstart(&s, &p);
        assert_eq!(want, got, "{s:?} {p:?}");
        assert!(same_view(c_view(&s, cs), r_view(&s, rs)), "{s:?} {p:?}");
    });
}

#[test]
fn bstr_eatend() {
    run("bstr_eatend", 19, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        let mut cs = Bstr::of(&s);
        let want = c!(bstr_eatend(&raw mut cs, Bstr::of(&p)));
        let (got, rs) = port::bstr_eatend(&s, &p);
        assert_eq!(want, got, "{s:?} {p:?}");
        assert!(same_view(c_view(&s, cs), r_view(&s, rs)), "{s:?} {p:?}");
    });
}

#[test]
fn bstr_eatstart0_eatend0() {
    run("bstr_eatstart0", 20, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        let p0 = cstring(&p);
        let mut cs = Bstr::of(&s);
        let want = c!(dt_bstr_eatstart0(&raw mut cs, p0.as_ptr().cast()));
        let (got, rs) = port::bstr_eatstart0(&s, &p);
        assert_eq!(want, got, "{s:?} {p:?}");
        assert!(same_view(c_view(&s, cs), r_view(&s, rs)), "{s:?} {p:?}");
    });
    run("bstr_eatend0", 21, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        let p0 = cstring(&p);
        let mut cs = Bstr::of(&s);
        let want = c!(dt_bstr_eatend0(&raw mut cs, p0.as_ptr().cast()));
        let (got, rs) = port::bstr_eatend0(&s, &p);
        assert_eq!(want, got, "{s:?} {p:?}");
        assert!(same_view(c_view(&s, cs), r_view(&s, rs)), "{s:?} {p:?}");
    });
}

#[test]
fn bstr_lower() {
    run("bstr_lower", 22, N, |rng| {
        let s = input::bytes(rng, 12);
        let mut want = s.clone();
        c!(bstr_lower(Bstr::of_mut(&mut want)));
        let mut got = s.clone();
        port::bstr_lower(&mut got);
        assert_eq!(want, got, "{s:?}");
    });
}

#[test]
fn bstr_parse_utf8_code_length() {
    // The domain has 256 values; all are tried.
    let mut b = 0_u8;
    run("bstr_parse_utf8_code_length", 0, 256, |_| {
        assert_eq!(
            c!(bstr_parse_utf8_code_length(b)),
            port::bstr_parse_utf8_code_length(b),
            "{b}"
        );
        b = b.wrapping_add(1);
    });
}

fn utf8_input(rng: &mut Rng) -> Vec<u8> {
    if rng.chance(3) {
        input::bytes(rng, 8)
    } else {
        input::utf8ish(rng, 5)
    }
}

#[test]
fn bstr_decode_partial_utf8() {
    run("bstr_decode_partial_utf8", 23, N, |rng| {
        let s = utf8_input(rng);
        let mut cs = Bstr::of(&s);
        let want = c!(bstr_decode_partial_utf8(&raw mut cs));
        let (got, rs) = port::bstr_decode_partial_utf8(&s);
        assert_eq!(want, got, "{s:?}");
        assert!(same_view(c_view(&s, cs), r_view(&s, rs)), "{s:?}");
    });
}

#[test]
fn bstr_decode_utf8() {
    run("bstr_decode_utf8", 24, N, |rng| {
        let s = utf8_input(rng);
        // The out-parameter starts as the whole string, to see it is left
        // alone on an error.
        let mut next = Bstr::of(&s);
        let want = c!(bstr_decode_utf8(Bstr::of(&s), &raw mut next));
        let (got, r_next) = port::bstr_decode_utf8(&s);
        assert_eq!(want, got, "{s:?}");
        assert!(same_view(c_view(&s, next), r_view(&s, r_next)), "{s:?}");
    });
}

#[test]
fn bstr_split_utf8() {
    run("bstr_split_utf8", 25, N, |rng| {
        let s = utf8_input(rng);
        let mut next = Bstr::of(&s);
        let ret = c!(bstr_split_utf8(Bstr::of(&s), &raw mut next));
        let (r_ret, r_next) = port::bstr_split_utf8(&s);
        assert!(same_view(c_view(&s, ret), r_view(&s, r_ret)), "{s:?}");
        assert!(same_view(c_view(&s, next), r_view(&s, r_next)), "{s:?}");
    });
}

#[test]
fn bstr_validate_utf8() {
    run("bstr_validate_utf8", 26, N, |rng| {
        let s = utf8_input(rng);
        assert_eq!(
            c!(bstr_validate_utf8(Bstr::of(&s))),
            port::bstr_validate_utf8(&s),
            "{s:?}"
        );
    });
}

#[test]
fn bstr_sanitize_utf8_latin1() {
    run("bstr_sanitize_utf8_latin1", 27, N, |rng| {
        let s = utf8_input(rng);
        let ret = c!(bstr_sanitize_utf8_latin1(null_mut(), Bstr::of(&s)));
        let got = port::bstr_sanitize_utf8_latin1(&s);
        if ret.start.cast_const() == s.as_ptr() {
            assert_eq!(got, None, "{s:?}");
        } else {
            assert_eq!(Some(copy_out(ret.start, ret.len)), got, "{s:?}");
            c!(ta_free(ret.start.cast()));
        }
    });
}

#[test]
fn bstr_xappend() {
    run("bstr_xappend", 28, N, |rng| {
        let mut cs = Bstr::NULL;
        let mut rs = Vec::new();
        for _ in 0..rng.below(5) {
            let piece = input::bytes(rng, 9);
            c!(bstr_xappend(null_mut(), &raw mut cs, Bstr::of(&piece)));
            port::bstr_xappend(&mut rs, &piece);
            assert_eq!(copy_out(cs.start, cs.len), rs);
        }
        c!(ta_free(cs.start.cast()));
    });
}

#[test]
fn bstr_xappend0() {
    run("bstr_xappend0", 29, N, |rng| {
        let mut cs = Bstr::NULL;
        let mut rs = Vec::new();
        for _ in 0..rng.below(4) {
            let piece = input::bytes(rng, 9);
            let piece0 = cstring(&piece);
            c!(dt_bstr_xappend0(&raw mut cs, piece0.as_ptr().cast()));
            port::bstr_xappend0(&mut rs, &piece);
            assert_eq!(copy_out(cs.start, cs.len), rs);
        }
        c!(ta_free(cs.start.cast()));
    });
}

#[test]
fn bstr_case_startswith() {
    run("bstr_case_startswith", 30, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        assert_eq!(
            c!(bstr_case_startswith(Bstr::of(&s), Bstr::of(&p))),
            port::bstr_case_startswith(&s, &p),
            "{s:?} {p:?}"
        );
    });
}

#[test]
fn bstr_case_endswith() {
    run("bstr_case_endswith", 31, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        assert_eq!(
            c!(bstr_case_endswith(Bstr::of(&s), Bstr::of(&p))),
            port::bstr_case_endswith(&s, &p),
            "{s:?} {p:?}"
        );
    });
}

#[test]
fn bstr_decode_hex() {
    run("bstr_decode_hex", 33, N, |rng| {
        let mut s = Vec::new();
        for _ in 0..rng.below(12) {
            s.push(if rng.chance(12) {
                input::byte(rng)
            } else {
                rng.pick(b"0123456789abcdefABCDEF")
            });
        }
        let mut out = Bstr::NULL;
        let ok = c!(bstr_decode_hex(null_mut(), Bstr::of(&s), &raw mut out));
        let got = port::bstr_decode_hex(&s);
        assert_eq!(ok, got.is_some(), "{s:?}");
        if ok {
            assert_eq!(Some(copy_out(out.start, out.len)), got, "{s:?}");
            c!(ta_free(out.start.cast()));
        }
    });
}

fn string_list(rng: &mut Rng, s: &[u8]) -> Vec<Vec<u8>> {
    (0..rng.below(5))
        .map(|_| {
            if rng.chance(3) {
                input::near(rng, s)
            } else {
                input::bytes(rng, 6)
            }
        })
        .collect()
}

fn with_c_list<T>(list: &[Vec<u8>], f: impl FnOnce(*mut *mut c_char) -> T) -> T {
    let owned: Vec<Vec<u8>> = list.iter().map(|s| cstring(s)).collect();
    let mut ptrs: Vec<*mut c_char> = owned.iter().map(|s| s.as_ptr().cast_mut().cast()).collect();
    ptrs.push(null_mut());
    f(ptrs.as_mut_ptr())
}

#[test]
fn bstr_find_in_list0() {
    run("bstr_find_in_list0", 34, N, |rng| {
        let s = input::bytes(rng, 6);
        let list = string_list(rng, &s);
        let case_sensitive = rng.chance(2);
        let want = with_c_list(&list, |p| {
            c!(bstr_find_in_list0(Bstr::of(&s), p, case_sensitive))
        });
        assert_eq!(
            want,
            port::bstr_find_in_list0(&s, &list, case_sensitive),
            "{s:?} {list:?}"
        );
    });
}

#[test]
fn bstr_in_list0() {
    run("bstr_in_list0", 35, N, |rng| {
        let s = input::bytes(rng, 6);
        let list = string_list(rng, &s);
        let want = with_c_list(&list, |p| c!(bstr_in_list0(Bstr::of(&s), p)));
        assert_eq!(want, port::bstr_in_list0(&s, &list), "{s:?} {list:?}");
    });
}

#[test]
fn h_to_i() {
    // The domain has 256 values; all are tried.
    let mut b = 0_u8;
    run("h_to_i", 0, 256, |_| {
        assert_eq!(c!(dt_h_to_i(b)), port::h_to_i(b), "{b}");
        b = b.wrapping_add(1);
    });
}

#[test]
fn bstr_cut() {
    run("bstr_cut", 36, N, |rng| {
        let s = input::bytes(rng, 10);
        let n = rng.int();
        let want = c_view(&s, c!(dt_bstr_cut(Bstr::of(&s), n)));
        assert!(
            same_view(want, r_view(&s, port::bstr_cut(&s, n))),
            "{s:?} {n}"
        );
    });
}

#[test]
fn bstr_dup() {
    run("bstrdup0", 37, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = c!(dt_bstrdup0(Bstr::of(&s)));
        assert_eq!(copy_cstr(p), port::bstrdup0(&s), "{s:?}");
        c!(ta_free(p.cast()));
    });
    run("bstrto0", 38, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = c!(dt_bstrto0(Bstr::of(&s)));
        assert_eq!(copy_cstr(p), port::bstrto0(&s), "{s:?}");
        c!(ta_free(p.cast()));
    });
    run("bstrdup", 39, N, |rng| {
        let s = input::bytes(rng, 10);
        let d = c!(dt_bstrdup(Bstr::of(&s)));
        assert_eq!(copy_out(d.start, d.len), port::bstrdup(&s), "{s:?}");
        c!(ta_free(d.start.cast()));
    });
    run("bstr0", 40, N, |rng| {
        let s = input::bytes(rng, 10);
        let s0 = cstring(&s);
        let v = c!(dt_bstr0(s0.as_ptr().cast()));
        assert!(
            same_view(c_view(&s0, v), r_view(&s, port::bstr0(&s))),
            "{s:?}"
        );
    });
}

#[test]
fn bstr_predicates() {
    run("bstr_startswith", 41, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        assert_eq!(
            c!(dt_bstr_startswith(Bstr::of(&s), Bstr::of(&p))),
            port::bstr_startswith(&s, &p),
            "{s:?} {p:?}"
        );
    });
    run("bstr_startswith0", 42, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        let p0 = cstring(&p);
        assert_eq!(
            c!(dt_bstr_startswith0(Bstr::of(&s), p0.as_ptr().cast())),
            port::bstr_startswith0(&s, &p),
            "{s:?} {p:?}"
        );
    });
    run("bstr_endswith", 43, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        assert_eq!(
            c!(dt_bstr_endswith(Bstr::of(&s), Bstr::of(&p))),
            port::bstr_endswith(&s, &p),
            "{s:?} {p:?}"
        );
    });
    run("bstr_endswith0", 44, N, |rng| {
        let s = input::bytes(rng, 10);
        let p = affix(rng, &s);
        let p0 = cstring(&p);
        assert_eq!(
            c!(dt_bstr_endswith0(Bstr::of(&s), p0.as_ptr().cast())),
            port::bstr_endswith0(&s, &p),
            "{s:?} {p:?}"
        );
    });
    run("bstr_equals", 45, N, |rng| {
        let (a, b) = pair(rng);
        assert_eq!(
            c!(dt_bstr_equals(Bstr::of(&a), Bstr::of(&b))),
            port::bstr_equals(&a, &b),
            "{a:?} {b:?}"
        );
    });
    run("bstr_equals0", 46, N, |rng| {
        let (a, b) = pair(rng);
        let b0 = cstring(&b);
        assert_eq!(
            c!(dt_bstr_equals0(Bstr::of(&a), b0.as_ptr().cast())),
            port::bstr_equals0(&a, &b),
            "{a:?} {b:?}"
        );
    });
    run("bstrcmp0", 47, N, |rng| {
        let (a, b) = pair(rng);
        let b0 = cstring(&b);
        assert_eq!(
            sign(c!(dt_bstrcmp0(Bstr::of(&a), b0.as_ptr().cast()))),
            sign(port::bstrcmp0(&a, &b)),
            "{a:?} {b:?}"
        );
    });
    run("bstrcasecmp0", 48, N, |rng| {
        let (a, b) = pair(rng);
        let b0 = cstring(&b);
        assert_eq!(
            sign(c!(dt_bstrcasecmp0(Bstr::of(&a), b0.as_ptr().cast()))),
            sign(port::bstrcasecmp0(&a, &b)),
            "{a:?} {b:?}"
        );
    });
    run("bstr_find0", 49, N, |rng| {
        let h = input::bytes(rng, 16);
        let n = input::part_of(rng, &h);
        let n0 = cstring(&n);
        assert_eq!(
            c!(dt_bstr_find0(Bstr::of(&h), n0.as_ptr().cast())),
            port::bstr_find0(&h, &n),
            "{h:?} {n:?}"
        );
    });
}

#[test]
fn ctype() {
    // Each helper has 256 inputs; all are tried.
    let mut b = 0_u8;
    run("misc/ctype.h (9 helpers)", 0, 256, |_| {
        let ch = b as c_char;
        assert_eq!(c!(dt_mp_isprint(ch)) != 0, ctype::mp_isprint(b));
        assert_eq!(c!(dt_mp_isspace(ch)) != 0, ctype::mp_isspace(b));
        assert_eq!(c!(dt_mp_isupper(ch)) != 0, ctype::mp_isupper(b));
        assert_eq!(c!(dt_mp_islower(ch)) != 0, ctype::mp_islower(b));
        assert_eq!(c!(dt_mp_isdigit(ch)) != 0, ctype::mp_isdigit(b));
        assert_eq!(c!(dt_mp_isalpha(ch)) != 0, ctype::mp_isalpha(b));
        assert_eq!(c!(dt_mp_isalnum(ch)) != 0, ctype::mp_isalnum(b));
        assert_eq!(c!(dt_mp_tolower(ch)) as u8, ctype::mp_tolower(b));
        assert_eq!(c!(dt_mp_toupper(ch)) as u8, ctype::mp_toupper(b));
        b = b.wrapping_add(1);
    });
}

#[test]
fn common_helpers() {
    run("mp_log2", 50, N, |rng| {
        let v = if rng.chance(2) {
            rng.next_u64() as u32
        } else {
            1_u32
                .wrapping_shl(rng.below(33) as u32)
                .wrapping_add(rng.below(3) as u32)
                .wrapping_sub(1)
        };
        assert_eq!(c!(mp_log2(v)), common::mp_log2(v), "{v}");
        assert_eq!(
            c!(mp_round_next_power_of_2(v)),
            common::mp_round_next_power_of_2(v),
            "{v}"
        );
    });
    run("mp_append_utf8_bstr", 51, N, |rng| {
        // Below 2^31: above it PUT_UTF8 shifts a 32-bit value by 36, which C
        // leaves undefined.
        let cp = match rng.below(4) {
            0 => rng.next_u64() as u32 >> 1,
            1 => 1_u32.wrapping_shl(rng.below(32) as u32).wrapping_sub(1),
            _ => rng.below(0x11_0000) as u32,
        };
        let mut cs = Bstr::NULL;
        c!(mp_append_utf8_bstr(null_mut(), &raw mut cs, cp));
        let mut rs = Vec::new();
        common::mp_append_utf8_bstr(&mut rs, cp);
        assert_eq!(copy_out(cs.start, cs.len), rs, "{cp:#x}");
        c!(ta_free(cs.start.cast()));
    });
    run("mp_append_escaped_string", 52, N, |rng| {
        let mut s = Vec::new();
        for _ in 0..rng.below(6) {
            match rng.below(6) {
                0 | 1 => {
                    s.push(b'\\');
                    s.extend_from_slice(rng.pick(&[
                        &b"n"[..],
                        b"\"",
                        b"\\",
                        b"/",
                        b"b",
                        b"f",
                        b"r",
                        b"t",
                        b"e",
                        b"'",
                        b"x41",
                        b"xff",
                        b"x-1",
                        b"x 7",
                        b"x0",
                        b"xg1",
                        b"u0041",
                        b"u00e9",
                        b"u20ac",
                        b"ud83d\\ude00",
                        b"ud83d\\u0041",
                        b"ud83d",
                        b"udc00",
                        b"u-001",
                        b"u 0x1",
                        b"u0x1f",
                        b"u+1f6",
                        b"u12",
                        b"q",
                        b"",
                        b"x00",
                        b"u0000",
                    ]));
                }
                _ => s.extend_from_slice(&input::bytes(rng, 3)),
            }
        }
        let mut dst = Bstr::NULL;
        let mut src = Bstr::of(&s);
        let ok = c!(mp_append_escaped_string(
            null_mut(),
            &raw mut dst,
            &raw mut src
        ));
        let mut r_dst = Vec::new();
        let got = common::mp_append_escaped_string(&mut r_dst, &s);
        assert_eq!(ok, got.is_some(), "{:?}", String::from_utf8_lossy(&s));
        if let Some(rest) = got {
            assert_eq!(
                copy_out(dst.start, dst.len),
                r_dst,
                "{:?}",
                String::from_utf8_lossy(&s)
            );
            assert!(
                same_view(c_view(&s, src), r_view(&s, rest)),
                "{:?}",
                String::from_utf8_lossy(&s)
            );
        }
        // For a string without escapes the C hands back a view of its input
        // instead of the copy its comment promises, unless that view starts
        // where the rest starts. Only an allocation may be freed.
        let aliases_input = s.as_ptr_range().contains(&dst.start.cast_const());
        if !aliases_input {
            c!(ta_free(dst.start.cast()));
        }
    });
}
