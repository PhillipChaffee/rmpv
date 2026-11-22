//! `bstr_sscanf` of `misc/bstr.c`, the `vsscanf` it calls, and `strtof`.
//!
//! The port interprets the format itself (`rmpv_misc::cscanf`). The test
//! runs it and glibc's `sscanf` (the C23 entry point, as mpv's build links
//! it) on generated formats and inputs, and compares the return value and
//! every argument: each C argument is a buffer filled with a pattern
//! first, so a store the port misses, or one it makes that the C does not,
//! shows. Inputs are built from the format, so that most conversions get
//! text they can at least partly match. Then mpv's `bstr_sscanf` runs on
//! the formats its callers use and on generated ones.

use std::ffi::{c_char, c_void};

use rmpv_misc::bstr as port;
use rmpv_misc::cfloat;
use rmpv_misc::cscanf::{self, ScanArg, ScanfError};
use rmpv_misc_difftest::c;
use rmpv_misc_difftest::ffi::Bstr;
use rmpv_misc_difftest::input::{self, Rng, cstring};

use crate::{N, run};

/// The type a conversion stores through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Bytes,
}

/// A value the port's argument points to.
#[derive(Debug, Clone, PartialEq)]
enum Val {
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(u32),
    F64(u64),
    Bytes(Vec<u8>),
    /// A C buffer, or the port's bytes as the C would have left that
    /// buffer.
    Slot(Vec<u8>),
}

impl Val {
    /// The port's `Bytes` as the C would have written them over the
    /// pattern: the bytes, then a NUL for `%s` and `%[`.
    fn as_slot(&self, conv: u8, len: usize) -> Self {
        match self {
            Self::Bytes(v) => {
                let mut slot = vec![PATTERN; len];
                if v.as_slice() != UNTOUCHED {
                    slot[..v.len()].copy_from_slice(v);
                    if conv != b'c' {
                        slot[v.len()] = 0;
                    }
                }
                Self::Slot(slot)
            }
            other => other.clone(),
        }
    }
}

const PATTERN: u8 = 0xa5;
/// What the port's `Bytes` argument holds before the call.
const UNTOUCHED: &[u8] = b"";

impl Kind {
    fn initial(self) -> Val {
        let w = u64::from_ne_bytes([PATTERN; 8]);
        match self {
            Self::I8 => Val::I8(w as i8),
            Self::U8 => Val::U8(w as u8),
            Self::I16 => Val::I16(w as i16),
            Self::U16 => Val::U16(w as u16),
            Self::I32 => Val::I32(w as i32),
            Self::U32 => Val::U32(w as u32),
            Self::I64 => Val::I64(w as i64),
            Self::U64 => Val::U64(w),
            Self::F32 => Val::F32(w as u32),
            Self::F64 => Val::F64(w),
            Self::Bytes => Val::Bytes(UNTOUCHED.to_vec()),
        }
    }

    /// What the C wrote into `slot`, read as this type.
    fn read_c(self, slot: &[u8]) -> Val {
        let mut w = [0_u8; 8];
        w.copy_from_slice(&slot[..8]);
        let w = u64::from_ne_bytes(w);
        match self {
            Self::I8 => Val::I8(w as i8),
            Self::U8 => Val::U8(w as u8),
            Self::I16 => Val::I16(w as i16),
            Self::U16 => Val::U16(w as u16),
            Self::I32 => Val::I32(w as i32),
            Self::U32 => Val::U32(w as u32),
            Self::I64 => Val::I64(w as i64),
            Self::U64 => Val::U64(w),
            Self::F32 => Val::F32(w as u32),
            Self::F64 => Val::F64(w),
            Self::Bytes => Val::Slot(slot.to_vec()),
        }
    }
}

/// A generated call: format, input, and the kind of each argument with the
/// conversion character it belongs to.
#[derive(Debug, Default)]
struct Call {
    fmt: Vec<u8>,
    input: Vec<u8>,
    kinds: Vec<(Kind, u8)>,
}

