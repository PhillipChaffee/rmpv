//! Port of `misc/io_utils.c`.

use std::fs::File;

use crate::bstr::bstr0;
use crate::clib::{strlen, strncmp};
use crate::os;
use crate::random::{MpRandState, mp_rand_next, mp_rand_seed};

/// `errno` values the C sets itself.
pub const EINVAL: i32 = 22;
pub const EEXIST: i32 = 17;

const O_CLOEXEC: i32 = 0o2_000_000;

/// The six characters `mp_mkostemps` writes for one random value.
///
/// This is what `snprintf(buf, 7, "%06zx", v)` leaves in the buffer: the
/// first six digits of the hexadecimal form. For a value of 2^24 or more
/// the low bits are cut off, not the high ones.
#[must_use]
pub fn mkostemps_name(v: u64) -> [u8; 6] {
    let mut digits = [b'0'; 16];
    let mut n = 0;
    let mut rest = v;
    while rest != 0 {
        let d = (rest & 15) as u8;
        digits[n] = if d < 10 { b'0' + d } else { b'a' + (d - 10) };
        rest >>= 4;
        n += 1;
    }
    let width = if n < 6 { 6 } else { n };
    let mut out = [0_u8; 6];
    let mut i = 0;
    while i < 6 {
        out[i] = digits[width - 1 - i];
        i += 1;
    }
    out
}

/// `mp_mkostemps` with the random state passed in, where the C seeds one
/// from the operating system.
///
/// # Errors
///
/// The `errno` the C leaves when it returns -1.
pub fn mp_mkostemps_with(
    template: &mut [u8],
    suffixlen: i32,
    flags: i32,
    s: &mut MpRandState,
) -> Result<File, i32> {
    let len = strlen(template);
    // `6 + suffixlen` is compared as `size_t`, so a negative sum is huge.
    let need = (6_i32.wrapping_add(suffixlen)) as usize;
    let xs = [b'X', b'X', b'X', b'X', b'X', b'X'];
    if len < need || strncmp(&template[len - need..], &xs, 6) != 0 {
        return Err(EINVAL);
    }
    let t = len - need;

    // The C returns from inside this loop; here it ends by setting `result`,
    // because Aeneas does not translate a return from this loop.
    let mut result = Err(EEXIST);
    let mut done = false;
    let mut attempt: u64 = 0;
    while !done && attempt < u32::MAX as u64 {
        let name = mkostemps_name(mp_rand_next(s));
        template[t..t + 6].copy_from_slice(&name);

        match os::open_exclusive(bstr0(template), flags) {
            Ok(file) => {
                result = Ok(file);
                done = true;
            }
            Err(errno) => {
                if errno != EEXIST {
                    result = Err(errno);
                    done = true;
                }
            }
        }
        attempt += 1;
    }

    result
}

/// `mp_mkostemps`: replaces the `XXXXXX` that sits `suffixlen` bytes before
/// the end of the C string `template` until a file of that name can be
/// created, and returns the open file.
///
/// # Errors
///
/// The `errno` the C leaves when it returns -1.
pub fn mp_mkostemps(template: &mut [u8], suffixlen: i32, flags: i32) -> Result<File, i32> {
    let mut s = mp_rand_seed(0);
    mp_mkostemps_with(template, suffixlen, flags, &mut s)
}

/// `mp_save_to_file`: writes `data` to a temporary file next to `filepath`
/// and renames it over `filepath`. Returns whether that worked.
#[must_use]
pub fn mp_save_to_file(filepath: &[u8], data: &[u8]) -> bool {
    let filepath = bstr0(filepath);
    let mut tmp = filepath.to_vec();
    tmp.extend_from_slice(&[b'X', b'X', b'X', b'X', b'X', b'X']);
    let file = match mp_mkostemps(&mut tmp, 0, O_CLOEXEC) {
        Ok(file) => file,
        Err(_) => return false,
    };
    if os::write_and_close(file, data) {
        os::rename(&tmp, filepath)
    } else {
        os::unlink(&tmp);
        false
    }
}
