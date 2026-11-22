//! The system calls `misc/` makes, as the port needs them.
//!
//! Charon is told to keep this module opaque: in Lean each function is a
//! declaration without a body, as a system call is on the C side. Paths are
//! bytes; a path with a NUL byte cannot name a file and fails like a missing
//! one.

use std::ffi::OsStr;
use std::fs::File;
use std::io::Write as _;
use std::os::unix::ffi::{OsStrExt as _, OsStringExt};
use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _, OpenOptionsExt as _};

/// `errno` when the standard library reports an error without one, which it
/// does for a path it rejects before any system call.
const EINVAL: i32 = 22;

/// `getenv`.
#[must_use]
pub fn getenv(name: &str) -> Option<Vec<u8>> {
    std::env::var_os(name).map(OsStringExt::into_vec)
}

/// Whether `stat` succeeds on `path`.
#[must_use]
pub fn stat_ok(path: &[u8]) -> bool {
    std::fs::metadata(OsStr::from_bytes(path)).is_ok()
}

/// Whether `stat` succeeds on `path` and reports a directory.
#[must_use]
pub fn stat_is_dir(path: &[u8]) -> bool {
    std::fs::metadata(OsStr::from_bytes(path)).is_ok_and(|st| st.is_dir())
}

/// Whether `stat` succeeds on both paths and reports the same device and
/// inode for them.
#[must_use]
pub fn stat_same_file(a: &[u8], b: &[u8]) -> bool {
    let (Ok(a), Ok(b)) = (
        std::fs::metadata(OsStr::from_bytes(a)),
        std::fs::metadata(OsStr::from_bytes(b)),
    ) else {
        return false;
    };
    a.dev() == b.dev() && a.ino() == b.ino()
}

/// `getcwd`, with a buffer as large as it takes.
#[must_use]
pub fn getcwd() -> Option<Vec<u8>> {
    std::env::current_dir()
        .ok()
        .map(|wd| wd.into_os_string().into_vec())
}

/// `realpath(path, NULL)`.
#[must_use]
pub fn realpath(path: &[u8]) -> Option<Vec<u8>> {
    std::fs::canonicalize(OsStr::from_bytes(path))
        .ok()
        .map(|p| p.into_os_string().into_vec())
}

/// `mkdir(path, 0777)`, ignoring the result as mpv does.
pub fn mkdir(path: &[u8]) {
    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o777);
    let _ = builder.create(OsStr::from_bytes(path));
}

/// `open(path, O_RDWR | O_CREAT | O_EXCL | flags, 0600)`.
///
/// # Errors
///
/// The `errno` of the failed call.
pub fn open_exclusive(path: &[u8], flags: i32) -> Result<File, i32> {
    let opened = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .custom_flags(flags)
        .mode(0o600)
        .open(OsStr::from_bytes(path));
    match opened {
        Ok(file) => Ok(file),
        Err(err) => Err(err.raw_os_error().unwrap_or(EINVAL)),
    }
}

/// `fwrite` of all of `data` followed by `fclose`: whether both worked. A
/// `File` is unbuffered, so the write reports what the C learns from the
/// two calls together.
#[must_use]
pub fn write_and_close(mut file: File, data: &[u8]) -> bool {
    data.is_empty() || file.write_all(data).is_ok()
}

/// `rename`: whether it worked.
#[must_use]
pub fn rename(from: &[u8], to: &[u8]) -> bool {
    std::fs::rename(OsStr::from_bytes(from), OsStr::from_bytes(to)).is_ok()
}

/// `unlink`, ignoring the result as mpv does.
pub fn unlink(path: &[u8]) {
    let _ = std::fs::remove_file(OsStr::from_bytes(path));
}

/// 32 bytes from the operating system as four words, for `mp_rand_seed`,
/// where the C calls `av_random_bytes` of libavutil. `None` if the system
/// has none to give.
#[must_use]
pub fn random_state() -> Option<[u64; 4]> {
    use std::io::Read as _;
    let mut buf = [0_u8; 32];
    let mut file = File::open("/dev/urandom").ok()?;
    file.read_exact(&mut buf).ok()?;
    let mut v = [0_u64; 4];
    for (word, chunk) in v.iter_mut().zip(buf.chunks_exact(8)) {
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(chunk);
        *word = u64::from_ne_bytes(bytes);
    }
    Some(v)
}

/// A seed from the clock, for `mp_rand_seed` when the system has no random
/// bytes. The C also mixes in two addresses, which safe Rust cannot see.
#[must_use]
pub fn clock_seed() -> u64 {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH);
    now.map_or(1, |d| {
        d.as_secs()
            .wrapping_mul(1_000_000_000)
            .wrapping_add(u64::from(d.subsec_nanos()))
    })
}

/// `O_NONBLOCK` on Linux.
const O_NONBLOCK: i32 = 0o4000;

/// The pipe of `mp_make_wakeup_pipe` (`osdep/io.c`): both ends
/// non-blocking and close-on-exec.
///
/// The standard library makes a pipe but cannot set `O_NONBLOCK` on it.
/// Opening `/proc/self/fd/N` opens the same pipe again, with the flags the
/// opener asks for, so each end is opened again that way and the first two
/// descriptors are closed. Without `/proc` this fails, which mpv's callers
/// see as they would a failed `pipe()`.
#[derive(Debug)]
pub struct WakeupPipe {
    read: File,
    write: File,
}

/// Opens the pipe end `fd` again, non-blocking.
fn reopen_nonblocking(fd: i32, write: bool) -> Option<File> {
    std::fs::OpenOptions::new()
        .read(!write)
        .write(write)
        .custom_flags(O_NONBLOCK)
        .open(format!("/proc/self/fd/{fd}"))
        .ok()
}

impl WakeupPipe {
    /// `mp_make_wakeup_pipe`; `None` where it returns -1.
    #[must_use]
    pub fn new() -> Option<Self> {
        use std::os::fd::AsRawFd as _;
        let (r, w) = std::io::pipe().ok()?;
        let read = reopen_nonblocking(r.as_raw_fd(), false)?;
        let write = reopen_nonblocking(w.as_raw_fd(), true)?;
        Some(Self { read, write })
    }

    /// The read end, for `poll`.
    #[must_use]
    pub fn read_fd(&self) -> i32 {
        use std::os::fd::AsRawFd as _;
        self.read.as_raw_fd()
    }

    /// `write(fd[1], &(char){0}, 1)`, ignoring the result as mpv does: on a
    /// full pipe it fails with `EAGAIN`.
    pub fn write_byte(&self) {
        let _ = (&self.write).write(&[0]);
    }

    /// Reads 256 bytes at a time until the pipe is empty, as
    /// `mp_cancel_reset` does: it stops at the first result that is not a
    /// positive count, except `EINTR`.
    pub fn drain(&self) {
        use std::io::Read as _;
        let mut buf = [0_u8; 256];
        loop {
            match (&self.read).read(&mut buf) {
                Ok(n) if n > 0 => {}
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                _ => break,
            }
        }
    }
}