impl Call {
    /// Room for anything a conversion can store: at most the whole input
    /// and a NUL.
    fn slot_len(&self) -> usize {
        self.input.len() + 16
    }

    fn glibc(&self) -> (i32, Vec<Val>) {
        let mut slots = vec![vec![PATTERN; self.slot_len()]; self.kinds.len()];
        let mut ptrs: Vec<*mut c_void> = slots.iter_mut().map(|s| s.as_mut_ptr().cast()).collect();
        ptrs.resize(24, std::ptr::null_mut());
        let fmt = cstring(&self.fmt);
        let s = cstring(&self.input);
        let r = c!(dt_sscanf(
            s.as_ptr().cast::<c_char>(),
            fmt.as_ptr().cast(),
            ptrs.as_ptr()
        ));
        let vals = self
            .kinds
            .iter()
            .zip(&slots)
            .map(|(&(k, _), slot)| k.read_c(slot))
            .collect();
        (r, vals)
    }

    fn port(&self) -> Result<(i32, Vec<Val>), ScanfError> {
        let mut vals: Vec<Val> = self.kinds.iter().map(|&(k, _)| k.initial()).collect();
        let mut f32s: Vec<f32> = Vec::new();
        let mut f64s: Vec<f64> = Vec::new();
        for v in &vals {
            match v {
                Val::F32(b) => f32s.push(f32::from_bits(*b)),
                Val::F64(b) => f64s.push(f64::from_bits(*b)),
                _ => {}
            }
        }
        let r = {
            let mut f32_iter = f32s.iter_mut();
            let mut f64_iter = f64s.iter_mut();
            let mut args: Vec<ScanArg<'_>> = vals
                .iter_mut()
                .map(|v| match v {
                    Val::I8(x) => ScanArg::I8(x),
                    Val::U8(x) => ScanArg::U8(x),
                    Val::I16(x) => ScanArg::I16(x),
                    Val::U16(x) => ScanArg::U16(x),
                    Val::I32(x) => ScanArg::I32(x),
                    Val::U32(x) => ScanArg::U32(x),
                    Val::I64(x) => ScanArg::I64(x),
                    Val::U64(x) => ScanArg::U64(x),
                    Val::F32(_) => ScanArg::F32(f32_iter.next().unwrap()),
                    Val::F64(_) => ScanArg::F64(f64_iter.next().unwrap()),
                    Val::Bytes(x) => ScanArg::Bytes(x),
                    Val::Slot(_) => unreachable!(),
                })
                .collect();
            cscanf::vsscanf(&self.input, &self.fmt, &mut args)?
        };
        let (mut i32s, mut i64s) = (f32s.into_iter(), f64s.into_iter());
        for v in &mut vals {
            match v {
                Val::F32(b) => *b = i32s.next().unwrap().to_bits(),
                Val::F64(b) => *b = i64s.next().unwrap().to_bits(),
                _ => {}
            }
        }
        let vals = vals
            .iter()
            .zip(&self.kinds)
            .map(|(v, &(_, conv))| v.as_slot(conv, self.slot_len()))
            .collect();
        Ok((r, vals))
    }
}

fn int_kind(rng: &mut Rng, modifier: &[u8]) -> Kind {
    let signed = rng.chance(2);
    match (modifier, signed) {
        (b"hh", true) => Kind::I8,
        (b"hh", false) => Kind::U8,
        (b"h", true) => Kind::I16,
        (b"h", false) => Kind::U16,
        (b"", true) => Kind::I32,
        (b"", false) => Kind::U32,
        (_, true) => Kind::I64,
        (_, false) => Kind::U64,
    }
}

