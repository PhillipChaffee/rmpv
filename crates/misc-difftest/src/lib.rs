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
