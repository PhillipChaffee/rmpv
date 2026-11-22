//! `vsnprintf` as glibc 2.39 behaves in the "C" locale, for the
//! `bstr_xappend_asprintf` family.
//!
//! A C variadic call becomes a slice of [`CArg`]: each argument as the C
//! callee sees it after the default argument promotions. The format string is
//! interpreted the way glibc's `__printf_buffer` does it
//! (`stdio-common/vfprintf-internal.c` and `vfprintf-process-arg.c` for
//! integers, strings and pointers, `printf_fp.c` for `%f`, `%e` and `%g`,
//! `printf_fphex.c` for `%a`).
//!
//! What the port does not model is refused with [`PrintfError::Unsupported`]
//! instead of being approximated: positional arguments (`%1$d`), `%n`, `%m`,
//! wide characters and strings (`%lc`, `%ls`, `%C`, `%S`), `long double`
//! (`%Lf`), the `I` flag, the `wN` length modifiers, and every conversion
//! glibc does not know (which glibc prints back literally). An argument of
//! the wrong kind, or a missing one, is undefined behaviour in C and is
//! refused with [`PrintfError::BadArgument`].
//!
//! Charon is told to leave this module out: its float formatting is outside
//! what Aeneas models, like `cfloat`.

/// `EINVAL`: glibc's error for a format that ends inside a conversion.
pub const EINVAL: i32 = 22;
/// `EOVERFLOW`: glibc's error for a width, precision or output length above
/// `INT_MAX`.
pub const EOVERFLOW: i32 = 75;

const INT_MAX: u64 = i32::MAX as u64;

/// One argument of a C variadic call, after the default argument promotions.
/// The sizes are those of x86-64 Linux (LP64).
#[derive(Copy)]
#[cfg_attr(not(aeneas), derive(Debug, PartialEq, Eq))]
pub enum CArg<'a> {
    /// `int`, and `char`, `short` and `bool` promoted to it.
    Int(i32),
    /// `unsigned int`, and `unsigned char` and `unsigned short` promoted to
    /// `int`, whose bits `va_arg(ap, unsigned)` reads the same.
    UInt(u32),
    /// `long`, `long long`, `ssize_t`, `ptrdiff_t`, `intmax_t`, `int64_t`.
    Long(i64),
    /// `unsigned long`, `unsigned long long`, `size_t`, `uintmax_t`,
    /// `uint64_t`.
    ULong(u64),
    /// `double`, and `float` promoted to it, as its bit pattern.
    Double(u64),
    /// `const char *`: a C string (it ends at its first NUL or at the end
    /// of the slice), or `None` for NULL.
    Str(Option<&'a [u8]>),
    /// `void *` for `%p`, as its address.
    Ptr(usize),
}

// The `Clone` impls are written out: Charon's translation of a derived
// `Clone` stops Aeneas with an internal error, even in an opaque module.
#[expect(
    clippy::expl_impl_clone_on_copy,
    reason = "a derived Clone stops Aeneas; see above"
)]
impl Clone for CArg<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Why [`vsnprintf`] produced no output.
#[derive(Copy)]
#[cfg_attr(not(aeneas), derive(Debug, PartialEq, Eq))]
pub enum PrintfError {
    /// glibc fails too: `vsnprintf` returns -1 with this `errno`
    /// ([`EINVAL`] or [`EOVERFLOW`]).
    Failed(i32),
    /// The conversion at this byte offset of the format is outside what the
    /// port models (see the module documentation).
    Unsupported(usize),
    /// The argument with this index is missing, or is not of the kind its
    /// conversion reads. In C this is undefined behaviour.
    BadArgument(usize),
}

#[expect(
    clippy::expl_impl_clone_on_copy,
    reason = "a derived Clone stops Aeneas; see above"
)]
impl Clone for PrintfError {
    fn clone(&self) -> Self {
        *self
    }
}

/// The output being built. Padding is kept as a run, so that a width or
/// precision near `INT_MAX` costs nothing until the text is known to fit.
struct Out {
    runs: Vec<Run>,
    count: u64,
}