/// Text an integer conversion in `base` (0 for `%i`) can read, or nearly.
fn int_text(rng: &mut Rng, base: u32) -> Vec<u8> {
    let mut t = Vec::new();
    if rng.chance(3) {
        t.push(rng.pick(b"+-"));
    }
    match rng.below(6) {
        0 => t.extend(rng.pick(&[&b"0x"[..], b"0X", b"0b", b"0B", b"0", b"00"])),
        1 => {}
        _ if base == 16 && rng.chance(2) => t.extend(b"0x"),
        _ => {}
    }
    let digits: &[u8] = match base {
        16 => b"0123456789abcdefABCDEF",
        8 => b"012345678",
        2 => b"0120",
        _ => b"0123456789",
    };
    let n = match rng.below(8) {
        0 => 15 + rng.below(10),
        _ => rng.below(6),
    };
    for _ in 0..n {
        t.push(rng.pick(digits));
    }
    if rng.chance(3) {
        t.push(rng.pick(b"xg9.: "));
    }
    t
}

fn float_text(rng: &mut Rng) -> Vec<u8> {
    match rng.below(10) {
        0 => {
            let mut t = Vec::new();
            if rng.chance(2) {
                t.push(rng.pick(b"+-"));
            }
            t.extend(rng.pick(&[
                &b"nan"[..],
                b"NaN",
                b"na",
                b"inf",
                b"INF",
                b"infinity",
                b"Infinit",
                b"infx",
                b"in",
                b"i",
                b"n",
                b"nan(1)",
            ]));
            t
        }
        1 => {
            let mut t = b"0x".to_vec();
            for _ in 0..rng.below(8) {
                t.push(rng.pick(b"0123456789abcdefABCDEF.pP+-"));
            }
            t
        }
        2 => {
            let mut t = Vec::new();
            for _ in 0..rng.below(8) {
                t.push(rng.pick(b"0123456789.eE+-"));
            }
            t
        }
        _ => input::float_text(rng),
    }
}

fn set_spec(rng: &mut Rng) -> Vec<u8> {
    rng.pick(&[
        &b"[a-z]"[..],
        b"[^ ]",
        b"[]x]",
        b"[^]x]",
        b"[-a]",
        b"[a-]",
        b"[0-9.]",
        b"[z-a]",
        b"[^:]",
        b"[a-c-e]",
        b"[a-a]",
        b"[+--]",
        b"[]-a]",
        b"[\x80-\xff]",
        b"[",
        b"[abc",
    ])
    .to_vec()
}

/// Appends one directive to the format and matching text to the input.
fn directive(rng: &mut Rng, call: &mut Call) {
    match rng.below(10) {
        0 => {
            let lit = rng.pick(b":.,x- \t\n%\x80");
            if lit == b'%' {
                call.fmt.extend(b"%%");
            } else {
                call.fmt.push(lit);
            }
            if !rng.chance(5) {
                if rng.chance(4) {
                    call.input.push(b' ');
                }
                call.input.push(lit);
            }
            return;
        }
        1 => {
            call.fmt.push(b' ');
            for _ in 0..rng.below(3) {
                call.input.push(rng.pick(b" \t\n"));
            }
            return;
        }
        _ => {}
    }
    let conv = rng.pick(b"diuoxXbfeEgGaAscn[%ddffuuxs");
    call.fmt.push(b'%');
    let suppress = rng.chance(6);
    if suppress {
        call.fmt.push(b'*');
    }
    if rng.chance(3) {
        call.fmt
            .extend(rng.pick(&[&b"1"[..], b"2", b"3", b"5", b"10", b"0", b"99999999999"]));
    }
    let modifiers: &[&[u8]] = match conv {
        b'f' | b'e' | b'E' | b'g' | b'G' | b'a' | b'A' => &[b"", b"l", b"", b"l", b"h", b"z"],
        b's' | b'c' | b'[' | b'%' => &[b""],
        _ => &[
            b"", b"", b"hh", b"h", b"l", b"ll", b"L", b"q", b"j", b"z", b"t",
        ],
    };
    let modifier = rng.pick(modifiers);
    call.fmt.extend(modifier);
    let float = b"feEgGaA".contains(&conv);
    if conv == b'[' {
        call.fmt.extend(set_spec(rng));
    } else {
        call.fmt.push(conv);
    }
    let kind = match conv {
        b'%' => None,
        b's' | b'c' | b'[' => Some(Kind::Bytes),
        _ if float => Some(if modifier == b"l" || modifier == b"z" {
            Kind::F64
        } else {
            Kind::F32
        }),
        _ => Some(int_kind(rng, modifier)),
    };
    if let Some(kind) = kind
        && !suppress
    {
        call.kinds.push((kind, conv));
    }
    if rng.chance(4) {
        call.input.push(rng.pick(b" \t"));
    }
    match conv {
        b'%' => call.input.push(b'%'),
        b'n' => {}
        b's' | b'c' | b'[' => call.input.extend(input::text(rng, 6)),
        b'x' | b'X' => call.input.extend(int_text(rng, 16)),
        b'o' => call.input.extend(int_text(rng, 8)),
        b'b' => call.input.extend(int_text(rng, 2)),
        b'i' => call.input.extend(int_text(rng, 0)),
        _ if float => call.input.extend(float_text(rng)),
        _ => call.input.extend(int_text(rng, 10)),
    }
}

