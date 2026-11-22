//! `bstr_xappend_asprintf` and `bstr_xappend_vasprintf` of `misc/bstr.c`,
//! and the `vsnprintf` they call.
//!
//! The port interprets the format itself (`rmpv_misc::cprintf`), so most of
//! the test is against glibc's `snprintf` on generated formats: every flag,
//! widths and precisions given inline and with `*`, every length modifier
//! glibc accepts with each conversion, `%%`, and integers and doubles biased
//! to their limits and to the halfway points where rounding decides. Then
//! mpv's own `bstr_xappend_asprintf` is run on sequences of appends.
//!
//! Each generated format is kept only if the port accepts it: what the port
//! refuses (`%n`, `%ls`, `%Lf`, positional arguments, unknown conversions)
//! is listed in `cprintf` and tested to be refused below, not compared.

use std::ffi::{c_char, c_long};
use std::ptr::null_mut;

use rmpv_misc::bstr as port;
use rmpv_misc::cprintf::{self, CArg, PrintfError};
use rmpv_misc_difftest::ffi::Bstr;
use rmpv_misc_difftest::input::{self, Rng, cstring};
use rmpv_misc_difftest::{c, copy_out};

use crate::{N, run};

/// An argument as the test owns it.
#[derive(Debug, Clone)]
enum Arg {
    Int(i32),
    UInt(u32),
    Long(i64),
    ULong(u64),
    Double(u64),
    Str(Option<Vec<u8>>),
    Ptr(usize),
}

/// A generated call: the format and its arguments.
#[derive(Debug, Default)]
struct Call {
    fmt: Vec<u8>,
    args: Vec<Arg>,
    doubles: usize,
}

impl Call {
    fn port_args(&self) -> Vec<CArg<'_>> {
        self.args
            .iter()
            .map(|a| match a {
                Arg::Int(v) => CArg::Int(*v),
                Arg::UInt(v) => CArg::UInt(*v),
                Arg::Long(v) => CArg::Long(*v),
                Arg::ULong(v) => CArg::ULong(*v),
                Arg::Double(v) => CArg::Double(*v),
                Arg::Str(v) => CArg::Str(v.as_deref()),
                Arg::Ptr(v) => CArg::Ptr(*v),
            })
            .collect()
    }

    /// The 24 integer slots and 8 doubles of `dt_snprintf`, and the C
    /// strings the slots point to, which must outlive the call.
    fn c_args(&self) -> ([c_long; 24], [f64; 8], Vec<Vec<u8>>) {
        let mut ints = [0; 24];
        let mut dbls = [0.0; 8];
        let mut strings = Vec::new();
        let (mut n_int, mut n_dbl) = (0, 0);
        for a in &self.args {
            let slot = match a {
                Arg::Double(v) => {
                    dbls[n_dbl] = f64::from_bits(*v);
                    n_dbl += 1;
                    continue;
                }
                Arg::Int(v) => *v as c_long,
                Arg::UInt(v) => *v as c_long,
                Arg::Long(v) => *v as c_long,
                Arg::ULong(v) => *v as c_long,
                Arg::Ptr(v) => *v as c_long,
                Arg::Str(None) => 0,
                Arg::Str(Some(s)) => {
                    strings.push(cstring(s));
                    strings.last().unwrap().as_ptr() as c_long
                }
            };
            ints[n_int] = slot;
            n_int += 1;
        }
        (ints, dbls, strings)
    }

    /// What glibc's `snprintf` returns: the text, or -1 and `errno`.
    fn glibc(&self) -> Result<Vec<u8>, i32> {
        let fmt = cstring(&self.fmt);
        let (ints, dbls, _strings) = self.c_args();
        let mut err = 0;
        let fmt_p = fmt.as_ptr().cast::<c_char>();
        let n = c!(dt_snprintf(
            null_mut(),
            0,
            fmt_p,
            ints.as_ptr(),
            dbls.as_ptr(),
            &raw mut err
        ));
        if n < 0 {
            return Err(err);
        }
        let mut buf = vec![0_u8; n as usize + 1];
        let m = c!(dt_snprintf(
            buf.as_mut_ptr().cast(),
            buf.len(),
            fmt_p,
            ints.as_ptr(),
            dbls.as_ptr(),
            &raw mut err
        ));
        assert_eq!(m, n);
        buf.truncate(n as usize);
        Ok(buf)
    }
}