enum Run {
    Bytes(Vec<u8>),
    Fill(u8, u64),
}

impl Out {
    fn put(&mut self, b: u8) {
        if let Some(Run::Bytes(v)) = self.runs.last_mut() {
            v.push(b);
        } else {
            self.runs.push(Run::Bytes(vec![b]));
        }
        self.count += 1;
    }

    fn write(&mut self, s: &[u8]) {
        for &b in s {
            self.put(b);
        }
    }

    /// `__printf_buffer_pad`: nothing for a count of 0 or less.
    fn pad(&mut self, b: u8, n: i64) {
        if n > 0 {
            self.runs.push(Run::Fill(b, n as u64));
            self.count += n as u64;
        }
    }

    /// `__printf_buffer_done`: the text, or `EOVERFLOW` if it is longer
    /// than `INT_MAX`.
    fn done(self) -> Result<Vec<u8>, PrintfError> {
        if self.count > INT_MAX {
            return Err(PrintfError::Failed(EOVERFLOW));
        }
        let mut text = Vec::with_capacity(self.count as usize);
        for run in self.runs {
            match run {
                Run::Bytes(v) => text.extend_from_slice(&v),
                Run::Fill(b, n) => text.resize(text.len() + n as usize, b),
            }
        }
        Ok(text)
    }
}

/// The flags, width and precision of one conversion (glibc's `printf_info`).
#[expect(
    clippy::struct_excessive_bools,
    reason = "glibc's printf_info flags, one for each flag character"
)]
#[derive(Copy)]
struct Spec {
    alt: bool,
    space: bool,
    left: bool,
    showsign: bool,
    pad: u8,
    /// An `int`: glibc adjusts it with `int` arithmetic, which wraps.
    width: i32,
    prec: i64,
    conv: u8,
}

/// The length modifiers, as glibc records them on LP64.
#[expect(
    clippy::struct_excessive_bools,
    reason = "glibc's own four flags; `hh` and `h` exclude each other, `L` implies `l`"
)]
#[derive(Copy)]
struct Length {
    is_char: bool,
    is_short: bool,
    is_long: bool,
    is_long_double: bool,
}

/// `width -= n` on glibc's `int` width: the difference is truncated to
/// `int`, as the compiled C does.
fn sub(width: i32, n: i64) -> i32 {
    (width as i64).wrapping_sub(n) as i32
}

#[expect(
    clippy::expl_impl_clone_on_copy,
    reason = "a derived Clone stops Aeneas; see above"
)]
impl Clone for Spec {
    fn clone(&self) -> Self {
        *self
    }
}

#[expect(
    clippy::expl_impl_clone_on_copy,
    reason = "a derived Clone stops Aeneas; see above"
)]
impl Clone for Length {
    fn clone(&self) -> Self {
        *self
    }
}

/// glibc's `read_int`: the decimal number at `f[*i]`, or `None` above
/// `INT_MAX`. Leaves `*i` after the digits.
fn read_int(f: &[u8], i: &mut usize) -> Option<i64> {
    let mut v: Option<i64> = Some(0);
    while is_digit(at(f, *i)) {
        let d = (at(f, *i) - b'0') as i64;
        v = match v {
            Some(n) if n * 10 + d <= INT_MAX as i64 => Some(n * 10 + d),
            _ => None,
        };
        *i += 1;
    }
    v
}

fn at(f: &[u8], i: usize) -> u8 {
    crate::clib::cstr_at(f, i)
}

fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

struct Args<'a, 'b> {
    list: &'b [CArg<'a>],
    next: usize,
}