fn generate(rng: &mut Rng) -> Call {
    let mut call = Call::default();
    for _ in 0..=rng.below(5) {
        directive(rng, &mut call);
    }
    // Some inputs end early, some have more after the format's end.
    if rng.chance(4) {
        call.input.truncate(rng.below(call.input.len() + 1));
    }
    if rng.chance(4) {
        call.input.extend(input::bytes(rng, 4));
    }
    call
}

fn compare(call: &Call) {
    let got = call.port().unwrap_or_else(|e| {
        panic!(
            "port refused {e:?}: {:?} {:?} {:?}",
            String::from_utf8_lossy(&call.fmt),
            String::from_utf8_lossy(&call.input),
            call.kinds
        )
    });
    let want = call.glibc();
    assert_eq!(
        want,
        got,
        "{:?} {:?} {:?}",
        String::from_utf8_lossy(&call.fmt),
        String::from_utf8_lossy(&call.input),
        call.kinds
    );
}

#[test]
fn vsscanf() {
    run("vsscanf (glibc sscanf)", 910, N, |rng| {
        compare(&generate(rng));
    });
}

/// The formats mpv passes to `bstr_sscanf` (`options/m_option.c`,
/// `demux/codec_tags.c`, `video/out/gpu/user_shaders.c`), on text made for
/// them, run through mpv's own `bstr_sscanf`.
#[test]
fn bstr_sscanf_callers() {
    let formats: &[(&[u8], &[Kind])] = &[
        (
            b"%u:%u:%lf%n",
            &[Kind::U32, Kind::U32, Kind::F64, Kind::I32],
        ),
        (b"%u:%lf%n", &[Kind::U32, Kind::F64, Kind::I32]),
        (b"%lf%n", &[Kind::F64, Kind::I32]),
        (b"%x.%u", &[Kind::U32, Kind::U32]),
        (b"%f", &[Kind::F32]),
        (b"%f %f", &[Kind::F32, Kind::F32]),
        (b"%d", &[Kind::I32]),
        (
            b"%d %d %d %d",
            &[Kind::I32, Kind::I32, Kind::I32, Kind::I32],
        ),
        (b"%d %d %d", &[Kind::I32, Kind::I32, Kind::I32]),
    ];
    run("bstr_sscanf (mpv's callers' formats)", 911, N, |rng| {
        let (fmt, kinds) = rng.pick(formats);
        let mut text = Vec::new();
        for (i, k) in kinds.iter().enumerate() {
            match k {
                Kind::F64 | Kind::F32 => text.extend(float_text(rng)),
                Kind::I32 if fmt.ends_with(b"%n") => {}
                _ if fmt.starts_with(b"%x") && i == 0 => text.extend(int_text(rng, 16)),
                _ => text.extend(int_text(rng, 10)),
            }
            if rng.chance(6) {
                continue;
            }
            text.push(
                match fmt.get(fmt.iter().position(|&b| b == b'%').unwrap() + 2) {
                    Some(b':') => b':',
                    Some(b'.') => b'.',
                    _ => b' ',
                },
            );
        }
        if rng.chance(5) {
            text = input::near(rng, &text);
        }
        let call = Call {
            fmt: fmt.to_vec(),
            input: text,
            kinds: kinds.iter().map(|&k| (k, b'd')).collect(),
        };
        compare_bstr(&call);
    });
}

