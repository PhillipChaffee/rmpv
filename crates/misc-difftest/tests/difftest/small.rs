//! `natural_sort.c`, `random.c`, `codepoint_width.c`, `hash.c`,
//! `language.c` and `charset_conv.c`.

use std::ffi::c_char;
use std::ptr::null_mut;

use rmpv_misc::{charset_conv, codepoint_width, hash, language, natural_sort, random};
use rmpv_misc_difftest::ffi::{self, Bstr};
use rmpv_misc_difftest::input::{self, Rng, cstring};
use rmpv_misc_difftest::{c, c_view, copy_cstr, copy_out, r_view, same_view, sign};

use crate::{N, run};

fn name(rng: &mut Rng) -> Vec<u8> {
    let mut s = Vec::new();
    for _ in 0..rng.below(6) {
        match rng.below(5) {
            0 | 1 => {
                s.extend(std::iter::repeat_n(b'0', rng.below(4)));
                for _ in 0..rng.below(5) {
                    s.push(rng.pick(b"0123456789"));
                }
            }
            2 => s.push(rng.pick(b"aAbBzZ._- ")),
            3 => s.push(input::byte(rng)),
            _ => s.push(rng.pick(b"abAB")),
        }
    }
    s
}

#[test]
fn mp_natural_sort_cmp() {
    run("mp_natural_sort_cmp", 100, N, |rng| {
        let a = name(rng);
        let b = if rng.chance(2) {
            input::near(rng, &a)
        } else {
            name(rng)
        };
        let (a0, b0) = (cstring(&a), cstring(&b));
        let want = c!(mp_natural_sort_cmp(a0.as_ptr().cast(), b0.as_ptr().cast()));
        assert_eq!(
            want,
            natural_sort::mp_natural_sort_cmp(&a, &b),
            "{a:?} {b:?}"
        );
    });
}

fn seed(rng: &mut Rng) -> u64 {
    match rng.below(4) {
        0 => rng.pick(&[1, 2, u64::MAX, 1 << 63, 42]),
        _ => rng.next_u64() | 1,
    }
}

#[test]
fn rotl_u64() {
    run("rotl_u64", 101, N, |rng| {
        // k = 0 and k = 64 shift by 64, which C leaves undefined.
        let (x, k) = (rng.next_u64(), 1 + rng.below(63) as i32);
        assert_eq!(c!(dt_rotl_u64(x, k)), random::rotl_u64(x, k), "{x} {k}");
    });
}

#[test]
fn splitmix64() {
    run("splitmix64", 102, N, |rng| {
        let x = if rng.chance(4) {
            rng.pick(&[0, u64::MAX, 1])
        } else {
            rng.next_u64()
        };
        let (mut cx, mut rx) = (x, x);
        assert_eq!(
            c!(dt_splitmix64(&raw mut cx)),
            random::splitmix64(&mut rx),
            "{x}"
        );
        assert_eq!(cx, rx, "{x}");
    });
}

#[test]
fn mp_rand_seed() {
    run("mp_rand_seed", 103, N, |rng| {
        let s = seed(rng);
        assert_eq!(c!(mp_rand_seed(s)).v, random::mp_rand_seed(s).v, "{s}");
    });
    // Seed 0 asks the system for the state. With libavutil's source of
    // random bytes replaced by fixed ones, the C must return them unchanged;
    // the port's own system source cannot be fixed, so only its shape is
    // checked.
    let fixed: Vec<u8> = (1..=32).collect();
    c!(dt_fix_random_bytes(fixed.as_ptr(), 1));
    let st = c!(mp_rand_seed(0));
    assert_eq!(st.v[0].to_ne_bytes(), fixed[..8]);
    assert_ne!(random::mp_rand_seed(0).v, random::mp_rand_seed(0).v);
}

#[test]
fn mp_rand_next() {
    run("mp_rand_next", 104, N, |rng| {
        let v = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];
        let mut cs = ffi::MpRandState { v };
        let mut rs = random::MpRandState { v };
        for _ in 0..3 {
            assert_eq!(
                c!(mp_rand_next(&raw mut cs)),
                random::mp_rand_next(&mut rs),
                "{v:?}"
            );
            assert_eq!(cs.v, rs.v, "{v:?}");
        }
    });
}

#[test]
fn mp_rand_next_double() {
    run("mp_rand_next_double", 105, N, |rng| {
        let v = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];
        let mut cs = ffi::MpRandState { v };
        let mut rs = random::MpRandState { v };
        let want = c!(mp_rand_next_double(&raw mut cs));
        assert_eq!(
            want.to_bits(),
            random::mp_rand_next_double(&mut rs).to_bits(),
            "{v:?}"
        );
        assert_eq!(cs.v, rs.v, "{v:?}");
    });
}