/// A `double` biased towards what formatting branches on: zeros, infinities
/// and NaNs of both signs, subnormals, the extremes, powers of ten and
/// values just around them, and halfway cases for rounding.
fn double_bits(rng: &mut Rng) -> u64 {
    let v: f64 = match rng.below(12) {
        0 => rng.pick(&[
            0.0,
            -0.0,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
            -f64::NAN,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::MIN,
            5e-324,
            f64::EPSILON,
            1.0,
            -1.0,
        ]),
        // Halfway at some decimal place: k / 2^j.
        1 => (rng.below(2001) as f64 - 1000.0) / (1_u64 << rng.below(12)) as f64,
        // Just below a power of ten, where rounding carries.
        2 => {
            let e = rng.below(40) as i32 - 20;
            let p = 10_f64.powi(e);
            let nines = 1.0 - 10_f64.powi(-(rng.below(17) as i32));
            p * nines * if rng.chance(2) { 1.0 } else { -1.0 }
        }
        3 => 10_f64.powi(rng.below(600) as i32 - 300),
        4 => (rng.next_u64() % 1_000_000) as f64 / 1000.0,
        5 => rng.int() as f64,
        6 => f64::from_bits(rng.next_u64() & 0x000f_ffff_ffff_ffff),
        7 => f64::from_bits(rng.next_u64() | 0x7ff0_0000_0000_0000),
        8 => {
            // Neighbours of a short decimal.
            let base = (rng.below(100_000) as f64) / 10_f64.powi(rng.below(8) as i32);
            let bits = base.to_bits();
            f64::from_bits(bits.wrapping_add(rng.below(3) as u64).wrapping_sub(1))
        }
        _ => return rng.next_u64(),
    };
    v.to_bits()
}

fn int_arg(rng: &mut Rng) -> Arg {
    let v = rng.int();
    if rng.chance(3) {
        Arg::UInt(v as u32)
    } else {
        Arg::Int(v)
    }
}

fn long_arg(rng: &mut Rng) -> Arg {
    let v = match rng.below(6) {
        0 => rng.pick(&[i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX, i32::MIN as i64]),
        1 => rng.int() as i64,
        _ => rng.next_u64() as i64 >> rng.below(64),
    };
    if rng.chance(2) {
        Arg::ULong(v as u64)
    } else {
        Arg::Long(v)
    }
}

/// A width, precision or `*` value: mostly small, sometimes negative.
fn star(rng: &mut Rng) -> i32 {
    match rng.below(10) {
        0 => -(rng.below(30) as i32),
        1 => rng.below(300) as i32,
        _ => rng.below(25) as i32,
    }
}

fn number(rng: &mut Rng) -> Vec<u8> {
    let n = match rng.below(12) {
        0 => rng.below(1000),
        1 => rng.below(1300),
        _ => rng.below(30),
    };
    n.to_string().into_bytes()
}