#[test]
fn bstr_sscanf() {
    run("bstr_sscanf (generated)", 912, N / 2, |rng| {
        let mut call = generate(rng);
        // bstr_sscanf copies the view, which may hold a NUL, into a C string.
        if rng.chance(10) {
            let at = rng.below(call.input.len() + 1);
            call.input.insert(at, 0);
        }
        compare_bstr(&call);
    });
}

/// Compares mpv's `bstr_sscanf` with the port's.
fn compare_bstr(call: &Call) {
    let mut slots = vec![vec![PATTERN; call.slot_len()]; call.kinds.len()];
    let mut ptrs: Vec<*mut c_void> = slots.iter_mut().map(|s| s.as_mut_ptr().cast()).collect();
    ptrs.resize(24, std::ptr::null_mut());
    let fmt = cstring(&call.fmt);
    let r = c!(dt_bstr_sscanf(
        Bstr::of(&call.input),
        fmt.as_ptr().cast(),
        ptrs.as_ptr()
    ));
    let want: Vec<Val> = call
        .kinds
        .iter()
        .zip(&slots)
        .map(|(&(k, _), slot)| k.read_c(slot))
        .collect();
    // The port's bstr_sscanf on the view itself.
    let mut vals: Vec<Val> = call.kinds.iter().map(|&(k, _)| k.initial()).collect();
    let port_call = Call {
        fmt: call.fmt.clone(),
        input: call.input.clone(),
        kinds: call.kinds.clone(),
    };
    let (direct, direct_vals) = port_call.port().unwrap();
    let mut f64s: Vec<f64> = Vec::new();
    let mut f32s: Vec<f32> = Vec::new();
    for v in &vals {
        match v {
            Val::F32(b) => f32s.push(f32::from_bits(*b)),
            Val::F64(b) => f64s.push(f64::from_bits(*b)),
            _ => {}
        }
    }
    let got = {
        let (mut fi, mut di) = (f32s.iter_mut(), f64s.iter_mut());
        let mut args: Vec<ScanArg<'_>> = vals
            .iter_mut()
            .map(|v| match v {
                Val::I8(x) => ScanArg::I8(x),
                Val::U8(x) => ScanArg::U8(x),
                Val::I16(x) => ScanArg::I16(x),
                Val::U16(x) => ScanArg::U16(x),
                Val::I32(x) => ScanArg::I32(x),
                Val::U32(x) => ScanArg::U32(x),
                Val::I64(x) => ScanArg::I64(x),
                Val::U64(x) => ScanArg::U64(x),
                Val::F32(_) => ScanArg::F32(fi.next().unwrap()),
                Val::F64(_) => ScanArg::F64(di.next().unwrap()),
                Val::Bytes(x) => ScanArg::Bytes(x),
                Val::Slot(_) => unreachable!(),
            })
            .collect();
        port::bstr_sscanf(&call.input, &call.fmt, &mut args).unwrap()
    };
    assert_eq!((r, &want), (direct, &direct_vals));
    assert_eq!(r, got);
    // The values stored through bstr_sscanf match those of the direct call.
    let (mut fi, mut di) = (f32s.into_iter(), f64s.into_iter());
    for v in &mut vals {
        match v {
            Val::F32(b) => *b = fi.next().unwrap().to_bits(),
            Val::F64(b) => *b = di.next().unwrap().to_bits(),
            _ => {}
        }
    }
    let vals: Vec<Val> = vals
        .iter()
        .zip(&call.kinds)
        .map(|(v, &(_, conv))| v.as_slot(conv, call.slot_len()))
        .collect();
    assert_eq!(vals, want);
}