#[test]
fn mp_rand_in_range32() {
    run("mp_rand_in_range32", 106, N, |rng| {
        let v = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];
        // min < max is asserted by the C.
        let (a, b) = match rng.below(4) {
            0 => (rng.next_u64() as u32, rng.next_u64() as u32),
            1 => (0, rng.pick(&[1, 2, 3, u32::MAX, 1 << 31, (1 << 31) + 1])),
            2 => (u32::MAX - 1 - rng.below(3) as u32, u32::MAX),
            _ => (rng.below(100) as u32, 100 + rng.below(1000) as u32),
        };
        if a == b {
            return;
        }
        let (min, max) = (a.min(b), a.max(b));
        let mut cs = ffi::MpRandState { v };
        let mut rs = random::MpRandState { v };
        assert_eq!(
            c!(mp_rand_in_range32(&raw mut cs, min, max)),
            random::mp_rand_in_range32(&mut rs, min, max),
            "{v:?} {min} {max}"
        );
        assert_eq!(cs.v, rs.v, "{v:?}");
    });
}

#[test]
fn ucd_statics() {
    // Every code point: ucdLookup's whole valid domain.
    let mut cp = 0_u32;
    run("ucdLookup", 0, 0x11_0000, |_| {
        assert_eq!(
            c!(dt_ucd_lookup(cp)),
            codepoint_width::ucd_lookup(cp),
            "{cp:#x}"
        );
        cp += 1;
    });
    run("ucdGraphemeJoins", 107, N, |rng| {
        // The table has rows for states 0 and 1 only.
        let (state, lead, trail) = (
            rng.below(2) as i32,
            rng.next_u64() as i32,
            rng.next_u64() as i32,
        );
        assert_eq!(
            c!(dt_ucd_grapheme_joins(state, lead, trail)),
            codepoint_width::ucd_grapheme_joins(state, lead, trail)
        );
    });
    run("ucdGraphemeDone", 108, N, |rng| {
        let state = rng.int();
        assert_eq!(
            c!(dt_ucd_grapheme_done(state)),
            codepoint_width::ucd_grapheme_done(state)
        );
    });
    run("ucdToCharacterWidth", 109, N, |rng| {
        let val = rng.int();
        assert_eq!(
            c!(dt_ucd_to_character_width(val)),
            codepoint_width::ucd_to_character_width(val)
        );
    });
}

#[test]
fn is_valid_partial_sequence() {
    run("is_valid_partial_sequence", 110, N, |rng| {
        let mut s = input::bytes(rng, 4);
        if !s.is_empty() && rng.chance(2) {
            s[0] = rng.pick(&[0xE0, 0xED, 0xF0, 0xF4, 0xE1]);
        }
        assert_eq!(
            c!(dt_is_valid_partial_sequence(Bstr::of(&s))),
            codepoint_width::is_valid_partial_sequence(&s),
            "{s:?}"
        );
    });
}

#[test]
fn term_disp_width() {
    run("term_disp_width", 111, N, |rng| {
        let mut s = input::utf8ish(rng, 10);
        if rng.chance(4) {
            s.extend_from_slice(&input::bytes(rng, 6));
        }
        let max_width = match rng.below(4) {
            0 => rng.int(),
            1 => i32::MAX,
            _ => rng.below(14) as i32 - 1,
        };
        let mut cut: *const u8 = std::ptr::null();
        let want = c!(term_disp_width(Bstr::of(&s), max_width, &raw mut cut));
        // usize::MAX stands for "not written".
        let (got, r_cut) = codepoint_width::term_disp_width(&s, max_width, usize::MAX);
        assert_eq!(want, got, "{s:?} {max_width}");
        let c_cut = if cut.is_null() {
            usize::MAX
        } else {
            cut as usize - s.as_ptr() as usize
        };
        assert_eq!(c_cut, r_cut, "{s:?} {max_width}");
    });
}

#[test]
fn mp_hash_to_bstr() {
    // 20,000 inputs: each allocates a hash context in libavutil and the
    // inputs are up to 300 bytes, to cross several 64-byte blocks.
    run("mp_hash_to_bstr", 112, 20_000, |rng| {
        let len = match rng.below(4) {
            0 => rng.pick(&[0, 1, 55, 56, 57, 63, 64, 65, 119, 120, 128]),
            1 => rng.below(300),
            _ => rng.below(70),
        };
        let data: Vec<u8> = (0..len).map(|_| rng.next_u64() as u8).collect();
        let alg: &[u8] = if rng.chance(2) { b"MD5" } else { b"SHA256" };
        let alg0 = cstring(alg);
        let ret = c!(mp_hash_to_bstr(
            null_mut(),
            data.as_ptr(),
            data.len(),
            alg0.as_ptr().cast()
        ));
        assert_eq!(
            copy_out(ret.start, ret.len),
            hash::mp_hash_to_bstr(&data, alg),
            "{data:?}"
        );
        c!(ta_free(ret.start.cast()));
    });
}