/// Appends one conversion to `call`, with the arguments it reads.
fn conversion(rng: &mut Rng, call: &mut Call, floats_only: bool) {
    let conv = if floats_only {
        rng.pick(b"feEgGaAFf")
    } else {
        rng.pick(b"diuoxXbBcspfFeEgGaA%dduxsgf")
    };
    call.fmt.push(b'%');
    for _ in 0..rng.below(4) {
        call.fmt.push(rng.pick(b" +-#0'0#"));
    }
    match rng.below(5) {
        0 | 1 => {}
        2 => {
            call.fmt.push(b'*');
            call.args.push(Arg::Int(star(rng)));
        }
        _ => call.fmt.extend(number(rng)),
    }
    match rng.below(6) {
        0 | 1 => {}
        2 => call.fmt.push(b'.'),
        3 => {
            call.fmt.extend(b".*");
            call.args.push(Arg::Int(star(rng)));
        }
        _ => {
            call.fmt.push(b'.');
            call.fmt.extend(number(rng));
        }
    }
    let float = b"feEgGaAF".contains(&conv);
    let lengths: &[&[u8]] = match conv {
        b'd' | b'i' | b'u' | b'o' | b'x' | b'X' | b'b' | b'B' => &[
            b"", b"", b"", b"hh", b"h", b"l", b"ll", b"L", b"q", b"j", b"z", b"Z", b"t",
        ],
        b'c' | b's' => &[b"", b"", b"hh"],
        b'p' => &[b"", b"l", b"ll", b"L", b"q", b"z", b"j", b"t", b"hh"],
        b'%' => &[b"", b"h", b"l", b"ll", b"hh", b"z"],
        _ => &[b"", b"", b"", b"l", b"z", b"j", b"t", b"Z", b"hh"],
    };
    let length = rng.pick(lengths);
    call.fmt.extend(length);
    call.fmt.push(conv);
    let long = matches!(
        length,
        b"l" | b"ll" | b"L" | b"q" | b"j" | b"z" | b"Z" | b"t"
    );
    match conv {
        b'%' => {}
        _ if float => {
            call.args.push(Arg::Double(double_bits(rng)));
            call.doubles += 1;
        }
        b'c' => call.args.push(int_arg(rng)),
        b's' => call.args.push(Arg::Str(if rng.chance(8) {
            None
        } else {
            Some(input::bytes(rng, 12))
        })),
        b'p' => call.args.push(Arg::Ptr(match rng.below(4) {
            0 => 0,
            1 => rng.below(1 << 16),
            _ => rng.next_u64() as usize,
        })),
        _ if long => call.args.push(long_arg(rng)),
        _ => call.args.push(int_arg(rng)),
    }
}

fn literal(rng: &mut Rng, call: &mut Call) {
    for b in input::bytes(rng, 4) {
        if b == 0 {
            break;
        }
        call.fmt.push(b);
        if b == b'%' {
            call.fmt.push(b'%');
        }
    }
}

fn generate(rng: &mut Rng, floats_only: bool) -> Call {
    let mut call = Call::default();
    for _ in 0..=rng.below(4) {
        if rng.chance(2) {
            literal(rng, &mut call);
        }
        // At most 8 doubles reach the C; see shim/variadic_shim.c.
        if call.doubles < 8 {
            conversion(rng, &mut call, floats_only);
        }
    }
    if rng.chance(3) {
        literal(rng, &mut call);
    }
    call
}

/// Compares one call; returns whether the port took it.
fn compare(call: &Call) -> bool {
    let args = call.port_args();
    let got = cprintf::vsnprintf(&call.fmt, &args);
    let got = match got {
        Ok(v) => Ok(v),
        Err(PrintfError::Failed(e)) => Err(e),
        Err(PrintfError::Unsupported(_) | PrintfError::BadArgument(_)) => return false,
    };
    let want = call.glibc();
    assert_eq!(
        want.as_ref()
            .map(|v| String::from_utf8_lossy(v).into_owned()),
        got.as_ref()
            .map(|v| String::from_utf8_lossy(v).into_owned()),
        "{:?} {:?}",
        String::from_utf8_lossy(&call.fmt),
        call.args
    );
    assert_eq!(want, got);
    true
}

#[test]
fn vsnprintf() {
    let mut refused = 0;
    run("vsnprintf (glibc snprintf)", 900, N, |rng| {
        let call = generate(rng, false);
        if !compare(&call) {
            refused += 1;
        }
    });
    println!("  of which the port refused {refused}");
    // The generator only builds what glibc defines; the refusals are the
    // length modifiers the port leaves out (`%lc`, `%ls`, `%Lf`, `%llf`).
    assert!(refused < N / 10, "{refused}");
}

#[test]
fn vsnprintf_floats() {
    let mut refused = 0;
    run("vsnprintf %f %e %g %a (glibc snprintf)", 901, N, |rng| {
        let call = generate(rng, true);
        if !compare(&call) {
            refused += 1;
        }
    });
    assert!(refused < N / 10, "{refused}");
}