/// What the port refuses.
#[test]
fn vsscanf_refusals() {
    let mut int = 0_i32;
    let mut bytes = Vec::new();
    let mut double = 0.0_f64;
    let cases: Vec<(&[u8], ScanfError)> = vec![
        (b"%1$d", ScanfError::Unsupported(0)),
        (b"%Id", ScanfError::Unsupported(0)),
        (b"%ms", ScanfError::Unsupported(0)),
        (b"%w32d", ScanfError::Unsupported(0)),
        (b"%ls", ScanfError::Unsupported(0)),
        (b"%lc", ScanfError::Unsupported(0)),
        (b"%l[a]", ScanfError::Unsupported(0)),
        (b"%S", ScanfError::Unsupported(0)),
        (b"%C", ScanfError::Unsupported(0)),
        (b"%p", ScanfError::Unsupported(0)),
        (b"%Lf", ScanfError::Unsupported(0)),
        (b"%llf", ScanfError::Unsupported(0)),
        (b"%d", ScanfError::BadArgument(0)),
        (b"%d%d", ScanfError::BadArgument(1)),
    ];
    for (fmt, want) in &cases {
        let mut args = [ScanArg::I32(&mut int)];
        let n = usize::from(fmt != b"%d");
        let r = cscanf::vsscanf(b"12 34", fmt, &mut args[..n]);
        assert_eq!(r, Err(*want), "{}", String::from_utf8_lossy(fmt));
    }
    // Wrong kinds, which C would write through anyway.
    let r = cscanf::vsscanf(b"12", b"%s", &mut [ScanArg::I32(&mut int)]);
    assert_eq!(r, Err(ScanfError::BadArgument(0)));
    let r = cscanf::vsscanf(b"12", b"%lf", &mut [ScanArg::Bytes(&mut bytes)]);
    assert_eq!(r, Err(ScanfError::BadArgument(0)));
    let r = cscanf::vsscanf(b"12", b"%ld", &mut [ScanArg::F64(&mut double)]);
    assert_eq!(r, Err(ScanfError::BadArgument(0)));
    // A conversion that fails before it stores takes no argument, in C too.
    assert_eq!(cscanf::vsscanf(b"x", b"%d", &mut []), Ok(0));
    assert_eq!(cscanf::vsscanf(b"", b"%d", &mut []), Ok(-1));
    println!(
        "difftest vsscanf refusals: {} formats refused",
        cases.len() + 3
    );
}

#[test]
fn strtof() {
    run("strtof", 913, N, |rng| {
        let s = if rng.chance(8) {
            // Near the float limits and halfway points.
            rng.pick(&[
                &b"3.4028235e38"[..],
                b"3.40282357e38",
                b"3.4028236e38",
                b"1.17549435e-38",
                b"1.4e-45",
                b"7.006492e-46",
                b"7.0064923e-46",
                b"7.0064924e-46",
                b"0x1.fffffep127",
                b"0x1.ffffffp127",
                b"0x1.000001p0",
                b"0x1.0000008p0",
                b"0x1.00000080000000001p0",
                // Halfway, and halfway plus bits beyond the 64 that fit.
                b"0x1.000001p0",
                b"0x1.0000010000000000001p0",
                b"0x1.0000030000000000000p0",
                b"0x0.000001000000000000001p-126",
                b"0x0.000002p-126",
                b"0x0.000001p-126",
                b"0x0.0000018p-126",
                b"16777217",
                b"16777217.000000001",
                b"8388608.5",
                b"8388609.5",
            ])
            .to_vec()
        } else if rng.chance(3) {
            float_text(rng)
        } else {
            input::float_text(rng)
        };
        let cs = cstring(&s);
        let mut end: *mut c_char = std::ptr::null_mut();
        let want = c!(strtof(cs.as_ptr().cast(), &raw mut end));
        let want_len = end as usize - cs.as_ptr() as usize;
        let (bits, consumed) = cfloat::strtof(&s);
        assert_eq!(
            (want.to_bits(), want_len),
            (bits, consumed),
            "{:?}",
            String::from_utf8_lossy(&s)
        );
    });
}
