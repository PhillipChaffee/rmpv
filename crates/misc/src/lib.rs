//! Safe Rust port of mpv's `misc/` folder.
//!
//! One module per C file, one function per C function, with mpv's names. The
//! conventions that relate each function to its C original are in the crate's
//! `README.md`.
//!
//! Charon reads the crate with `--cfg aeneas`. Under it the types derive no
//! traits: Aeneas rejects a derived implementation on a recursive type, and
//! the port itself uses none of them.

#![forbid(unsafe_code)]

pub mod clib;
pub mod common;
pub mod ctype;

pub mod bstr;
pub mod cfloat;
pub mod charset_conv;
pub mod codepoint_width;
pub mod hash;
pub mod io_utils;
pub mod json;
pub mod language;
pub mod natural_sort;
pub mod node;
pub mod os;
pub mod path_utils;
pub mod random;
pub mod rendezvous;
pub mod thread_tools;
