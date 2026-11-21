extern "C" {
    pub type mp_log;
    fn strcasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn mp_msg(log: *mut mp_log, lev: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ...);
    fn bstr_validate_utf8(s: bstr) -> ::core::ffi::c_int;
    fn bstr_sanitize_utf8_latin1(talloc_ctx: *mut ::core::ffi::c_void, s: bstr) -> bstr;
    fn bstr_case_startswith(s: bstr, prefix: bstr) -> bool;
}
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bstr {
    pub start: *mut ::core::ffi::c_uchar,
    pub len: size_t,
}
pub type C2RustUnnamed = ::core::ffi::c_uint;
pub const MSGL_MAX: C2RustUnnamed = 8;
pub const MSGL_STATS: C2RustUnnamed = 8;
pub const MSGL_TRACE: C2RustUnnamed = 7;
pub const MSGL_DEBUG: C2RustUnnamed = 6;
pub const MSGL_V: C2RustUnnamed = 5;
pub const MSGL_STATUS: C2RustUnnamed = 4;
pub const MSGL_INFO: C2RustUnnamed = 3;
pub const MSGL_WARN: C2RustUnnamed = 2;
pub const MSGL_ERR: C2RustUnnamed = 1;
pub const MSGL_FATAL: C2RustUnnamed = 0;
pub type C2RustUnnamed_0 = ::core::ffi::c_uint;
pub const MP_NO_LATIN1_FALLBACK: C2RustUnnamed_0 = 8;
pub const MP_STRICT_UTF8: C2RustUnnamed_0 = 4;
pub const MP_ICONV_ALLOW_CUTOFF: C2RustUnnamed_0 = 2;
pub const MP_ICONV_VERBOSE: C2RustUnnamed_0 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn bstr0(mut s: *const ::core::ffi::c_char) -> bstr {
    return bstr {
        start: s as *mut ::core::ffi::c_uchar,
        len: if !s.is_null() { strlen(s) } else { 0 as size_t },
    };
}
#[inline]
unsafe extern "C" fn bstr_startswith(mut str: bstr, mut prefix: bstr) -> bool {
    if str.len < prefix.len {
        return false_0 != 0;
    }
    return memcmp(
        str.start as *const ::core::ffi::c_void,
        prefix.start as *const ::core::ffi::c_void,
        prefix.len,
    ) == 0;
}
#[inline]
unsafe extern "C" fn bstr_startswith0(
    mut str: bstr,
    mut prefix: *const ::core::ffi::c_char,
) -> bool {
    return bstr_startswith(str, bstr0(prefix));
}
#[no_mangle]
pub unsafe extern "C" fn mp_charset_is_utf8(mut user_cp: *const ::core::ffi::c_char) -> bool {
    return !user_cp.is_null()
        && (strcasecmp(
            user_cp,
            b"utf8\0" as *const u8 as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
            || strcasecmp(
                user_cp,
                b"utf-8\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn mp_charset_is_utf16(mut user_cp: *const ::core::ffi::c_char) -> bool {
    let mut s: bstr = bstr0(user_cp) as bstr;
    return bstr_case_startswith(
        s as bstr,
        bstr0(b"utf16\0" as *const u8 as *const ::core::ffi::c_char),
    ) as ::core::ffi::c_int
        != 0
        || bstr_case_startswith(
            s as bstr,
            bstr0(b"utf-16\0" as *const u8 as *const ::core::ffi::c_char),
        ) as ::core::ffi::c_int
            != 0;
}
static mut utf_bom: [*const ::core::ffi::c_char; 3] = [
    b"\xEF\xBB\xBF\0" as *const u8 as *const ::core::ffi::c_char,
    b"\xFF\xFE\0" as *const u8 as *const ::core::ffi::c_char,
    b"\xFE\xFF\0" as *const u8 as *const ::core::ffi::c_char,
];
static mut utf_enc: [*const ::core::ffi::c_char; 3] = [
    b"utf-8\0" as *const u8 as *const ::core::ffi::c_char,
    b"utf-16le\0" as *const u8 as *const ::core::ffi::c_char,
    b"utf-16be\0" as *const u8 as *const ::core::ffi::c_char,
];
unsafe extern "C" fn ms_bom_guess(mut buf: bstr) -> *const ::core::ffi::c_char {
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n < 3 as ::core::ffi::c_int {
        if bstr_startswith0(buf as bstr, utf_bom[n as usize]) {
            return utf_enc[n as usize];
        }
        n += 1;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn mp_charset_guess(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut log: *mut mp_log,
    mut buf: bstr,
    mut user_cp: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    if *user_cp.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '+' as i32 {
        mp_msg(
            log,
            MSGL_V as ::core::ffi::c_int,
            b"Forcing charset '%s'.\n\0" as *const u8 as *const ::core::ffi::c_char,
            user_cp.offset(1 as ::core::ffi::c_int as isize),
        );
        return user_cp.offset(1 as ::core::ffi::c_int as isize);
    }
    let mut bom_cp: *const ::core::ffi::c_char = ms_bom_guess(buf);
    if !bom_cp.is_null() {
        mp_msg(
            log,
            MSGL_V as ::core::ffi::c_int,
            b"Data has a BOM, assuming %s as charset.\n\0" as *const u8
                as *const ::core::ffi::c_char,
            bom_cp,
        );
        return bom_cp;
    }
    let mut r: ::core::ffi::c_int = bstr_validate_utf8(buf as bstr);
    if r >= 0 as ::core::ffi::c_int
        || r > -(8 as ::core::ffi::c_int)
            && flags & MP_ICONV_ALLOW_CUTOFF as ::core::ffi::c_int != 0
    {
        if strcmp(
            user_cp,
            b"auto\0" as *const u8 as *const ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_int
            && !mp_charset_is_utf8(user_cp)
        {
            mp_msg(
                log,
                MSGL_V as ::core::ffi::c_int,
                b"Data looks like UTF-8, ignoring user-provided charset.\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return b"utf-8\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let mut res: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if strcasecmp(
        user_cp,
        b"auto\0" as *const u8 as *const ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        if res.is_null() {
            mp_msg(
                log,
                MSGL_V as ::core::ffi::c_int,
                b"Charset auto-detection failed.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
            res = b"UTF-8-BROKEN\0" as *const u8 as *const ::core::ffi::c_char;
        }
    } else {
        res = user_cp;
    }
    mp_msg(
        log,
        MSGL_V as ::core::ffi::c_int,
        b"Using charset '%s'.\n\0" as *const u8 as *const ::core::ffi::c_char,
        res,
    );
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn mp_iconv_to_utf8(
    mut log: *mut mp_log,
    mut buf: bstr,
    mut cp: *const ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bstr {
    if flags & MP_NO_LATIN1_FALLBACK as ::core::ffi::c_int != 0 {
        return buf;
    } else {
        return bstr_sanitize_utf8_latin1(NULL, buf as bstr) as bstr;
    };
}
