//! Support for the differential tests of `rmpv-misc` against mpv's C.
//!
//! [`ffi`] declares mpv's functions as `libmpv.a` exports them, and the
//! `dt_*` entry points of the C shims in `shim/`. [`input`] generates test
//! inputs from fixed seeds. [`node_wire`] carries `mpv_node` trees across
//! the boundary.
//!
//! The crate is empty unless the build script found mpv's static build.

#![cfg(mpv_build)]

pub mod ffi;
pub mod input;
pub mod node_wire;

// Only the integration tests call the port.
use rmpv_misc as _;

/// Calls a C function declared in [`ffi`].
#[macro_export]
macro_rules! c {
    ($f:ident($($a:expr),* $(,)?)) => {{
        // SAFETY: the callers in this crate's tests pass pointers that
        // `ffi::Bstr::of`, `input::cstring` or a local buffer produced: each
        // points to memory that lives across the call and is as long as the
        // length passed with it, and each C string ends in a NUL byte. The
        // declarations in `ffi` match the C prototypes.
        unsafe { $crate::ffi::$f($($a),*) }
    }};
}

/// Copies `len` bytes out of a buffer a C function returned.
///
/// # Panics
///
/// If `start` is NULL while `len` is not 0.
#[must_use]
#[expect(
    clippy::not_unsafe_ptr_arg_deref,
    reason = "test helper, always applied directly to what a C call returned"
)]
pub fn copy_out(start: *const u8, len: usize) -> Vec<u8> {
    if len == 0 {
        return Vec::new();
    }
    assert!(!start.is_null());
    // SAFETY: the caller got `start` and `len` from C code that says `len`
    // bytes are readable at `start`, and they stay so until the caller frees
    // the buffer after this copy.
    unsafe { std::slice::from_raw_parts(start, len) }.to_vec()
}

/// Copies a NUL-terminated string a C function returned, without the NUL.
///
/// # Panics
///
/// If `s` is NULL.
#[must_use]
#[expect(
    clippy::not_unsafe_ptr_arg_deref,
    reason = "test helper, always applied directly to what a C call returned"
)]
pub fn copy_cstr(s: *const std::ffi::c_char) -> Vec<u8> {
    assert!(!s.is_null());
    // SAFETY: the caller got `s` from C code that returns a NUL-terminated
    // string, which stays valid until the caller frees it after this copy.
    unsafe { std::ffi::CStr::from_ptr(s) }.to_bytes().to_vec()
}

/// Takes ownership of a `malloc`ed buffer from a shim and frees it.
#[must_use]
pub fn take_malloced(start: *mut u8, len: usize) -> Vec<u8> {
    let out = copy_out(start, len);
    c!(free(start.cast()));
    out
}

/// Where a view sits in the buffer it was cut from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct View {
    /// Offset of the first byte from the start of the buffer. `None` for a
    /// C view whose `start` is NULL.
    pub offset: Option<usize>,
    pub len: usize,
}

/// The position of a C view within `base`.
#[must_use]
pub fn c_view(base: &[u8], v: ffi::Bstr) -> View {
    let offset = if v.start.is_null() {
        None
    } else {
        Some((v.start as usize).wrapping_sub(base.as_ptr() as usize))
    };
    View { offset, len: v.len }
}

/// The position of a Rust view within `base`.
#[must_use]
pub fn r_view(base: &[u8], v: &[u8]) -> View {
    View {
        offset: Some((v.as_ptr() as usize).wrapping_sub(base.as_ptr() as usize)),
        len: v.len(),
    }
}

/// Whether a C view and a Rust view are the same part of the buffer. A NULL
/// C view matches any empty Rust view, as the port has no NULL views.
#[must_use]
pub fn same_view(c: View, r: View) -> bool {
    c.len == r.len && (c.offset == r.offset || (c.offset.is_none() && c.len == 0))
}

/// The sign of a comparison result: all that C defines for `memcmp` and its
/// relatives.
#[must_use]
pub fn sign(v: i32) -> i32 {
    v.signum()
}

/// How many bytes are waiting to be read from `fd` (`FIONREAD`).
///
/// # Panics
///
/// If `ioctl` fails.
#[must_use]
pub fn fd_pending(fd: std::ffi::c_int) -> i32 {
    const FIONREAD: std::ffi::c_ulong = 0x541b;
    let mut n: std::ffi::c_int = 0;
    let r = c!(ioctl(fd, FIONREAD, &raw mut n));
    assert_eq!(r, 0, "ioctl(FIONREAD) on {fd}");
    n
}

/// Whether `poll` reports `fd` readable now.
#[must_use]
pub fn fd_readable(fd: std::ffi::c_int) -> bool {
    const POLLIN: i16 = 1;
    let mut p = ffi::PollFd {
        fd,
        events: POLLIN,
        revents: 0,
    };
    let r = c!(poll(&raw mut p, 1, 0));
    r == 1 && p.revents & POLLIN != 0
}

/// Runs `f` on its own thread and fails the calling test if it has not
/// finished after `seconds`.
///
/// A hung test then fails instead of blocking the run. The hung thread is
/// left behind; the test binary exits when the test harness does.
///
/// # Panics
///
/// On the timeout, and with `f`'s panic if it panics.
pub fn with_timeout<T: Send + 'static>(
    name: &str,
    seconds: u64,
    f: impl FnOnce() -> T + Send + 'static,
) -> T {
    let (tx, rx) = std::sync::mpsc::channel();
    let worker = std::thread::Builder::new()
        .name(name.to_owned())
        .spawn(move || {
            let _ = tx.send(std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)));
        })
        .expect("spawn the test thread");
    match rx.recv_timeout(std::time::Duration::from_secs(seconds)) {
        Ok(Ok(v)) => {
            worker.join().expect("test thread");
            v
        }
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(
            std::sync::mpsc::RecvTimeoutError::Timeout
            | std::sync::mpsc::RecvTimeoutError::Disconnected,
        ) => {
            panic!("{name}: no result after {seconds} s, the test hangs")
        }
    }
}