/// Precisions beyond the digits a double has, where the port writes zeros
/// without formatting them.
#[test]
fn vsnprintf_long_precision() {
    run(
        "vsnprintf long precisions (glibc snprintf)",
        902,
        5000,
        |rng| {
            let conv = rng.pick(b"feEgGa");
            let prec = rng.pick(&[760, 767, 768, 799, 800, 801, 1073, 1074, 1075, 1100, 1500]);
            let flags: &[u8] = rng.pick(&[b"", b"#", b"-", b"0", b"+"]);
            let mut fmt = vec![b'%'];
            fmt.extend_from_slice(flags);
            if rng.chance(2) {
                fmt.extend(b"1600");
            }
            fmt.extend(format!(".{prec}").bytes());
            fmt.push(conv);
            let call = Call {
                fmt,
                args: vec![Arg::Double(double_bits(rng))],
                doubles: 1,
            };
            assert!(compare(&call));
        },
    );
}

/// Widths and precisions at `INT_MAX` and above, and output longer than
/// `INT_MAX`: glibc fails these with `EOVERFLOW`.
#[test]
fn vsnprintf_overflow() {
    let cases: &[(&[u8], Vec<Arg>)] = &[
        (b"%2147483648d", vec![Arg::Int(1)]),
        (b"%99999999999999999999d", vec![Arg::Int(1)]),
        (b"%.2147483648d", vec![Arg::Int(1)]),
        (b"%.2147483648f", vec![Arg::Double(1.5_f64.to_bits())]),
        (b"%*2147483648d", vec![Arg::Int(1), Arg::Int(1)]),
        (b"%1073741824d%1073741824d", vec![Arg::Int(1), Arg::Int(2)]),
        (b"%2147483647d%d", vec![Arg::Int(1), Arg::Int(2)]),
        (
            b"%.2147483647d%s",
            vec![Arg::Int(1), Arg::Str(Some(b"x".to_vec()))],
        ),
        (
            b"%.1073741824f%.1073741824f",
            vec![Arg::Double(0), Arg::Double(0)],
        ),
        (
            b"%1073741824s%1073741824c",
            vec![Arg::Str(None), Arg::Int(65)],
        ),
        (b"%-2147483647a%a", vec![Arg::Double(0), Arg::Double(0)]),
        (b"%", vec![]),
        (b"abc%", vec![]),
        (b"%5", vec![]),
        (b"%-#.3l", vec![]),
        (b"%h", vec![]),
    ];
    for (fmt, args) in cases {
        let call = Call {
            fmt: fmt.to_vec(),
            args: args.clone(),
            doubles: 0,
        };
        assert!(compare(&call), "{}", String::from_utf8_lossy(fmt));
    }
    println!("difftest vsnprintf overflow: {} inputs agree", cases.len());
}

/// `*` with `INT_MIN`: glibc negates it, which leaves it negative, so the
/// conversion is left-justified with no padding.
#[test]
fn vsnprintf_star_int_min() {
    for fmt in [&b"[%*d]"[..], b"[%*s]", b"[%*c]", b"[%*f]", b"[%*x]"] {
        let value = match fmt[3] {
            b's' => Arg::Str(Some(b"ab".to_vec())),
            b'f' => Arg::Double(2.5_f64.to_bits()),
            _ => Arg::Int(65),
        };
        let call = Call {
            fmt: fmt.to_vec(),
            args: vec![Arg::Int(i32::MIN), value],
            doubles: 1,
        };
        assert!(compare(&call));
    }
    println!("difftest vsnprintf * of INT_MIN: 5 inputs agree");
}