const CODES: [&[u8]; 24] = [
    b"en", b"eng", b"de", b"ger", b"deu", b"fr", b"fre", b"fra", b"pt", b"por", b"zh", b"chi",
    b"zho", b"ja", b"jpn", b"EN", b"Ger", b"xx", b"x", b"", b"BR", b"hans", b"US", b"e\0n",
];

fn lang_tag(rng: &mut Rng) -> Vec<u8> {
    let mut s = Vec::new();
    for i in 0..rng.below(4) {
        if i > 0 || rng.chance(8) {
            s.push(b'-');
            if rng.chance(8) {
                s.push(b'-');
            }
        }
        if rng.chance(6) {
            s.extend_from_slice(&input::bytes(rng, 3));
        } else {
            s.extend_from_slice(rng.pick(&CODES));
        }
    }
    s
}

#[test]
fn lang_compare_and_canonicalize() {
    let len = c!(dt_langmap_len());
    run("lang_compare", 113, N, |rng| {
        let key = if rng.chance(3) {
            input::bytes(rng, 4)
        } else {
            rng.pick(&CODES).to_vec()
        };
        let index = rng.below(len as usize);
        let want = c!(dt_lang_compare(Bstr::of(&key), index as i32));
        assert_eq!(
            sign(want),
            sign(language::lang_compare(&key, index)),
            "{key:?} {index}"
        );
    });
    run("canonicalize", 114, N, |rng| {
        let key = match rng.below(3) {
            0 => input::bytes(rng, 4),
            1 => rng.pick(&CODES).to_vec(),
            _ => vec![
                b'a' + rng.below(26) as u8,
                rng.pick(b"abcdefghijklmnopqrstuvwxyzAZ"),
                b'a' + rng.below(26) as u8,
            ][..2 + rng.below(2)]
                .to_vec(),
        };
        let ret = c!(dt_canonicalize(Bstr::of(&key)));
        let got = language::canonicalize(&key);
        assert_eq!(copy_out(ret.start, ret.len), got, "{key:?}");
        // Either both return the key itself or both return a table entry.
        assert_eq!(
            ret.start.cast_const() == key.as_ptr(),
            got.as_ptr() == key.as_ptr() || (key.is_empty() && got.is_empty()),
            "{key:?}"
        );
    });
}

#[test]
fn mp_match_lang() {
    run("mp_match_lang", 115, N, |rng| {
        let lang = lang_tag(rng);
        let langs: Vec<Vec<u8>> = (0..rng.below(4)).map(|_| lang_tag(rng)).collect();
        let owned: Vec<Vec<u8>> = langs.iter().map(|s| cstring(s)).collect();
        let mut ptrs: Vec<*mut c_char> =
            owned.iter().map(|s| s.as_ptr().cast_mut().cast()).collect();
        ptrs.push(null_mut());
        let lang0 = cstring(&lang);
        let want = c!(mp_match_lang(ptrs.as_mut_ptr(), lang0.as_ptr().cast()));
        let refs: Vec<&[u8]> = langs.iter().map(Vec::as_slice).collect();
        assert_eq!(
            want,
            language::mp_match_lang(&refs, &lang),
            "{lang:?} {langs:?}"
        );
    });
}

fn subtitle_name(rng: &mut Rng) -> Vec<u8> {
    let mut s = input::text(rng, 4);
    let delim = rng.pick(&[
        (b'.', 0_u8),
        (b'(', b')'),
        (b'[', b']'),
        (b'.', 0),
        (b'-', 0),
    ]);
    for _ in 0..rng.below(5) {
        s.push(delim.0);
        match rng.below(8) {
            0 => s.extend_from_slice(rng.pick(&[
                &b"sdh"[..],
                b"hi",
                b"cc",
                b"forced",
                b"default",
                b"SDH",
                b"Forced",
            ])),
            1 => s.extend_from_slice(&input::bytes(rng, 4)),
            2 => {
                s.extend_from_slice(rng.pick(&CODES));
                s.push(b'-');
                s.extend_from_slice(rng.pick(&[
                    &b"BR"[..],
                    b"hans",
                    b"abcdefgh",
                    b"abcdefghi",
                    b"",
                    b"a",
                ]));
            }
            _ => s.extend_from_slice(rng.pick(&CODES)),
        }
        if delim.1 != 0 && !rng.chance(8) {
            s.push(delim.1);
        }
    }
    if rng.chance(2) {
        s.extend_from_slice(rng.pick(&[
            &b".srt"[..],
            b".ass",
            b".",
            b" ",
            b".x.y",
            b")",
            b"]",
            b")]",
        ]));
    }
    s
}