impl<'a> Args<'a, '_> {
    fn take(&mut self) -> Result<(usize, CArg<'a>), PrintfError> {
        let i = self.next;
        self.next += 1;
        match self.list.get(i) {
            Some(&a) => Ok((i, a)),
            None => Err(PrintfError::BadArgument(i)),
        }
    }

    /// `va_arg(ap, int)` or `va_arg(ap, unsigned int)`: the 32 bits.
    fn int(&mut self) -> Result<u32, PrintfError> {
        match self.take()? {
            (_, CArg::Int(v)) => Ok(v as u32),
            (_, CArg::UInt(v)) => Ok(v),
            (i, _) => Err(PrintfError::BadArgument(i)),
        }
    }

    /// `va_arg(ap, long)` or `va_arg(ap, unsigned long)`: the 64 bits.
    fn long(&mut self) -> Result<u64, PrintfError> {
        match self.take()? {
            (_, CArg::Long(v)) => Ok(v as u64),
            (_, CArg::ULong(v)) => Ok(v),
            (i, _) => Err(PrintfError::BadArgument(i)),
        }
    }

    /// The argument of `%d` and `%i`, converted as glibc converts it.
    fn signed(&mut self, len: Length) -> Result<i64, PrintfError> {
        Ok(if len.is_long_double || len.is_long {
            self.long()? as i64
        } else if len.is_char {
            self.int()? as u8 as i8 as i64
        } else if len.is_short {
            self.int()? as u16 as i16 as i64
        } else {
            self.int()? as i32 as i64
        })
    }

    /// The argument of `%u`, `%o`, `%x`, `%X`, `%b` and `%B`.
    fn unsigned(&mut self, len: Length) -> Result<u64, PrintfError> {
        Ok(if len.is_long_double || len.is_long {
            self.long()?
        } else if len.is_char {
            self.int()? as u8 as u64
        } else if len.is_short {
            self.int()? as u16 as u64
        } else {
            self.int()? as u64
        })
    }

    fn double(&mut self) -> Result<u64, PrintfError> {
        match self.take()? {
            (_, CArg::Double(v)) => Ok(v),
            (i, _) => Err(PrintfError::BadArgument(i)),
        }
    }

    fn string(&mut self) -> Result<Option<&'a [u8]>, PrintfError> {
        match self.take()? {
            (_, CArg::Str(v)) => Ok(v),
            (i, _) => Err(PrintfError::BadArgument(i)),
        }
    }

    fn pointer(&mut self) -> Result<usize, PrintfError> {
        match self.take()? {
            (_, CArg::Ptr(v)) => Ok(v),
            (i, _) => Err(PrintfError::BadArgument(i)),
        }
    }
}

