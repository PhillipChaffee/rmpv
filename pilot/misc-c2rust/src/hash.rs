extern "C" {
    pub type AVHashContext;
    fn av_hash_alloc(
        ctx: *mut *mut AVHashContext,
        name: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn av_hash_get_size(ctx: *const AVHashContext) -> ::core::ffi::c_int;
    fn av_hash_init(ctx: *mut AVHashContext);
    fn av_hash_update(ctx: *mut AVHashContext, src: *const uint8_t, len: size_t);
    fn av_hash_final(ctx: *mut AVHashContext, dst: *mut uint8_t);
    fn av_hash_freep(ctx: *mut *mut AVHashContext);
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn bstr_xappend_asprintf(
        talloc_ctx: *mut ::core::ffi::c_void,
        s: *mut bstr,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}
pub type __uint8_t = u8;
pub type uint8_t = __uint8_t;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bstr {
    pub start: *mut ::core::ffi::c_uchar,
    pub len: size_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 68] = unsafe {
    ::core::mem::transmute::<[u8; 68], [::core::ffi::c_char; 68]>(
        *b"bstr mp_hash_to_bstr(void *, const uint8_t *, size_t, const char *)\0",
    )
};
#[no_mangle]
pub unsafe extern "C" fn mp_hash_to_bstr(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut data: *const uint8_t,
    mut len: size_t,
    mut algorithm: *const ::core::ffi::c_char,
) -> bstr {
    let mut ctx: *mut AVHashContext = ::core::ptr::null_mut::<AVHashContext>();
    if av_hash_alloc(&raw mut ctx, algorithm) == 0 as ::core::ffi::c_int {
    } else {
        __assert_fail(
            b"av_hash_alloc(&ctx, algorithm) == 0\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/hash.c\0" as *const u8 as *const ::core::ffi::c_char,
            28 as ::core::ffi::c_uint,
            __ASSERT_FUNCTION.as_ptr(),
        );
    };
    let mut size: ::core::ffi::c_int = av_hash_get_size(ctx);
    let mut hash: [uint8_t; 64] = [
        0 as ::core::ffi::c_int as uint8_t,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    av_hash_init(ctx);
    av_hash_update(ctx, data, len);
    av_hash_final(ctx, &raw mut hash as *mut uint8_t);
    av_hash_freep(&raw mut ctx);
    let mut ret: bstr = bstr {
        start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        len: 0,
    };
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n < size {
        bstr_xappend_asprintf(
            talloc_ctx,
            &raw mut ret,
            b"%02X\0" as *const u8 as *const ::core::ffi::c_char,
            hash[n as usize] as ::core::ffi::c_int,
        );
        n += 1;
    }
    return ret;
}
