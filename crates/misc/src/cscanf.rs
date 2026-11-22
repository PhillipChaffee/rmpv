//! `vsscanf` as glibc 2.39 behaves in the "C" locale, for `bstr_sscanf`.
//!
//! mpv's build calls `__isoc23_vsscanf` (glibc's C23 entry point, see trap 1
//! in `docs/pilot-misc-rust.md`): `%i` accepts a `0b` prefix and `%b` reads
//! binary. The interpreter follows `stdio-common/vfscanf-internal.c`
//! statement by statement, including where it reads one character more than
//! it uses, so that `%n` and the assignment count come out the same.
//!
//! Each C pointer argument is a [`ScanArg`]. What the port does not model is
//! refused with [`ScanfError::Unsupported`]: positional arguments (`%1$d`),
//! the `I` flag, the `m` allocation flag, the `wN` length modifiers, wide
//! characters and strings (`%lc`, `%ls`, `%l[`, `%C`, `%S`), `long double`
//! (`%Lf`), and `%p`. An argument of the wrong kind, or a missing one, is
//! undefined behaviour in C and is refused with [`ScanfError::BadArgument`],
//! but only when glibc would read the argument: like the C, a conversion
//! that fails before it assigns takes nothing.
//!
//! Charon is told to leave this module out, as it holds `f32` and `f64`.

use crate::cfloat::{strtod, strtof};
use crate::clib::{cstr_at, digit_value, isspace, scan_integer};

