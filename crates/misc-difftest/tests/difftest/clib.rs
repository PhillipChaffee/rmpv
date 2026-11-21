//! The port's stand-ins for C library routines, against glibc itself.

use std::ffi::c_char;

use rmpv_misc::clib as port;
use rmpv_misc_difftest::input::{self, cstring};
use rmpv_misc_difftest::{c, sign};

use crate::{N, run};

#[test]
fn strtoll() {
    run("libc strtoll", 60, N, |rng| {
        let s = input::integer_text(rng);
        let base = match rng.below(3) {
            0 => rng.pick(&[0, 2, 8, 10, 16, 36]),
            1 => 2 + rng.below(35) as i32,
            _ => 0,
        };
        let s0 = cstring(&s);
        let (mut consumed, mut erange) = (0, 0);
        let want = c!(dt_strtoll(
            s0.as_ptr().cast(),
            base,
            &raw mut consumed,
            &raw mut erange
        ));
        let got = port::strtoll(&s, base);
        let text = String::from_utf8_lossy(&s);
        assert_eq!(
            (want, consumed, erange != 0),
            (got.value, got.consumed, got.erange),
            "{text:?} base {base}"
        );
    });
}

#[test]
fn strtod() {
    run("libc strtod", 61, N, |rng| {
        let s = input::float_text(rng);
        let s0 = cstring(&s);
        let (mut consumed, mut erange) = (0, 0);
        let want = c!(dt_strtod(
            s0.as_ptr().cast(),
            &raw mut consumed,
            &raw mut erange
        ));
        let got = port::strtod(&s);
        let text = String::from_utf8_lossy(&s);
        assert_eq!(
            (want.to_bits(), consumed, erange != 0),
            (got.bits, got.consumed, got.erange),
            "{text:?}: {want:e} {:e}",
            f64::from_bits(got.bits)
        );
    });
}

#[test]
fn comparisons() {
    run(
        "libc memcmp, strcmp, strncmp, strcasecmp, strncasecmp (sign)",
        62,
        N,
        |rng| {
            let a = input::bytes(rng, 12);
            let b = input::near(rng, &a);
            let n = rng.below(a.len().min(b.len()) + 1);
            assert_eq!(
                sign(c!(dt_memcmp(a.as_ptr(), b.as_ptr(), n))),
                sign(port::memcmp(&a, &b, n)),
                "{a:?} {b:?} {n}"
            );
            let (a0, b0) = (cstring(&a), cstring(&b));
            let (pa, pb): (*const c_char, *const c_char) = (a0.as_ptr().cast(), b0.as_ptr().cast());
            let n = rng.below(16);
            assert_eq!(
                sign(c!(dt_strcmp(pa, pb))),
                sign(port::strcmp(&a, &b)),
                "{a:?} {b:?}"
            );
            assert_eq!(
                sign(c!(dt_strncmp(pa, pb, n))),
                sign(port::strncmp(&a, &b, n)),
                "{a:?} {b:?} {n}"
            );
            assert_eq!(
                sign(c!(dt_strcasecmp(pa, pb))),
                sign(port::strcasecmp(&a, &b)),
                "{a:?} {b:?}"
            );
            assert_eq!(
                sign(c!(dt_strncasecmp(pa, pb, n))),
                sign(port::strncasecmp(&a, &b, n)),
                "{a:?} {b:?} {n}"
            );
        },
    );
}

#[test]
fn strchr() {
    run("libc strchr", 63, N, |rng| {
        let s = input::bytes(rng, 10);
        let ch = if s.is_empty() || rng.chance(3) {
            input::byte(rng)
        } else {
            s[rng.below(s.len())]
        };
        let s0 = cstring(&s);
        let want = c!(dt_strchr(s0.as_ptr().cast(), i32::from(ch)));
        let got = port::strchr(&s, ch).map_or(-1, |i| i as isize);
        assert_eq!(want, got, "{s:?} {ch}");
    });
}

#[test]
fn formatting() {
    run("libc printf %f", 64, N, |rng| {
        let bits = rmpv_misc_difftest::node_wire::random_double(rng);
        let mut buf = vec![0_u8; 400];
        let len = c!(dt_format_f(
            f64::from_bits(bits),
            buf.as_mut_ptr().cast(),
            buf.len()
        ));
        assert_eq!(&buf[..len as usize], port::format_f(bits), "{bits:#x}");
    });
    run("libc printf PRId64", 65, N, |rng| {
        let v = match rng.below(3) {
            0 => rng.pick(&[i64::MIN, i64::MAX, 0, -1, 9, 10, -10]),
            1 => rng.next_u64() as i64,
            _ => (rng.next_u64() as i64) >> rng.below(64),
        };
        let mut buf = [0_u8; 32];
        let len = c!(dt_format_d(v, buf.as_mut_ptr().cast(), buf.len()));
        assert_eq!(&buf[..len as usize], port::format_d(v), "{v}");
    });
}
