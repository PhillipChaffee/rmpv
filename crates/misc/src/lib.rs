//! Safe Rust port of mpv's `misc/` folder.
//!
//! One module per C file, one function per C function, with mpv's names. The
//! conventions that relate each function to its C original are in the crate's
//! `README.md`.

#![forbid(unsafe_code)]

pub mod clib;
pub mod common;
pub mod ctype;

pub mod bstr;
pub mod charset_conv;
pub mod codepoint_width;
pub mod hash;
pub mod io_utils;
pub mod json;
pub mod language;
pub mod natural_sort;
pub mod node;
pub mod path_utils;
pub mod random;
