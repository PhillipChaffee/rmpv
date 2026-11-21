#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![feature(c_variadic)]
#![feature(core_intrinsics)]
#![feature(extern_types)]
#![feature(raw_ref_op)]

pub mod src {
    pub mod bstr;
    pub mod charset_conv;
    pub mod codepoint_width;
    pub mod dispatch;
    pub mod hash;
    pub mod io_utils;
    pub mod json;
    pub mod language;
    pub mod natural_sort;
    pub mod node;
    pub mod path_utils;
    pub mod random;
    pub mod rendezvous;
    pub mod thread_pool;
    pub mod thread_tools;
} // mod src