/// What the port refuses, and that glibc indeed does something the port
/// does not model for each.
#[test]
fn vsnprintf_refusals() {
    let s = Some(&b"x"[..]);
    let cases: &[(&[u8], &[CArg<'_>], PrintfError)] = &[
        (b"%n", &[CArg::Ptr(8)], PrintfError::Unsupported(0)),
        (b"ab%m", &[], PrintfError::Unsupported(2)),
        (b"%ls", &[CArg::Str(s)], PrintfError::Unsupported(0)),
        (b"%lc", &[CArg::Int(65)], PrintfError::Unsupported(0)),
        (b"%zc", &[CArg::Int(65)], PrintfError::Unsupported(0)),
        (b"%S", &[CArg::Str(s)], PrintfError::Unsupported(0)),
        (b"%C", &[CArg::Int(65)], PrintfError::Unsupported(0)),
        (b"%Lf", &[CArg::Double(0)], PrintfError::Unsupported(0)),
        (b"%llf", &[CArg::Double(0)], PrintfError::Unsupported(0)),
        (b"%1$d", &[CArg::Int(1)], PrintfError::Unsupported(0)),
        (b"%*1$d", &[CArg::Int(1)], PrintfError::Unsupported(0)),
        (b"%Id", &[CArg::Int(1)], PrintfError::Unsupported(0)),
        (b"%w32d", &[CArg::Int(1)], PrintfError::Unsupported(0)),
        (b"%y", &[], PrintfError::Unsupported(0)),
        (b"%5-d", &[CArg::Int(1)], PrintfError::Unsupported(0)),
        (b"%hf", &[CArg::Double(0)], PrintfError::Unsupported(0)),
        (b"%hs", &[CArg::Str(s)], PrintfError::Unsupported(0)),
        (b"%d", &[], PrintfError::BadArgument(0)),
        (b"%d", &[CArg::Long(1)], PrintfError::BadArgument(0)),
        (b"%ld", &[CArg::Int(1)], PrintfError::BadArgument(0)),
        (b"%f", &[CArg::Int(1)], PrintfError::BadArgument(0)),
        (b"%s", &[CArg::Ptr(1)], PrintfError::BadArgument(0)),
        (
            b"%d %s",
            &[CArg::Int(1), CArg::Int(1)],
            PrintfError::BadArgument(1),
        ),
        (
            b"%*d",
            &[CArg::Long(1), CArg::Int(1)],
            PrintfError::BadArgument(0),
        ),
    ];
    for (fmt, args, want) in cases {
        assert_eq!(
            cprintf::vsnprintf(fmt, args),
            Err(*want),
            "{}",
            String::from_utf8_lossy(fmt)
        );
    }
    // glibc prints an unknown conversion back, which the port does not do.
    let unknown = Call {
        fmt: b"%y".to_vec(),
        args: vec![],
        doubles: 0,
    };
    assert_eq!(unknown.glibc(), Ok(b"%y".to_vec()));
    println!(
        "difftest vsnprintf refusals: {} formats refused",
        cases.len()
    );
}

#[test]
fn bstr_xappend_asprintf() {
    run(
        "bstr_xappend_asprintf, bstr_xappend_vasprintf",
        903,
        N / 2,
        |rng| {
            let mut cs = Bstr::NULL;
            let mut rs = Vec::new();
            for _ in 0..=rng.below(4) {
                let floats_only = rng.chance(3);
                let call = generate(rng, floats_only);
                let args = call.port_args();
                let got = if rng.chance(2) {
                    port::bstr_xappend_asprintf(&mut rs, &call.fmt, &args)
                } else {
                    port::bstr_xappend_vasprintf(&mut rs, &call.fmt, &args)
                };
                let got = match got {
                    Ok(n) => n,
                    Err(PrintfError::Failed(_)) => -1,
                    Err(_) => continue,
                };
                let fmt = cstring(&call.fmt);
                let (ints, dbls, _strings) = call.c_args();
                let want = c!(dt_bstr_xappend_asprintf(
                    &raw mut cs,
                    fmt.as_ptr().cast(),
                    ints.as_ptr(),
                    dbls.as_ptr()
                ));
                assert_eq!(want, got, "{:?}", String::from_utf8_lossy(&call.fmt));
                assert_eq!(copy_out(cs.start, cs.len), rs);
            }
            c!(ta_free(cs.start.cast()));
        },
    );
}
