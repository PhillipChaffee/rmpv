extern "C" {
    fn av_random_bytes(buf: *mut uint8_t, len: size_t) -> ::core::ffi::c_int;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn mp_round_next_power_of_2(v: uint32_t) -> uint32_t;
    fn mp_raw_time_ns() -> uint64_t;
}
pub type __uint8_t = u8;
pub type __uint32_t = u32;
pub type __uint64_t = u64;
pub type uint8_t = __uint8_t;
pub type uint32_t = __uint32_t;
pub type uint64_t = __uint64_t;
pub type uintptr_t = usize;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_rand_state {
    pub v: [uint64_t; 4],
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 65] = unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"uint32_t mp_rand_in_range32(mp_rand_state *, uint32_t, uint32_t)\0",
    )
};
#[inline]
unsafe extern "C" fn rotl_u64(x: uint64_t, k: ::core::ffi::c_int) -> uint64_t {
    return x << k | x >> 64 as ::core::ffi::c_int - k;
}
#[inline]
unsafe extern "C" fn splitmix64(x: *mut uint64_t) -> uint64_t {
    *x = (*x as ::core::ffi::c_ulong).wrapping_add(0x9e3779b97f4a7c15 as ::core::ffi::c_ulong)
        as uint64_t as uint64_t;
    let mut z: uint64_t = *x;
    z = (z ^ z >> 30 as ::core::ffi::c_int).wrapping_mul(0xbf58476d1ce4e5b9 as uint64_t);
    z = (z ^ z >> 27 as ::core::ffi::c_int).wrapping_mul(0x94d049bb133111eb as uint64_t);
    return z ^ z >> 31 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mp_rand_seed(mut seed: uint64_t) -> mp_rand_state {
    let mut ret: mp_rand_state = mp_rand_state { v: [0; 4] };
    if seed == 0 as uint64_t {
        if av_random_bytes(
            &raw mut ret.v as *mut uint64_t as *mut ::core::ffi::c_void as *mut uint8_t,
            ::core::mem::size_of::<[uint64_t; 4]>() as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            return ret;
        }
        seed = mp_raw_time_ns();
        seed = (seed as ::core::ffi::c_ulong
            ^ ::core::mem::transmute::<
                Option<unsafe extern "C" fn(uint64_t) -> mp_rand_state>,
                uintptr_t,
            >(Some(
                mp_rand_seed as unsafe extern "C" fn(uint64_t) -> mp_rand_state,
            )) as ::core::ffi::c_ulong) as uint64_t;
        seed = (seed as ::core::ffi::c_ulong)
            .wrapping_add(&raw mut ret as uintptr_t as ::core::ffi::c_ulong)
            as uint64_t as uint64_t;
    }
    ret.v[0 as ::core::ffi::c_int as usize] = seed;
    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        ret.v[i as usize] = splitmix64(&raw mut seed);
        i += 1;
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn mp_rand_next(mut s: *mut mp_rand_state) -> uint64_t {
    let mut result: uint64_t = 0;
    let mut t: uint64_t = 0;
    let mut state: *mut uint64_t = &raw mut (*s).v as *mut uint64_t;
    result = rotl_u64(
        (*state.offset(1 as ::core::ffi::c_int as isize)).wrapping_mul(5 as uint64_t),
        7 as ::core::ffi::c_int,
    )
    .wrapping_mul(9 as uint64_t);
    t = *state.offset(1 as ::core::ffi::c_int as isize) << 17 as ::core::ffi::c_int;
    *state.offset(2 as ::core::ffi::c_int as isize) ^=
        *state.offset(0 as ::core::ffi::c_int as isize);
    *state.offset(3 as ::core::ffi::c_int as isize) ^=
        *state.offset(1 as ::core::ffi::c_int as isize);
    *state.offset(1 as ::core::ffi::c_int as isize) ^=
        *state.offset(2 as ::core::ffi::c_int as isize);
    *state.offset(0 as ::core::ffi::c_int as isize) ^=
        *state.offset(3 as ::core::ffi::c_int as isize);
    *state.offset(2 as ::core::ffi::c_int as isize) ^= t;
    *state.offset(3 as ::core::ffi::c_int as isize) = rotl_u64(
        *state.offset(3 as ::core::ffi::c_int as isize),
        45 as ::core::ffi::c_int,
    );
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn mp_rand_next_double(mut s: *mut mp_rand_state) -> ::core::ffi::c_double {
    return (mp_rand_next(s) >> 11 as ::core::ffi::c_int) as ::core::ffi::c_double
        * 1.1102230246251566e-16f64;
}
#[no_mangle]
pub unsafe extern "C" fn mp_rand_in_range32(
    mut s: *mut mp_rand_state,
    mut min: uint32_t,
    mut max: uint32_t,
) -> uint32_t {
    if min < max {
    } else {
        __assert_fail(
            b"min < max\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/random.c\0" as *const u8 as *const ::core::ffi::c_char,
            94 as ::core::ffi::c_uint,
            __ASSERT_FUNCTION.as_ptr(),
        );
    };
    let mut range: uint32_t = max.wrapping_sub(min);
    let mut mask: uint32_t = mp_round_next_power_of_2(range).wrapping_sub(1 as uint32_t);
    let mut ret: uint32_t = 0;
    loop {
        ret = (mp_rand_next(s) & mask as uint64_t) as uint32_t;
        if !(ret >= range) {
            break;
        }
    }
    return min.wrapping_add(ret);
}