/// What `vsnprintf(NULL, 0, fmt, args)` computes, with the text it would
/// have written. `fmt` is a C string.
///
/// # Errors
///
/// [`PrintfError::Failed`] where glibc returns -1, and the port's two
/// refusals; see [`PrintfError`].
pub fn vsnprintf(fmt: &[u8], args: &[CArg<'_>]) -> Result<Vec<u8>, PrintfError> {
    let mut out = Out {
        runs: Vec::new(),
        count: 0,
    };
    let mut args = Args {
        list: args,
        next: 0,
    };
    let mut i = 0;
    loop {
        let c = at(fmt, i);
        if c == 0 {
            break;
        }
        if c != b'%' {
            out.put(c);
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        conversion(fmt, start, &mut i, &mut args, &mut out)?;
    }
    out.done()
}

/// Parses the flags, width and precision of the conversion that starts at
/// `fmt[start]` (`%`), taking `*` arguments. `*i` is just after the `%`, and
/// is left after the precision.
fn parse_spec(
    fmt: &[u8],
    start: usize,
    i: &mut usize,
    args: &mut Args<'_, '_>,
) -> Result<Spec, PrintfError> {
    let unsupported = Err(PrintfError::Unsupported(start));
    let mut s = Spec {
        alt: false,
        space: false,
        left: false,
        showsign: false,
        pad: b' ',
        width: 0,
        prec: -1,
        conv: 0,
    };
    // Step 0: flags, in any order and number.
    loop {
        match at(fmt, *i) {
            b' ' => s.space = true,
            b'+' => s.showsign = true,
            b'-' => {
                s.left = true;
                s.pad = b' ';
            }
            b'#' => s.alt = true,
            b'0' => {
                if !s.left {
                    s.pad = b'0';
                }
            }
            // Grouping: the "C" locale has no thousands separator.
            b'\'' => {}
            b'I' => return unsupported,
            _ => break,
        }
        *i += 1;
    }
    // Width.
    if at(fmt, *i) == b'*' {
        *i += 1;
        if is_digit(at(fmt, *i)) {
            // `%*N$`, or digits glibc does not accept here.
            let mut j = *i;
            if read_int(fmt, &mut j).is_none() {
                return Err(PrintfError::Failed(EOVERFLOW));
            }
            return unsupported;
        }
        let w = args.int()? as i32;
        if w < 0 {
            // glibc negates an int: INT_MIN stays negative.
            s.width = w.wrapping_neg();
            s.pad = b' ';
            s.left = true;
        } else {
            s.width = w;
        }
    } else if is_digit(at(fmt, *i)) {
        match read_int(fmt, i) {
            Some(w) => s.width = w as i32,
            None => return Err(PrintfError::Failed(EOVERFLOW)),
        }
        if at(fmt, *i) == b'$' {
            return unsupported;
        }
    }
    // Precision.
    if at(fmt, *i) == b'.' {
        *i += 1;
        if at(fmt, *i) == b'*' {
            *i += 1;
            if is_digit(at(fmt, *i)) {
                let mut j = *i;
                if read_int(fmt, &mut j).is_none() {
                    return Err(PrintfError::Failed(EOVERFLOW));
                }
                return unsupported;
            }
            let p = args.int()? as i32;
            s.prec = if p < 0 { -1 } else { p as i64 };
        } else if is_digit(at(fmt, *i)) {
            match read_int(fmt, i) {
                Some(p) => s.prec = p,
                None => return Err(PrintfError::Failed(EOVERFLOW)),
            }
        } else {
            s.prec = 0;
        }
    }
    Ok(s)
}

/// Parses the length modifiers at `fmt[*i]`. Returns them and glibc's jump
/// table for what may follow: 2 after none, 3 after one `h`, 4 after one
/// `l`, 5 after a complete modifier.
fn parse_length(fmt: &[u8], start: usize, i: &mut usize) -> Result<(Length, u8), PrintfError> {
    let mut len = Length {
        is_char: false,
        is_short: false,
        is_long: false,
        is_long_double: false,
    };
    let mut step = 2;
    loop {
        let c = at(fmt, *i);
        match (step, c) {
            (2, b'h') => {
                len.is_short = true;
                step = 3;
            }
            (3, b'h') => {
                len.is_short = false;
                len.is_char = true;
                step = 5;
            }
            (2, b'l') => {
                len.is_long = true;
                step = 4;
            }
            (2, b'L' | b'q') | (4, b'l') => {
                len.is_long_double = true;
                len.is_long = true;
                step = 5;
            }
            // size_t, ptrdiff_t and intmax_t are as wide as long on LP64.
            (2, b'z' | b'Z' | b't' | b'j') => {
                len.is_long = true;
                step = 5;
            }
            (2, b'w') => return Err(PrintfError::Unsupported(start)),
            _ => break,
        }
        *i += 1;
    }
    Ok((len, step))
}

/// Parses and writes the conversion that starts at `fmt[start]` (`%`);
/// `*i` is just after the `%`, and is left after the conversion character.
fn conversion(
    fmt: &[u8],
    start: usize,
    i: &mut usize,
    args: &mut Args<'_, '_>,
    out: &mut Out,
) -> Result<(), PrintfError> {
    let unsupported = Err(PrintfError::Unsupported(start));
    let mut s = parse_spec(fmt, start, i, args)?;
    let (len, step) = parse_length(fmt, start, i)?;
    let c = at(fmt, *i);
    s.conv = c;
    // What each step's table admits; anything else is glibc's
    // form_unknown.
    let admitted = match c {
        b'%' | b'd' | b'i' | b'u' | b'o' | b'x' | b'X' | b'b' | b'B' | b'n' => true,
        b'e' | b'E' | b'f' | b'F' | b'g' | b'G' | b'a' | b'A' | b'c' | b's' | b'S' | b'p'
        | b'm' | b'C' => step != 3,
        _ => false,
    };
    if !admitted {
        if c == 0 {
            // The format ended inside the conversion.
            return Err(PrintfError::Failed(EINVAL));
        }
        return unsupported;
    }
    *i += 1;
    match c {
        b'%' => out.put(b'%'),
        b'd' | b'i' => {
            let v = args.signed(len)?;
            integer(out, s, 10, v < 0, v.unsigned_abs());
        }
        b'u' | b'o' | b'x' | b'X' | b'b' | b'B' => {
            let v = args.unsigned(len)?;
            let base = match c {
                b'u' => 10,
                b'o' => 8,
                b'x' | b'X' => 16,
                _ => 2,
            };
            // ISO C defines `+` and ` ` for signed conversions only.
            s.showsign = false;
            s.space = false;
            integer(out, s, base, false, v);
        }
        b'p' => {
            let p = args.pointer()?;
            if p == 0 {
                // "(nil)", printed whole whatever the precision.
                if s.prec < 5 {
                    s.prec = 5;
                }
                string(out, s, Some(b"(nil)"));
            } else {
                // `%#lx`, keeping the `+` and ` ` flags.
                s.alt = true;
                s.conv = b'x';
                integer(out, s, 16, false, p as u64);
            }
        }
        b'c' => {
            if len.is_long {
                return unsupported;
            }
            let ch = args.int()? as u8;
            let width = sub(s.width, 1);
            if !s.left {
                out.pad(b' ', width.into());
            }
            out.put(ch);
            if s.left {
                out.pad(b' ', width.into());
            }
        }
        b's' => {
            if len.is_long {
                return unsupported;
            }
            let v = args.string()?;
            string(out, s, v);
        }
        b'e' | b'E' | b'f' | b'F' | b'g' | b'G' => {
            if len.is_long_double {
                return unsupported;
            }
            let bits = args.double()?;
            float(out, s, bits);
        }
        b'a' | b'A' => {
            if len.is_long_double {
                return unsupported;
            }
            let bits = args.double()?;
            float_hex(out, s, bits);
        }
        // %n, %m, %C, %S
        _ => return unsupported,
    }
    Ok(())
}

/// The digits of `v` in `base`, most significant first (`_itoa_word`).
fn itoa(mut v: u64, base: u64, upper: bool) -> Vec<u8> {
    let digits: &[u8; 16] = if upper {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };
    let mut rev = Vec::new();
    loop {
        rev.push(digits[(v % base) as usize]);
        v /= base;
        if v == 0 {
            break;
        }
    }
    rev.reverse();
    rev
}

/// glibc's `LABEL (number)` and `LABEL (unsigned_number)`: an integer of
/// magnitude `word` and sign `is_negative`.
fn integer(out: &mut Out, mut s: Spec, base: u64, is_negative: bool, word: u64) {
    let mut prec = s.prec;
    if prec < 0 {
        prec = 1;
    } else {
        // A precision turns the `0` flag off.
        s.pad = b' ';
    }
    let digits = if prec == 0 && word == 0 {
        if base == 8 && s.alt {
            vec![b'0']
        } else {
            Vec::new()
        }
    } else {
        itoa(word, base, s.conv == b'X')
    };
    let number_length = digits.len() as i64;
    let octal_marker = prec <= number_length && word != 0 && s.alt && base == 8;
    let prefix = word != 0 && s.alt && (base == 16 || base == 2);
    let prec_inc = (prec - number_length).max(0);
    let mut width = s.width;
    let sign = if is_negative {
        Some(b'-')
    } else if s.showsign {
        Some(b'+')
    } else if s.space {
        Some(b' ')
    } else {
        None
    };
    if s.left {
        if let Some(b) = sign {
            out.put(b);
            width = sub(width, 1);
        }
        if prefix {
            out.put(b'0');
            out.put(s.conv);
            width = sub(width, 2);
        }
        if octal_marker {
            width = sub(width, 1);
        }
        width = sub(width, number_length + prec_inc);
        out.pad(b'0', prec_inc);
        if octal_marker {
            out.put(b'0');
        }
        out.write(&digits);
        out.pad(b' ', width.into());
    } else {
        width = sub(width, number_length + prec_inc);
        if prefix {
            width = sub(width, 2);
        }
        if octal_marker {
            width = sub(width, 1);
        }
        if sign.is_some() {
            width = sub(width, 1);
        }
        if s.pad == b' ' {
            out.pad(b' ', width.into());
            width = 0;
        }
        if let Some(b) = sign {
            out.put(b);
        }
        if prefix {
            out.put(b'0');
            out.put(s.conv);
        }
        width = sub(width, -prec_inc);
        out.pad(b'0', width.into());
        if octal_marker {
            out.put(b'0');
        }
        out.write(&digits);
    }
}

/// glibc's `LABEL (print_string)`.
fn string(out: &mut Out, s: Spec, v: Option<&[u8]>) {
    let text: &[u8] = match v {
        // "(null)" only if it fits the precision.
        None if s.prec == -1 || s.prec >= 6 => b"(null)",
        None => b"",
        Some(v) => {
            let mut n = crate::clib::strlen(v);
            if s.prec >= 0 && (s.prec as usize) < n {
                n = s.prec as usize;
            }
            &v[..n]
        }
    };
    let width = sub(s.width, text.len() as i64);
    if width < 0 {
        out.write(text);
        return;
    }
    if !s.left {
        out.pad(b' ', width.into());
    }
    out.write(text);
    if s.left {
        out.pad(b' ', width.into());
    }
}

const SIGN_BIT: u64 = 1 << 63;
const EXP_MASK: u64 = 0x7ff0_0000_0000_0000;
const MANT_MASK: u64 = (1 << 52) - 1;

/// A double's decimal digits beyond this many places after the point are
/// all 0 (2^-1074 has 1074 of them).
const MAX_FRAC_DIGITS: i64 = 1074;
/// ... and it has at most this many significant digits.
const MAX_SIG_DIGITS: i64 = 800;

/// The `inf` and `nan` of `printf_fp.c` and `printf_fphex.c`: padded with
/// spaces whatever the flags.
fn special(out: &mut Out, s: Spec, negative: bool, word: [u8; 3]) {
    let mut width = s.width;
    if negative || s.showsign || s.space {
        width = sub(width, 1);
    }
    width = sub(width, 3);
    if !s.left {
        out.pad(b' ', width.into());
    }
    if negative {
        out.put(b'-');
    } else if s.showsign {
        out.put(b'+');
    } else if s.space {
        out.put(b' ');
    }
    out.write(&word);
    if s.left {
        out.pad(b' ', width.into());
    }
}

fn special_word(s: Spec, bits: u64) -> Option<[u8; 3]> {
    let upper = s.conv.is_ascii_uppercase();
    let magnitude = bits & !SIGN_BIT;
    if magnitude > EXP_MASK {
        Some(if upper { *b"NAN" } else { *b"nan" })
    } else if magnitude == EXP_MASK {
        Some(if upper { *b"INF" } else { *b"inf" })
    } else {
        None
    }
}

/// The body of `%f` (without sign): `|x|` rounded to `prec` places, ties to
/// even, which is what glibc does in the default rounding mode. Digits
/// beyond what a double can have are zeros, written as a count to pad
/// with.
fn fixed_digits(magnitude: u64, prec: i64) -> (Vec<u8>, i64) {
    let magnitude = f64::from_bits(magnitude);
    let exact = prec.min(MAX_FRAC_DIGITS);
    let text = format!("{magnitude:.*}", exact as usize);
    (text.into_bytes(), prec - exact)
}

/// The significant digits of `|x|` rounded to `prec + 1` of them, and the
/// decimal exponent of the first: `|x| ~ d.ddd * 10^exp`. Zero has exponent
/// 0. Digits beyond what a double can have are zeros, given as a count.
fn exp_digits(magnitude: u64, prec: i64) -> (Vec<u8>, i64, i64) {
    let magnitude = f64::from_bits(magnitude);
    let exact = prec.min(MAX_SIG_DIGITS);
    let text = format!("{magnitude:.*e}", exact as usize);
    let (mantissa, exp) = text.split_once('e').unwrap_or((&text, "0"));
    let digits: Vec<u8> = mantissa.bytes().filter(u8::is_ascii_digit).collect();
    let exp: i64 = exp.parse().unwrap_or(0);
    (digits, exp, prec - exact)
}

/// The exponent part of `%e`: the letter, a sign and at least two digits.
fn exponent_text(letter: u8, exp: i64) -> Vec<u8> {
    let mut v = vec![letter, if exp < 0 { b'-' } else { b'+' }];
    let digits = itoa(exp.unsigned_abs(), 10, false);
    if digits.len() < 2 {
        v.push(b'0');
    }
    v.extend_from_slice(&digits);
    v
}

/// `printf_fp.c` for `%f`, `%F`, `%e`, `%E`, `%g` and `%G`.
fn float(out: &mut Out, s: Spec, bits: u64) {
    let negative = bits & SIGN_BIT != 0;
    if let Some(word) = special_word(s, bits) {
        special(out, s, negative, word);
        return;
    }
    // The bits of |x|; a double crosses function boundaries as its bits.
    let magnitude = bits & !SIGN_BIT;
    // The body as bytes, then a run of zeros to append that may be too long
    // to hold, then the exponent.
    let mut body: Vec<u8>;
    let mut zeros: i64;
    let mut tail: Vec<u8> = Vec::new();
    match s.conv {
        b'f' | b'F' => {
            let prec = if s.prec < 0 { 6 } else { s.prec };
            (body, zeros) = fixed_digits(magnitude, prec);
            if prec == 0 && s.alt {
                body.push(b'.');
            }
        }
        b'e' | b'E' => {
            let prec = if s.prec < 0 { 6 } else { s.prec };
            let (digits, exp, extra) = exp_digits(magnitude, prec);
            body = vec![digits[0]];
            if prec > 0 || s.alt {
                body.push(b'.');
            }
            body.extend_from_slice(&digits[1..]);
            zeros = extra;
            tail = exponent_text(if s.conv == b'E' { b'E' } else { b'e' }, exp);
        }
        _ => {
            // %g: P significant digits; the style follows the exponent X of
            // the %e conversion with precision P - 1.
            let p = if s.prec < 0 {
                6
            } else if s.prec == 0 {
                1
            } else {
                s.prec
            };
            let (digits, exp, extra) = exp_digits(magnitude, p - 1);
            if p > exp && exp >= -4 {
                (body, zeros) = fixed_digits(magnitude, p - 1 - exp);
            } else {
                body = vec![digits[0]];
                body.push(b'.');
                body.extend_from_slice(&digits[1..]);
                zeros = extra;
                tail = exponent_text(if s.conv == b'G' { b'E' } else { b'e' }, exp);
            }
            if s.alt && exp == p && exp_digits(magnitude, MAX_SIG_DIGITS).1 == p - 1 {
                // glibc chose the %f style before rounding, and rounding
                // carried into a new digit: it then switches to the %e
                // style but keeps only as many fraction digits as the %f
                // style had, none. So `%#g` of 999999.5 is "1.e+06", where
                // ISO C asks for "1.00000e+06".
                body = vec![b'1', b'.'];
                zeros = 0;
            } else if s.alt {
                if !body.contains(&b'.') {
                    body.push(b'.');
                }
            } else {
                // Trailing zeros go, and then a point with nothing after it.
                zeros = 0;
                if body.contains(&b'.') {
                    while body.last() == Some(&b'0') {
                        body.pop();
                    }
                    if body.last() == Some(&b'.') {
                        body.pop();
                    }
                }
            }
        }
    }
    let mut width = s.width;
    if negative || s.showsign || s.space {
        width = sub(width, 1);
    }
    width = sub(width, body.len() as i64 + zeros + tail.len() as i64);
    if !s.left && s.pad != b'0' {
        out.pad(s.pad, width.into());
    }
    if negative {
        out.put(b'-');
    } else if s.showsign {
        out.put(b'+');
    } else if s.space {
        out.put(b' ');
    }
    if !s.left && s.pad == b'0' {
        out.pad(b'0', width.into());
    }
    out.write(&body);
    out.pad(b'0', zeros);
    out.write(&tail);
    if s.left {
        out.pad(s.pad, width.into());
    }
}

/// Adds one to the hex digits `digits` in place, as `printf_fphex.c` rounds
/// up. Returns whether it carried out of the first digit.
fn hex_increment(digits: &mut [u8], conv: u8) -> bool {
    let mut cnt = digits.len();
    while cnt > 0 {
        cnt -= 1;
        let ch = digits[cnt];
        if ch == b'9' {
            // '9' + 1 is the letter a, in the case of the conversion.
            digits[cnt] = conv;
            return false;
        } else if ch.to_ascii_lowercase() < b'f' {
            digits[cnt] += 1;
            return false;
        }
        digits[cnt] = b'0';
    }
    true
}

/// `printf_fphex.c` for `%a` and `%A`.
fn float_hex(out: &mut Out, s: Spec, bits: u64) {
    let negative = bits & SIGN_BIT != 0;
    if let Some(word) = special_word(s, bits) {
        special(out, s, negative, word);
        return;
    }
    let upper = s.conv == b'A';
    let mant = bits & MANT_MASK;
    let biased = ((bits & EXP_MASK) >> 52) as i64;
    let zero_mantissa = mant == 0;
    // The 13 hex digits of the stored mantissa.
    let mut numstr = itoa(mant, 16, upper);
    while numstr.len() < 13 {
        numstr.insert(0, b'0');
    }
    let mut leading = if biased == 0 { b'0' } else { b'1' };
    let (exponent, expnegative) = if biased == 0 {
        if zero_mantissa {
            (0, false)
        } else {
            (1022, true)
        }
    } else if biased >= 1023 {
        (biased - 1023, false)
    } else {
        (1023 - biased, true)
    };
    let mut precision = s.prec;
    let numend: usize;
    if zero_mantissa {
        if precision == -1 {
            precision = 0;
        }
        numend = 0;
    } else {
        let mut end = numstr.len();
        while numstr[end - 1] == b'0' {
            end -= 1;
        }
        numend = end;
        let mut round_away = false;
        if precision != -1 && (precision as usize) < numend {
            let p = precision as usize;
            let last = if p > 0 { numstr[p - 1] } else { leading };
            let last_value = crate::clib::digit_value(last);
            let next_value = crate::clib::digit_value(numstr[p]);
            let more_bits = next_value & 7 != 0 || p + 1 < numend;
            // Round to nearest, ties to even.
            round_away = next_value >= 8 && (last_value & 1 == 1 || more_bits);
        }
        if precision == -1 {
            precision = numend as i64;
        } else if round_away && hex_increment(&mut numstr[..precision as usize], s.conv) {
            // All shown digits were f: the leading digit goes up. It is 0
            // (subnormal) or 1 here, so glibc's cases for a leading 9 or f,
            // which also move the exponent, cannot occur.
            leading += 1;
        }
    }
    let expstr = itoa(exponent as u64, 10, false);
    let mut width = s.width;
    width = sub(
        width,
        i64::from(negative || s.showsign || s.space) + 2 + 1 + precision + 1 + 1,
    );
    width = sub(width, expstr.len() as i64);
    if precision > 0 || s.alt {
        width = sub(width, 1);
    }
    if !s.left && s.pad != b'0' {
        out.pad(b' ', width.into());
    }
    if negative {
        out.put(b'-');
    } else if s.showsign {
        out.put(b'+');
    } else if s.space {
        out.put(b' ');
    }
    out.put(b'0');
    out.put(if upper { b'X' } else { b'x' });
    if !s.left && s.pad == b'0' {
        out.pad(b'0', width.into());
    }
    out.put(leading);
    if precision > 0 || s.alt {
        out.put(b'.');
    }
    if precision > 0 {
        let shown = numend.min(precision as usize);
        out.write(&numstr[..shown]);
        out.pad(b'0', precision - numend as i64);
    }
    out.put(if upper { b'P' } else { b'p' });
    out.put(if expnegative { b'-' } else { b'+' });
    out.write(&expstr);
    if s.left && s.pad != b'0' {
        out.pad(s.pad, width.into());
    }
}