#[test]
fn mp_guess_lang_from_filename() {
    run("mp_guess_lang_from_filename", 116, N, |rng| {
        let s = subtitle_name(rng);
        let (mut start, mut flags) = (0, 0);
        let ret = c!(mp_guess_lang_from_filename(
            Bstr::of(&s),
            &raw mut start,
            &raw mut flags
        ));
        let (got, r_start, r_flags) = language::mp_guess_lang_from_filename(&s);
        let text = String::from_utf8_lossy(&s);
        assert!(same_view(c_view(&s, ret), r_view(&s, got)), "{text:?}");
        assert_eq!((start, flags), (r_start, r_flags), "{text:?}");
    });
}

fn charset_name(rng: &mut Rng) -> Vec<u8> {
    match rng.below(3) {
        0 => input::text(rng, 6),
        _ => rng
            .pick(&[
                &b"utf8"[..],
                b"UTF-8",
                b"Utf-8",
                b"utf-8x",
                b"utf",
                b"utf16",
                b"UTF-16LE",
                b"utf-16",
                b"utf-1",
                b"auto",
                b"AUTO",
                b"+utf-8",
                b"+",
                b"",
                b"latin1",
                b"ASCII",
                b"UTF-8-BROKEN",
                b"EUC-KR",
                b"cp1252",
                b"utf8\0x",
            ])
            .to_vec(),
    }
}

fn charset_data(rng: &mut Rng) -> Vec<u8> {
    let mut s = Vec::new();
    if rng.chance(4) {
        s.extend_from_slice(rng.pick(&[
            &b"\xEF\xBB\xBF"[..],
            b"\xFF\xFE",
            b"\xFE\xFF",
            b"\xEF\xBB",
            b"\xFF",
        ]));
    }
    s.extend_from_slice(&if rng.chance(3) {
        input::bytes(rng, 8)
    } else {
        input::utf8ish(rng, 6)
    });
    s
}

#[test]
fn charset_conv() {
    run("mp_charset_is_utf8", 117, N, |rng| {
        let cp = charset_name(rng);
        let cp0 = cstring(&cp);
        assert_eq!(
            c!(mp_charset_is_utf8(cp0.as_ptr().cast())),
            charset_conv::mp_charset_is_utf8(&cp),
            "{cp:?}"
        );
    });
    run("mp_charset_is_utf16", 118, N, |rng| {
        let cp = charset_name(rng);
        let cp0 = cstring(&cp);
        assert_eq!(
            c!(mp_charset_is_utf16(cp0.as_ptr().cast())),
            charset_conv::mp_charset_is_utf16(&cp),
            "{cp:?}"
        );
    });
    run("ms_bom_guess", 119, N, |rng| {
        let s = charset_data(rng);
        let ret = c!(dt_ms_bom_guess(Bstr::of(&s)));
        let want = if ret.is_null() {
            None
        } else {
            Some(copy_cstr(ret))
        };
        assert_eq!(want.as_deref(), charset_conv::ms_bom_guess(&s), "{s:?}");
    });
    run("mp_charset_guess", 120, N, |rng| {
        let s = charset_data(rng);
        let cp = charset_name(rng);
        let cp0 = cstring(&cp);
        let flags = rng.below(16) as i32;
        let mut out = [0_u8; 64];
        let where_c = c!(dt_charset_guess(
            Bstr::of(&s),
            cp0.as_ptr().cast(),
            flags,
            out.as_mut_ptr().cast(),
            out.len()
        ));
        let got = charset_conv::mp_charset_guess(&s, &cp, flags);
        assert_eq!(copy_cstr(out.as_ptr().cast()), got, "{s:?} {cp:?} {flags}");
        // Whether the result is (part of) user_cp or a string of mpv's own.
        let where_r =
            if cp.as_ptr_range().contains(&got.as_ptr()) || got.as_ptr() == cp.as_ptr_range().end {
                got.as_ptr() as isize - cp.as_ptr() as isize
            } else {
                -1
            };
        assert_eq!(where_c, where_r, "{s:?} {cp:?} {flags}");
    });
    run("mp_iconv_to_utf8", 121, N, |rng| {
        let s = charset_data(rng);
        let cp = charset_name(rng);
        let cp0 = cstring(&cp);
        let flags = rng.below(16) as i32;
        let ret = c!(mp_iconv_to_utf8(
            c!(dt_null_log()),
            Bstr::of(&s),
            cp0.as_ptr().cast(),
            flags
        ));
        let got = charset_conv::mp_iconv_to_utf8(&s, &cp, flags);
        if ret.start.cast_const() == s.as_ptr() {
            assert_eq!(got, None, "{s:?} {cp:?} {flags}");
        } else {
            assert_eq!(
                Some(copy_out(ret.start, ret.len)),
                got,
                "{s:?} {cp:?} {flags}"
            );
            c!(ta_free(ret.start.cast()));
        }
    });
}