/// One pointer argument of a `scanf` call. A conversion writes through the
/// pointer of its width whatever its signedness, as the C does.
#[derive(Debug)]
pub enum ScanArg<'a> {
    /// `signed char *` (`%hhd`, `%hhn`).
    I8(&'a mut i8),
    /// `unsigned char *` (`%hhu`, `%hhx`).
    U8(&'a mut u8),
    /// `short *` (`%hd`, `%hn`).
    I16(&'a mut i16),
    /// `unsigned short *` (`%hu`).
    U16(&'a mut u16),
    /// `int *` (`%d`, `%i`, `%n`).
    I32(&'a mut i32),
    /// `unsigned int *` (`%u`, `%x`, `%o`, `%b`).
    U32(&'a mut u32),
    /// `long *` and `long long *` (`%ld`, `%lld`, `%zd`, `%jd`, `%ln`).
    I64(&'a mut i64),
    /// `unsigned long *` and `unsigned long long *` (`%lu`, `%zu`).
    U64(&'a mut u64),
    /// `float *` (`%f`, `%e`, `%g`, `%a`).
    F32(&'a mut f32),
    /// `double *` (`%lf`, and `%zf`, `%jf`, `%tf`, which glibc reads alike).
    F64(&'a mut f64),
    /// `char *` for `%s`, `%c` and `%[`: the port sets the `Vec` to the
    /// bytes stored. The C writes them at the start of the caller's buffer,
    /// followed by a NUL for `%s` and `%[`.
    Bytes(&'a mut Vec<u8>),
}

/// Why [`vsscanf`] did not run to its result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanfError {
    /// The conversion at this byte offset of the format is outside what the
    /// port models (see the module documentation).
    Unsupported(usize),
    /// The argument with this index is missing, or is not of the kind its
    /// conversion writes. In C this is undefined behaviour.
    BadArgument(usize),
}

/// `EOF`.
const EOF: i32 = -1;

/// The input as glibc's `inchar` and `ungetc` see it: a cursor, the last
/// character read (`None` for `EOF`, which sticks once read), and
/// `read_in`, the count `%n` reports.
struct Input<'s> {
    s: &'s [u8],
    pos: usize,
    c: Option<u8>,
    read_in: usize,
}

impl Input<'_> {
    /// `inchar()`.
    fn inchar(&mut self) -> Option<u8> {
        self.c?;
        let b = cstr_at(self.s, self.pos);
        if b == 0 {
            self.c = None;
        } else {
            self.pos += 1;
            self.read_in += 1;
            self.c = Some(b);
        }
        self.c
    }

    /// `ungetc(c, s)`: pushes back the last character unless it was `EOF`.
    fn ungetc(&mut self) {
        if self.c.is_some() {
            self.pos -= 1;
            self.read_in -= 1;
        }
    }
}

/// How a directive ended: glibc's `conv_error()` and `input_error()`, or a
/// refusal of the port.
enum Stop {
    Conv,
    Input,
    Refused(ScanfError),
}

impl From<ScanfError> for Stop {
    fn from(e: ScanfError) -> Self {
        Self::Refused(e)
    }
}

/// glibc's conversion flags that matter in the "C" locale.
#[expect(
    clippy::struct_excessive_bools,
    reason = "glibc's own flag bits; `hh`, `h` and `l` exclude each other"
)]
#[derive(Clone, Copy, Default)]
struct Flags {
    suppress: bool,
    char_: bool,
    short: bool,
    long: bool,
    long_double: bool,
}

fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn is_xdigit(c: u8) -> bool {
    digit_value(c) < 16
}

/// glibc's `read_int`: `None` above `INT_MAX`.
fn read_int(f: &[u8], i: &mut usize) -> Option<i32> {
    let mut v: Option<i32> = Some(0);
    while is_digit(cstr_at(f, *i)) {
        let d = (cstr_at(f, *i) - b'0') as i32;
        v = v
            .and_then(|n| n.checked_mul(10))
            .and_then(|n| n.checked_add(d));
        *i += 1;
    }
    v
}

struct Args<'a, 'b> {
    list: &'b mut [ScanArg<'a>],
    next: usize,
}

impl<'a> Args<'a, '_> {
    /// `ARG (type *)`: the next argument.
    fn take(&mut self) -> Result<(usize, &mut ScanArg<'a>), ScanfError> {
        let i = self.next;
        self.next += 1;
        match self.list.get_mut(i) {
            Some(a) => Ok((i, a)),
            None => Err(ScanfError::BadArgument(i)),
        }
    }

    /// Stores an integer through a pointer of the width `flags` select.
    fn store_int(&mut self, flags: Flags, v: u64) -> Result<(), ScanfError> {
        let (i, arg) = self.take()?;
        match arg {
            ScanArg::I64(p) if flags.long => **p = v as i64,
            ScanArg::U64(p) if flags.long => **p = v,
            ScanArg::I16(p) if !flags.long && flags.short => **p = v as i16,
            ScanArg::U16(p) if !flags.long && flags.short => **p = v as u16,
            ScanArg::I32(p) if !flags.long && !flags.short && !flags.char_ => **p = v as i32,
            ScanArg::U32(p) if !flags.long && !flags.short && !flags.char_ => **p = v as u32,
            ScanArg::I8(p) if !flags.long && !flags.short && flags.char_ => **p = v as i8,
            ScanArg::U8(p) if !flags.long && !flags.short && flags.char_ => **p = v as u8,
            _ => return Err(ScanfError::BadArgument(i)),
        }
        Ok(())
    }

    fn bytes(&mut self) -> Result<&mut Vec<u8>, ScanfError> {
        match self.take()? {
            (_, ScanArg::Bytes(v)) => Ok(v),
            (i, _) => Err(ScanfError::BadArgument(i)),
        }
    }
}

/// `sscanf(s, fmt, args...)` with the arguments in `args`; `s` and `fmt`
/// are C strings. Returns what the C returns: the number of assignments,
/// or `EOF` (-1) if the input ended before the first one.
///
/// # Errors
///
/// The port's refusals; see [`ScanfError`].
pub fn vsscanf(s: &[u8], fmt: &[u8], args: &mut [ScanArg<'_>]) -> Result<i32, ScanfError> {
    let mut input = Input {
        s,
        pos: 0,
        c: Some(0),
        read_in: 0,
    };
    let mut args = Args {
        list: args,
        next: 0,
    };
    let mut done: i32 = 0;
    let mut skip_space = false;
    let mut f = 0;
    while cstr_at(fmt, f) != 0 {
        let r = directive(
            &mut input,
            fmt,
            &mut f,
            &mut args,
            &mut done,
            &mut skip_space,
        );
        match r {
            Ok(()) => {}
            Err(Stop::Conv) => return Ok(done),
            Err(Stop::Input) => return Ok(if done == 0 { EOF } else { done }),
            Err(Stop::Refused(e)) => return Err(e),
        }
    }
    // glibc then eats trailing white space, which nothing can observe.
    Ok(done)
}

/// A byte of the format other than `%`.
fn literal(input: &mut Input<'_>, fc: u8, skip_space: &mut bool) -> Result<(), Stop> {
    // A byte above 0x7f is no multibyte character in the "C" locale, so
    // glibc treats it as an ordinary one.
    if isspace(fc) {
        *skip_space = true;
        return Ok(());
    }
    let mut c = input.inchar().ok_or(Stop::Input)?;
    if *skip_space {
        while isspace(c) {
            c = input.inchar().ok_or(Stop::Input)?;
        }
        *skip_space = false;
    }
    if c != fc {
        input.ungetc();
        return Err(Stop::Conv);
    }
    Ok(())
}

/// The flags, width and length modifier of the conversion that starts at
/// `fmt[start]` (`%`); `*f` is just after the `%` and is left on the
/// conversion character. A width of -1 means none.
fn parse_spec(fmt: &[u8], f: &mut usize, start: usize) -> Result<(Flags, i32), Stop> {
    let unsupported = Stop::Refused(ScanfError::Unsupported(start));
    let mut flags = Flags::default();
    // A number right after the % is a position if a $ follows, else the
    // width; in the second case the * and ' flags are not looked for.
    let mut width: i32 = 0;
    let mut have_width = false;
    if is_digit(cstr_at(fmt, *f)) {
        let n = read_int(fmt, f);
        if cstr_at(fmt, *f) == b'$' {
            return Err(unsupported);
        }
        width = n.unwrap_or(-1);
        have_width = true;
    }
    if !have_width {
        loop {
            match cstr_at(fmt, *f) {
                b'*' => flags.suppress = true,
                // Grouping: the "C" locale has no thousands separator.
                b'\'' => {}
                b'I' => return Err(unsupported),
                _ => break,
            }
            *f += 1;
        }
        if is_digit(cstr_at(fmt, *f)) {
            width = read_int(fmt, f).unwrap_or(-1);
        }
    }
    if width == 0 {
        width = -1;
    }
    match cstr_at(fmt, *f) {
        b'h' => {
            *f += 1;
            if cstr_at(fmt, *f) == b'h' {
                *f += 1;
                flags.char_ = true;
            } else {
                flags.short = true;
            }
        }
        b'l' => {
            *f += 1;
            if cstr_at(fmt, *f) == b'l' {
                *f += 1;
                flags.long_double = true;
            }
            flags.long = true;
        }
        b'q' | b'L' => {
            *f += 1;
            flags.long_double = true;
            flags.long = true;
        }
        // size_t, intmax_t and ptrdiff_t are as wide as long on LP64.
        b'z' | b'j' | b't' => {
            *f += 1;
            flags.long = true;
        }
        b'm' | b'w' => return Err(unsupported),
        // `a` is a conversion, not a flag, in the C99 and later modes.
        _ => {}
    }
    Ok((flags, width))
}

/// One literal byte, white space run or conversion of the format.
fn directive(
    input: &mut Input<'_>,
    fmt: &[u8],
    f: &mut usize,
    args: &mut Args<'_, '_>,
    done: &mut i32,
    skip_space: &mut bool,
) -> Result<(), Stop> {
    let start = *f;
    let fc = cstr_at(fmt, *f);
    *f += 1;
    if fc != b'%' {
        return literal(input, fc, skip_space);
    }
    let (flags, width) = parse_spec(fmt, f, start)?;
    let unsupported = Stop::Refused(ScanfError::Unsupported(start));
    if cstr_at(fmt, *f) == 0 {
        return Err(Stop::Conv);
    }
    let fc = cstr_at(fmt, *f);
    *f += 1;
    if *skip_space || (fc != b'[' && fc != b'c' && fc != b'C' && fc != b'n') {
        // Eat white space. Reaching the end here is no error yet.
        while input.c.is_some() {
            match input.inchar() {
                Some(c) if isspace(c) => {}
                _ => break,
            }
        }
        input.ungetc();
        *skip_space = false;
    }
    match fc {
        b'%' => {
            let c = input.inchar().ok_or(Stop::Input)?;
            if c != b'%' {
                input.ungetc();
                return Err(Stop::Conv);
            }
        }
        b'n' => {
            if !flags.suppress {
                args.store_int(flags, input.read_in as u64)?;
            }
        }
        b'c' | b's' | b'[' if flags.long => return Err(unsupported),
        b'c' => {
            scan_chars(input, args, flags, width)?;
            if !flags.suppress {
                *done += 1;
            }
        }
        b's' => {
            scan_string(input, args, flags, width)?;
            if !flags.suppress {
                *done += 1;
            }
        }
        b'[' => {
            scan_set(input, fmt, f, args, flags, width)?;
            if !flags.suppress {
                *done += 1;
            }
        }
        b'x' | b'X' | b'o' | b'b' | b'u' | b'd' | b'i' => {
            let (base, signed) = match fc {
                b'x' | b'X' => (16, false),
                b'o' => (8, false),
                b'b' => (2, false),
                b'u' => (10, false),
                b'd' => (10, true),
                _ => (0, true),
            };
            scan_number(input, args, flags, width, base, signed)?;
            if !flags.suppress {
                *done += 1;
            }
        }
        b'e' | b'E' | b'f' | b'F' | b'g' | b'G' | b'a' | b'A' => {
            if flags.long_double {
                return Err(unsupported);
            }
            scan_float(input, args, flags, width)?;
            if !flags.suppress {
                *done += 1;
            }
        }
        b'C' | b'S' | b'p' => return Err(unsupported),
        // An unknown conversion character.
        _ => return Err(Stop::Conv),
    }
    Ok(())
}

/// `%c`: `width` characters (1 if none was given), stored without a NUL.
fn scan_chars(
    input: &mut Input<'_>,
    args: &mut Args<'_, '_>,
    flags: Flags,
    width: i32,
) -> Result<(), Stop> {
    let mut width = if width == -1 { 1 } else { width };
    let mut out = if flags.suppress {
        None
    } else {
        Some(args.bytes()?)
    };
    let mut c = input.inchar().ok_or(Stop::Input)?;
    let mut stored = Vec::new();
    loop {
        stored.push(c);
        width -= 1;
        if width <= 0 {
            break;
        }
        match input.inchar() {
            Some(next) => c = next,
            None => break,
        }
    }
    if let Some(out) = out.as_mut() {
        **out = stored;
    }
    Ok(())
}

/// `%s`: characters up to white space.
fn scan_string(
    input: &mut Input<'_>,
    args: &mut Args<'_, '_>,
    flags: Flags,
    width: i32,
) -> Result<(), Stop> {
    let mut width = width;
    let mut out = if flags.suppress {
        None
    } else {
        Some(args.bytes()?)
    };
    let mut c = input.inchar().ok_or(Stop::Input)?;
    let mut stored = Vec::new();
    loop {
        if isspace(c) {
            input.ungetc();
            break;
        }
        stored.push(c);
        if width > 0 {
            width -= 1;
            if width == 0 {
                break;
            }
        }
        match input.inchar() {
            Some(next) => c = next,
            None => break,
        }
    }
    if let Some(out) = out.as_mut() {
        **out = stored;
    }
    Ok(())
}

/// `%[`: the set at `fmt[*f]`, then the longest run of characters in it.
fn scan_set(
    input: &mut Input<'_>,
    fmt: &[u8],
    f: &mut usize,
    args: &mut Args<'_, '_>,
    flags: Flags,
    width: i32,
) -> Result<(), Stop> {
    let mut out = if flags.suppress {
        None
    } else {
        Some(args.bytes()?)
    };
    let not_in = cstr_at(fmt, *f) == b'^';
    if not_in {
        *f += 1;
    }
    let mut set = [false; 256];
    let first = cstr_at(fmt, *f);
    if first == b']' || first == b'-' {
        // Before any other character, these are members.
        set[first as usize] = true;
        *f += 1;
    }
    let mut fc;
    loop {
        fc = cstr_at(fmt, *f);
        *f += 1;
        if fc == 0 || fc == b']' {
            break;
        }
        let next = cstr_at(fmt, *f);
        let before = if *f >= 2 { cstr_at(fmt, *f - 2) } else { 0 };
        if fc == b'-' && next != 0 && next != b']' && before <= next {
            // The range from the character before the '-' up to the one
            // after it, which the next round adds.
            let mut b = before;
            while b < next {
                set[b as usize] = true;
                b += 1;
            }
        } else {
            set[fc as usize] = true;
        }
    }
    if fc == 0 {
        // The format ended inside the set.
        return Err(Stop::Conv);
    }
    let now = input.read_in;
    let mut c = input.inchar().ok_or(Stop::Input)?;
    let mut width = width;
    let mut stored = Vec::new();
    loop {
        if set[c as usize] == not_in {
            input.ungetc();
            break;
        }
        stored.push(c);
        if width >= 0 {
            width -= 1;
            if width <= 0 {
                break;
            }
        }
        match input.inchar() {
            Some(next) => c = next,
            None => break,
        }
    }
    if now == input.read_in {
        return Err(Stop::Conv);
    }
    if let Some(out) = out.as_mut() {
        **out = stored;
    }
    Ok(())
}

/// glibc's `number:` label: `%d`, `%i`, `%u`, `%o`, `%x`, `%X` and `%b`.
fn scan_number(
    input: &mut Input<'_>,
    args: &mut Args<'_, '_>,
    flags: Flags,
    width: i32,
    base: u32,
    signed: bool,
) -> Result<(), Stop> {
    let mut width = width;
    let mut base = base;
    let mut buf: Vec<u8> = Vec::new();
    let c = input.inchar().ok_or(Stop::Input)?;
    let mut cur = Some(c);
    if c == b'-' || c == b'+' {
        buf.push(c);
        if width > 0 {
            width -= 1;
        }
        cur = input.inchar();
    }
    if width != 0 && cur == Some(b'0') {
        if width > 0 {
            width -= 1;
        }
        buf.push(b'0');
        cur = input.inchar();
        let lower = cur.map(|b| b.to_ascii_lowercase());
        if width != 0 && lower == Some(b'x') {
            if base == 0 {
                base = 16;
            }
            if base == 16 {
                if width > 0 {
                    width -= 1;
                }
                cur = input.inchar();
            }
        } else if width != 0 && lower == Some(b'b') && (base == 2 || base == 0) {
            // C23: a 0b prefix, for %b and for %i.
            base = 2;
            if width > 0 {
                width -= 1;
            }
            cur = input.inchar();
        } else if base == 0 {
            base = 8;
        }
    }
    if base == 0 {
        base = 10;
    }
    while let Some(ch) = cur {
        if width == 0 {
            break;
        }
        let ok = if base == 16 {
            is_xdigit(ch)
        } else {
            is_digit(ch) && ((ch - b'0') as u32) < base
        };
        if !ok {
            break;
        }
        buf.push(ch);
        if width > 0 {
            width -= 1;
        }
        cur = input.inchar();
    }
    if buf.is_empty() || (buf.len() == 1 && (buf[0] == b'+' || buf[0] == b'-')) {
        input.ungetc();
        return Err(Stop::Conv);
    }
    input.ungetc();
    // strtol or strtoul on what was read, which is a sign and digits.
    let (negative, acc, _, overflow) = scan_integer(&buf, base as i32, false);
    let value: u64 = if signed {
        let limit: u64 = if negative { 1 << 63 } else { (1 << 63) - 1 };
        if overflow || acc > limit {
            if negative {
                i64::MIN as u64
            } else {
                i64::MAX as u64
            }
        } else if negative {
            0_u64.wrapping_sub(acc)
        } else {
            acc
        }
    } else if overflow {
        u64::MAX
    } else if negative {
        0_u64.wrapping_sub(acc)
    } else {
        acc
    };
    if !flags.suppress {
        args.store_int(flags, value)?;
    }
    Ok(())
}

/// Reads the next character for the float scanner: `None` where glibc's
/// test `width == 0 || inchar () == EOF` holds.
fn next_within(input: &mut Input<'_>, width: i32) -> Option<u8> {
    if width == 0 {
        return None;
    }
    input.inchar()
}

/// Matches the rest of a word (`nan`, `inf`, `inity`) case-insensitively,
/// as glibc does one character at a time; any mismatch is a `conv_error`.
fn scan_word(
    input: &mut Input<'_>,
    buf: &mut Vec<u8>,
    width: &mut i32,
    word: &[u8],
) -> Result<(), Stop> {
    for &want in word {
        let c = next_within(input, *width).ok_or(Stop::Conv)?;
        if c.to_ascii_lowercase() != want {
            return Err(Stop::Conv);
        }
        if *width > 0 {
            *width -= 1;
        }
        buf.push(c);
    }
    Ok(())
}

/// `nan`, `inf` or `infinity` after their first letter `c`, which is in
/// `buf` already.
fn scan_nan_inf(
    input: &mut Input<'_>,
    buf: &mut Vec<u8>,
    width: &mut i32,
    c: u8,
) -> Result<(), Stop> {
    if c.eq_ignore_ascii_case(&b'n') {
        return scan_word(input, buf, width, b"an");
    }
    scan_word(input, buf, width, b"nf")?;
    if *width != 0
        && let Some(c) = input.inchar()
    {
        if c.eq_ignore_ascii_case(&b'i') {
            if *width > 0 {
                *width -= 1;
            }
            buf.push(c);
            scan_word(input, buf, width, b"nity")?;
        } else {
            input.ungetc();
        }
    }
    Ok(())
}

/// `%e`, `%f`, `%g`, `%a` and their capitals.
fn scan_float(
    input: &mut Input<'_>,
    args: &mut Args<'_, '_>,
    flags: Flags,
    width: i32,
) -> Result<(), Stop> {
    let mut width = width;
    let mut buf: Vec<u8> = Vec::new();
    let mut c = input.inchar().ok_or(Stop::Input)?;
    if width > 0 {
        width -= 1;
    }
    let mut got_sign = false;
    if c == b'-' || c == b'+' {
        got_sign = true;
        buf.push(c);
        // Only before the first character is the end an input error.
        c = next_within(input, width).ok_or(Stop::Conv)?;
        if width > 0 {
            width -= 1;
        }
    }
    if c.eq_ignore_ascii_case(&b'n') || c.eq_ignore_ascii_case(&b'i') {
        buf.push(c);
        scan_nan_inf(input, &mut buf, &mut width, c)?;
        return convert_float(&buf, args, flags);
    }
    let mut hexa = false;
    let mut exp_char = b'e';
    let mut got_digit = false;
    let mut got_dot = false;
    let mut got_e = false;
    let mut cur = Some(c);
    if width != 0 && c == b'0' {
        buf.push(c);
        cur = input.inchar();
        if width > 0 {
            width -= 1;
        }
        if let Some(x) = cur.filter(|b| width != 0 && b.eq_ignore_ascii_case(&b'x')) {
            buf.push(x);
            hexa = true;
            exp_char = b'p';
            cur = input.inchar();
            if width > 0 {
                width -= 1;
            }
        } else {
            got_digit = true;
        }
    }
    loop {
        // glibc tests the character even after the end of the input; an
        // EOF matches nothing and is pushed back as nothing.
        let ch = cur;
        let last = buf.last().copied();
        match ch {
            Some(ch) if is_digit(ch) => {
                buf.push(ch);
                got_digit = true;
            }
            Some(ch) if !got_e && hexa && is_xdigit(ch) => {
                buf.push(ch);
                got_digit = true;
            }
            Some(ch) if got_e && last == Some(exp_char) && (ch == b'-' || ch == b'+') => {
                buf.push(ch);
            }
            Some(ch) if got_digit && !got_e && ch.to_ascii_lowercase() == exp_char => {
                buf.push(exp_char);
                got_e = true;
                got_dot = true;
            }
            Some(b'.') if !got_dot => {
                buf.push(b'.');
                got_dot = true;
            }
            _ => {
                input.ungetc();
                break;
            }
        }
        cur = next_within(input, width);
        if cur.is_none() {
            break;
        }
        if width > 0 {
            width -= 1;
        }
    }
    let sign = usize::from(got_sign);
    if buf.len() == sign || (hexa && buf.len() == 2 + sign) {
        return Err(Stop::Conv);
    }
    convert_float(&buf, args, flags)
}

/// glibc's `scan_float:` label: `strtof` or `strtod` on what was read. It
/// may use less than all of it ("1e" is 1); a value is stored if it used
/// anything.
fn convert_float(buf: &[u8], args: &mut Args<'_, '_>, flags: Flags) -> Result<(), Stop> {
    if flags.long {
        let d = strtod(buf);
        if d.consumed == 0 {
            return Err(Stop::Conv);
        }
        if !flags.suppress {
            match args.take()? {
                (_, ScanArg::F64(p)) => **p = f64::from_bits(d.bits),
                (i, _) => return Err(ScanfError::BadArgument(i).into()),
            }
        }
    } else {
        let (bits, consumed) = strtof(buf);
        if consumed == 0 {
            return Err(Stop::Conv);
        }
        if !flags.suppress {
            match args.take()? {
                (_, ScanArg::F32(p)) => **p = f32::from_bits(bits),
                (i, _) => return Err(ScanfError::BadArgument(i).into()),
            }
        }
    }
    Ok(())
}
