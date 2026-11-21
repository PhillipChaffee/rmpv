extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memrchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn strtod(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
    fn strtoll(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
    fn abort() -> !;
    fn vsnprintf(
        __s: *mut ::core::ffi::c_char,
        __maxlen: size_t,
        __format: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn vsscanf(
        __s: *const ::core::ffi::c_char,
        __format: *const ::core::ffi::c_char,
        __arg: ::core::ffi::VaList,
    ) -> ::core::ffi::c_int;
    fn ta_alloc_size(ta_parent: *mut ::core::ffi::c_void, size: size_t)
        -> *mut ::core::ffi::c_void;
    fn ta_get_size(ptr: *mut ::core::ffi::c_void) -> size_t;
    fn ta_free(ptr: *mut ::core::ffi::c_void);
    fn ta_calc_array_size(element_size: size_t, count: size_t) -> size_t;
    fn ta_xstrndup(
        ta_parent: *mut ::core::ffi::c_void,
        str: *const ::core::ffi::c_char,
        n: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn ta_xrealloc_size(
        ta_parent: *mut ::core::ffi::c_void,
        ptr: *mut ::core::ffi::c_void,
        size: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn ta_dbg_set_loc(
        ptr: *mut ::core::ffi::c_void,
        name: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    fn mp_log2(v: uint32_t) -> ::core::ffi::c_uint;
    fn mp_append_utf8_bstr(
        talloc_ctx: *mut ::core::ffi::c_void,
        buf: *mut bstr,
        codepoint: uint32_t,
    );
}
pub type __builtin_va_list = [__va_list_tag; 1];
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __va_list_tag {
    pub gp_offset: ::core::ffi::c_uint,
    pub fp_offset: ::core::ffi::c_uint,
    pub overflow_arg_area: *mut ::core::ffi::c_void,
    pub reg_save_area: *mut ::core::ffi::c_void,
}
pub type size_t = usize;
pub type va_list = __builtin_va_list;
pub type __uint32_t = u32;
pub type uint32_t = __uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bstr {
    pub start: *mut ::core::ffi::c_uchar,
    pub len: size_t,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 43] = unsafe {
    ::core::mem::transmute::<[u8; 43], [::core::ffi::c_char; 43]>(
        *b"void resize_append(void *, bstr *, size_t)\0",
    )
};
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn ta_oom_p(mut p: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    if p.is_null() {
        abort();
    }
    return p;
}
#[inline]
unsafe extern "C" fn mp_isspace(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int == ' ' as i32
        || c as ::core::ffi::c_int == '\u{c}' as i32
        || c as ::core::ffi::c_int == '\n' as i32
        || c as ::core::ffi::c_int == '\r' as i32
        || c as ::core::ffi::c_int == '\t' as i32
        || c as ::core::ffi::c_int == '\u{b}' as i32) as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn mp_isupper(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int >= 'A' as i32 && c as ::core::ffi::c_int <= 'Z' as i32)
        as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn mp_tolower(mut c: ::core::ffi::c_char) -> ::core::ffi::c_char {
    return (if mp_isupper(c) != 0 {
        c as ::core::ffi::c_int - 'A' as i32 + 'a' as i32
    } else {
        c as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn bstrdup0(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut str: bstr,
) -> *mut ::core::ffi::c_char {
    return ta_xstrndup(talloc_ctx, str.start as *mut ::core::ffi::c_char, str.len);
}
#[inline]
unsafe extern "C" fn bstr0(mut s: *const ::core::ffi::c_char) -> bstr {
    return bstr {
        start: s as *mut ::core::ffi::c_uchar,
        len: if !s.is_null() { strlen(s) } else { 0 as size_t },
    };
}
pub const BSTR_DECODE_OUT_OF_RANGE: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);
pub const BSTR_DECODE_TRUNCATED_SEQUENCE: ::core::ffi::c_int = -(2 as ::core::ffi::c_int);
pub const BSTR_DECODE_OVERLONG_ENCODING: ::core::ffi::c_int = -(3 as ::core::ffi::c_int);
#[inline]
unsafe extern "C" fn bstr_cut(mut str: bstr, mut n: ::core::ffi::c_int) -> bstr {
    if n < 0 as ::core::ffi::c_int {
        n = (n as size_t).wrapping_add(str.len) as ::core::ffi::c_int as ::core::ffi::c_int;
        if n < 0 as ::core::ffi::c_int {
            n = 0 as ::core::ffi::c_int;
        }
    }
    if n as size_t > str.len {
        n = str.len as ::core::ffi::c_int;
    }
    return bstr {
        start: str.start.offset(n as isize),
        len: str.len.wrapping_sub(n as size_t),
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
unsafe extern "C" fn bstr_endswith(mut str: bstr, mut suffix: bstr) -> bool {
    if str.len < suffix.len {
        return false_0 != 0;
    }
    return memcmp(
        str.start
            .offset(str.len as isize)
            .offset(-(suffix.len as isize)) as *const ::core::ffi::c_void,
        suffix.start as *const ::core::ffi::c_void,
        suffix.len,
    ) == 0;
}
#[inline]
unsafe extern "C" fn bstr_endswith0(mut str: bstr, mut suffix: *const ::core::ffi::c_char) -> bool {
    return bstr_endswith(str, bstr0(suffix));
}
#[inline]
unsafe extern "C" fn bstrcmp0(
    mut str1: bstr,
    mut str2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return bstrcmp(str1, bstr0(str2));
}
#[inline]
unsafe extern "C" fn bstrcasecmp0(
    mut str1: bstr,
    mut str2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return bstrcasecmp(str1, bstr0(str2));
}
#[no_mangle]
pub unsafe extern "C" fn bstrcmp(mut str1: bstr, mut str2: bstr) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if str1.len != 0 && str2.len != 0 {
        ret = memcmp(
            str1.start as *const ::core::ffi::c_void,
            str2.start as *const ::core::ffi::c_void,
            if str1.len > str2.len {
                str2.len
            } else {
                str1.len
            },
        );
    }
    if ret == 0 {
        if str1.len == str2.len {
            return 0 as ::core::ffi::c_int;
        } else if str1.len > str2.len {
            return 1 as ::core::ffi::c_int;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn bstrcasecmp(mut str1: bstr, mut str2: bstr) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if str1.len != 0 && str2.len != 0 {
        ret = strncasecmp(
            str1.start as *const ::core::ffi::c_char,
            str2.start as *const ::core::ffi::c_char,
            if str1.len > str2.len {
                str2.len
            } else {
                str1.len
            },
        );
    }
    if ret == 0 {
        if str1.len == str2.len {
            return 0 as ::core::ffi::c_int;
        } else if str1.len > str2.len {
            return 1 as ::core::ffi::c_int;
        } else {
            return -(1 as ::core::ffi::c_int);
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn bstrchr(mut str: bstr, mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if str.len == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    let mut pos: *mut ::core::ffi::c_uchar =
        memchr(str.start as *const ::core::ffi::c_void, c, str.len) as *mut ::core::ffi::c_uchar;
    return (if !pos.is_null() {
        pos.offset_from(str.start) as ::core::ffi::c_long
    } else {
        -(1 as ::core::ffi::c_int) as ::core::ffi::c_long
    }) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn bstrrchr(mut str: bstr, mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if str.len == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    let mut pos: *mut ::core::ffi::c_uchar =
        memrchr(str.start as *const ::core::ffi::c_void, c, str.len) as *mut ::core::ffi::c_uchar;
    return (if !pos.is_null() {
        pos.offset_from(str.start) as ::core::ffi::c_long
    } else {
        -(1 as ::core::ffi::c_int) as ::core::ffi::c_long
    }) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn bstrcspn(
    mut str: bstr,
    mut reject: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while (i as size_t) < str.len {
        if !strchr(reject, *str.start.offset(i as isize) as ::core::ffi::c_int).is_null() {
            break;
        }
        i += 1;
    }
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn bstrspn(
    mut str: bstr,
    mut accept: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while (i as size_t) < str.len {
        if strchr(accept, *str.start.offset(i as isize) as ::core::ffi::c_int).is_null() {
            break;
        }
        i += 1;
    }
    return i;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_find(mut haystack: bstr, mut needle: bstr) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i as size_t) < haystack.len {
        if bstr_startswith(
            bstr_splice(haystack, i, haystack.len as ::core::ffi::c_int),
            needle,
        ) {
            return i;
        }
        i += 1;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn bstr_lstrip(mut str: bstr) -> bstr {
    while str.len != 0 && mp_isspace(*str.start as ::core::ffi::c_char) != 0 {
        str.start = str.start.offset(1);
        str.len = str.len.wrapping_sub(1);
    }
    return str;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_strip(mut str: bstr) -> bstr {
    str = bstr_lstrip(str);
    while str.len != 0
        && mp_isspace(
            *str.start.offset(str.len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_char,
        ) != 0
    {
        str.len = str.len.wrapping_sub(1);
    }
    return str;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_split(
    mut str: bstr,
    mut sep: *const ::core::ffi::c_char,
    mut rest: *mut bstr,
) -> bstr {
    let mut start: ::core::ffi::c_int = 0;
    start = 0 as ::core::ffi::c_int;
    while (start as size_t) < str.len {
        if strchr(sep, *str.start.offset(start as isize) as ::core::ffi::c_int).is_null() {
            break;
        }
        start += 1;
    }
    str = bstr_cut(str, start);
    let mut end: ::core::ffi::c_int = bstrcspn(str, sep);
    if !rest.is_null() {
        *rest = bstr_cut(str, end);
    }
    return bstr_splice(str, 0 as ::core::ffi::c_int, end);
}
#[no_mangle]
pub unsafe extern "C" fn bstr_split_tok(
    mut str: bstr,
    mut tok: *const ::core::ffi::c_char,
    mut out_left: *mut bstr,
    mut out_right: *mut bstr,
) -> bool {
    let mut bsep: bstr = bstr0(tok) as bstr;
    let mut pos: ::core::ffi::c_int = bstr_find(str as bstr, bsep as bstr);
    if pos < 0 as ::core::ffi::c_int {
        pos = str.len as ::core::ffi::c_int;
    }
    *out_left = bstr_splice(str as bstr, 0 as ::core::ffi::c_int, pos) as bstr;
    *out_right = bstr_cut(
        str as bstr,
        (pos as size_t).wrapping_add(bsep.len) as ::core::ffi::c_int,
    ) as bstr;
    return pos as size_t != str.len;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_splice(
    mut str: bstr,
    mut start: ::core::ffi::c_int,
    mut end: ::core::ffi::c_int,
) -> bstr {
    if str.len == 0 {
        return str;
    }
    if start < 0 as ::core::ffi::c_int {
        start = (start as size_t).wrapping_add(str.len) as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    if end < 0 as ::core::ffi::c_int {
        end = (end as size_t).wrapping_add(str.len) as ::core::ffi::c_int as ::core::ffi::c_int;
    }
    end = (if end as size_t > str.len {
        str.len
    } else {
        end as size_t
    }) as ::core::ffi::c_int;
    start = if start > 0 as ::core::ffi::c_int {
        start
    } else {
        0 as ::core::ffi::c_int
    };
    end = if end > start { end } else { start };
    str.start = str.start.offset(start as isize);
    str.len = (end - start) as size_t;
    return str;
}
#[no_mangle]
pub unsafe extern "C" fn bstrtoll(
    mut str: bstr,
    mut rest: *mut bstr,
    mut base: ::core::ffi::c_int,
) -> ::core::ffi::c_longlong {
    str = bstr_lstrip(str);
    let mut buf: [::core::ffi::c_char; 51] = [0; 51];
    let mut len: ::core::ffi::c_int = (if str.len > 50 as size_t {
        50 as size_t
    } else {
        str.len
    }) as ::core::ffi::c_int;
    memcpy(
        &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        str.start as *const ::core::ffi::c_void,
        len as size_t,
    );
    buf[len as usize] = 0 as ::core::ffi::c_char;
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut r: ::core::ffi::c_longlong = strtoll(
        &raw mut buf as *mut ::core::ffi::c_char,
        &raw mut endptr,
        base,
    );
    if !rest.is_null() {
        *rest = bstr_cut(
            str,
            endptr.offset_from(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_long
                as ::core::ffi::c_int,
        );
    }
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn bstrtod(mut str: bstr, mut rest: *mut bstr) -> ::core::ffi::c_double {
    str = bstr_lstrip(str);
    let mut buf: [::core::ffi::c_char; 101] = [0; 101];
    let mut len: ::core::ffi::c_int = (if str.len > 100 as size_t {
        100 as size_t
    } else {
        str.len
    }) as ::core::ffi::c_int;
    memcpy(
        &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        str.start as *const ::core::ffi::c_void,
        len as size_t,
    );
    buf[len as usize] = 0 as ::core::ffi::c_char;
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut r: ::core::ffi::c_double =
        strtod(&raw mut buf as *mut ::core::ffi::c_char, &raw mut endptr);
    if !rest.is_null() {
        *rest = bstr_cut(
            str,
            endptr.offset_from(&raw mut buf as *mut ::core::ffi::c_char) as ::core::ffi::c_long
                as ::core::ffi::c_int,
        );
    }
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_splitchar(
    mut str: bstr,
    mut rest: *mut bstr,
    c: ::core::ffi::c_char,
) -> bstr {
    let mut pos: ::core::ffi::c_int = bstrchr(str, c as ::core::ffi::c_int);
    if pos < 0 as ::core::ffi::c_int {
        pos = str.len as ::core::ffi::c_int;
    }
    if !rest.is_null() {
        *rest = bstr_cut(str, pos + 1 as ::core::ffi::c_int);
    }
    return bstr_splice(str, 0 as ::core::ffi::c_int, pos + 1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn bstr_strip_linebreaks(mut str: bstr) -> bstr {
    if bstr_endswith0(str, b"\r\n\0" as *const u8 as *const ::core::ffi::c_char) {
        str = bstr_splice(
            str,
            0 as ::core::ffi::c_int,
            str.len.wrapping_sub(2 as size_t) as ::core::ffi::c_int,
        );
    } else if bstr_endswith0(str, b"\n\0" as *const u8 as *const ::core::ffi::c_char) {
        str = bstr_splice(
            str,
            0 as ::core::ffi::c_int,
            str.len.wrapping_sub(1 as size_t) as ::core::ffi::c_int,
        );
    }
    return str;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_eatstart(mut s: *mut bstr, mut prefix: bstr) -> bool {
    if !bstr_startswith(*s, prefix) {
        return false_0 != 0;
    }
    *s = bstr_cut(*s, prefix.len as ::core::ffi::c_int);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_eatend(mut s: *mut bstr, mut prefix: bstr) -> bool {
    if !bstr_endswith(*s, prefix) {
        return false_0 != 0;
    }
    (*s).len = (*s).len.wrapping_sub(prefix.len);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_lower(mut str: bstr) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i as size_t) < str.len {
        *str.start.offset(i as isize) =
            mp_tolower(*str.start.offset(i as isize) as ::core::ffi::c_char)
                as ::core::ffi::c_uchar;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn bstr_sscanf(
    mut str: bstr,
    mut format: *const ::core::ffi::c_char,
    mut args: ...
) -> ::core::ffi::c_int {
    let mut ptr: *mut ::core::ffi::c_char = bstrdup0(NULL, str);
    let mut va: ::core::ffi::VaListImpl;
    va = args.clone();
    let mut ret: ::core::ffi::c_int = vsscanf(ptr, format, va.as_va_list());
    ta_free(ptr as *mut ::core::ffi::c_void);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_parse_utf8_code_length(
    mut b: ::core::ffi::c_uchar,
) -> ::core::ffi::c_int {
    if (b as ::core::ffi::c_int) < 128 as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    let mut bytes: ::core::ffi::c_int = (7 as ::core::ffi::c_uint).wrapping_sub(mp_log2(
        (b as ::core::ffi::c_int ^ 255 as ::core::ffi::c_int) as uint32_t,
    )) as ::core::ffi::c_int;
    return if bytes >= 2 as ::core::ffi::c_int && bytes <= 4 as ::core::ffi::c_int {
        bytes
    } else {
        -(1 as ::core::ffi::c_int)
    };
}
#[no_mangle]
pub unsafe extern "C" fn bstr_decode_partial_utf8(mut s: *mut bstr) -> ::core::ffi::c_int {
    if (*s).len == 0 as size_t {
        return -(1 as ::core::ffi::c_int);
    }
    let mut codepoint: ::core::ffi::c_uint =
        *(*s).start.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint;
    (*s).start = (*s).start.offset(1);
    (*s).len = (*s).len.wrapping_sub(1);
    if codepoint >= 128 as ::core::ffi::c_uint {
        let mut bytes: ::core::ffi::c_int =
            bstr_parse_utf8_code_length(codepoint as ::core::ffi::c_uchar);
        if bytes < 1 as ::core::ffi::c_int {
            return BSTR_DECODE_OUT_OF_RANGE;
        }
        codepoint &= (127 as ::core::ffi::c_int >> bytes) as ::core::ffi::c_uint;
        let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while n < bytes {
            if (*s).len == 0 {
                return BSTR_DECODE_TRUNCATED_SEQUENCE;
            }
            let mut tmp: ::core::ffi::c_int =
                *(*s).start.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
            if tmp & 0xc0 as ::core::ffi::c_int != 0x80 as ::core::ffi::c_int {
                return BSTR_DECODE_TRUNCATED_SEQUENCE;
            }
            codepoint = codepoint << 6 as ::core::ffi::c_int
                | (tmp & !(0xc0 as ::core::ffi::c_int)) as ::core::ffi::c_uint;
            (*s).start = (*s).start.offset(1);
            (*s).len = (*s).len.wrapping_sub(1);
            n += 1;
        }
        if codepoint > 0x10ffff as ::core::ffi::c_int as ::core::ffi::c_uint
            || codepoint >= 0xd800 as ::core::ffi::c_uint
                && codepoint <= 0xdfff as ::core::ffi::c_uint
        {
            return BSTR_DECODE_OUT_OF_RANGE;
        }
        let mut min: ::core::ffi::c_uint = (if bytes == 2 as ::core::ffi::c_int {
            0x80 as ::core::ffi::c_int
        } else {
            (1 as ::core::ffi::c_int) << 5 as ::core::ffi::c_int * bytes - 4 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint;
        if codepoint < min {
            return BSTR_DECODE_OVERLONG_ENCODING;
        }
    }
    return codepoint as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_decode_utf8(
    mut s: bstr,
    mut out_next: *mut bstr,
) -> ::core::ffi::c_int {
    let mut res: ::core::ffi::c_int = bstr_decode_partial_utf8(&raw mut s);
    if res >= 0 as ::core::ffi::c_int && !out_next.is_null() {
        *out_next = s;
    }
    return res;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_split_utf8(mut str: bstr, mut out_next: *mut bstr) -> bstr {
    let mut rest: bstr = bstr {
        start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        len: 0,
    };
    let mut code: ::core::ffi::c_int = bstr_decode_utf8(str, &raw mut rest);
    if code < 0 as ::core::ffi::c_int {
        return bstr {
            start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
            len: 0,
        };
    }
    if !out_next.is_null() {
        *out_next = rest as bstr;
    }
    return bstr_splice(
        str,
        0 as ::core::ffi::c_int,
        str.len.wrapping_sub(rest.len) as ::core::ffi::c_int,
    );
}
#[no_mangle]
pub unsafe extern "C" fn bstr_validate_utf8(mut s: bstr) -> ::core::ffi::c_int {
    while s.len != 0 {
        if bstr_decode_utf8(s, &raw mut s) < 0 as ::core::ffi::c_int {
            let mut codepoint: ::core::ffi::c_uint =
                *s.start.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint;
            let mut bytes: ::core::ffi::c_int =
                bstr_parse_utf8_code_length(codepoint as ::core::ffi::c_uchar);
            if bytes > 1 as ::core::ffi::c_int && s.len < 6 as size_t {
                let mut n: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                while n < bytes {
                    if n as size_t >= s.len {
                        return (bytes as size_t).wrapping_sub(s.len).wrapping_neg()
                            as ::core::ffi::c_int;
                    }
                    let mut tmp: ::core::ffi::c_int =
                        *s.start.offset(n as isize) as ::core::ffi::c_int;
                    if tmp & 0xc0 as ::core::ffi::c_int != 0x80 as ::core::ffi::c_int {
                        break;
                    }
                    n += 1;
                }
            }
            return -(8 as ::core::ffi::c_int);
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_sanitize_utf8_latin1(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut s: bstr,
) -> bstr {
    let mut new: bstr = bstr {
        start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        len: 0,
    };
    let mut left: bstr = s as bstr;
    let mut first_ok: *mut ::core::ffi::c_uchar = s.start;
    while left.len != 0 {
        let mut r: ::core::ffi::c_int = bstr_decode_utf8(left as bstr, &raw mut left);
        if r < 0 as ::core::ffi::c_int {
            bstr_xappend(
                talloc_ctx,
                &raw mut new,
                bstr {
                    start: first_ok,
                    len: left.start.offset_from(first_ok) as ::core::ffi::c_long as size_t,
                },
            );
            mp_append_utf8_bstr(
                talloc_ctx,
                &raw mut new,
                *left.start.offset(0 as ::core::ffi::c_int as isize) as uint32_t,
            );
            left.start = left.start.offset(1 as ::core::ffi::c_int as isize);
            left.len = left.len.wrapping_sub(1 as size_t);
            first_ok = left.start;
        }
    }
    if new.start.is_null() {
        return s;
    }
    if first_ok != left.start {
        bstr_xappend(
            talloc_ctx,
            &raw mut new,
            bstr {
                start: first_ok,
                len: left.start.offset_from(first_ok) as ::core::ffi::c_long as size_t,
            },
        );
    }
    return new as bstr;
}
unsafe extern "C" fn resize_append(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut s: *mut bstr,
    mut append_min: size_t,
) {
    let mut size: size_t = ta_get_size((*s).start as *mut ::core::ffi::c_void);
    if (*s).len <= size {
    } else {
        __assert_fail(
            b"s->len <= size\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/bstr.c\0" as *const u8 as *const ::core::ffi::c_char,
            364 as ::core::ffi::c_uint,
            __ASSERT_FUNCTION.as_ptr(),
        );
    };
    if append_min > size.wrapping_sub((*s).len) {
        if append_min < size {
            append_min = size;
        }
        if size >= (SIZE_MAX as size_t).wrapping_div(2 as size_t)
            || append_min >= (SIZE_MAX as size_t).wrapping_div(2 as size_t)
        {
            abort();
        }
        (*s).start = ta_dbg_set_loc(
            ta_xrealloc_size(
                talloc_ctx,
                (*s).start as *mut ::core::ffi::c_void,
                size.wrapping_add(append_min),
            ),
            b"../.mpv/misc/bstr.c:370\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut ::core::ffi::c_uchar;
    }
}
#[no_mangle]
pub unsafe extern "C" fn bstr_xappend(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut s: *mut bstr,
    mut append: bstr,
) {
    if append.len == 0 {
        return;
    }
    resize_append(talloc_ctx, s, append.len.wrapping_add(1 as size_t));
    memmove(
        (*s).start.offset((*s).len as isize) as *mut ::core::ffi::c_void,
        append.start as *const ::core::ffi::c_void,
        append.len,
    );
    (*s).len = (*s).len.wrapping_add(append.len);
    *(*s).start.offset((*s).len as isize) = '\0' as i32 as ::core::ffi::c_uchar;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_xappend_asprintf(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut s: *mut bstr,
    mut fmt: *const ::core::ffi::c_char,
    mut args: ...
) -> ::core::ffi::c_int {
    let mut ap: ::core::ffi::VaListImpl;
    ap = args.clone();
    let mut ret: ::core::ffi::c_int = bstr_xappend_vasprintf(talloc_ctx, s, fmt, ap.as_va_list());
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_xappend_vasprintf(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut s: *mut bstr,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) -> ::core::ffi::c_int {
    let mut size: ::core::ffi::c_int = 0;
    let mut copy: ::core::ffi::VaListImpl;
    copy = ap.clone();
    let mut avail: size_t =
        ta_get_size((*s).start as *mut ::core::ffi::c_void).wrapping_sub((*s).len);
    let mut dest: *mut ::core::ffi::c_char = (if !(*s).start.is_null() {
        (*s).start.offset((*s).len as isize)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_uchar>()
    }) as *mut ::core::ffi::c_char;
    size = vsnprintf(dest, avail, fmt, copy.as_va_list());
    if size < 0 as ::core::ffi::c_int {
        return size;
    }
    if avail < 1 as size_t || (size + 1 as ::core::ffi::c_int) as size_t > avail {
        resize_append(talloc_ctx, s, (size + 1 as ::core::ffi::c_int) as size_t);
        vsnprintf(
            (*s).start.offset((*s).len as isize) as *mut ::core::ffi::c_char,
            (size + 1 as ::core::ffi::c_int) as size_t,
            fmt,
            ap.as_va_list(),
        );
    }
    (*s).len = (*s).len.wrapping_add(size as size_t);
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_case_startswith(mut s: bstr, mut prefix: bstr) -> bool {
    let mut start: bstr = bstr_splice(s, 0 as ::core::ffi::c_int, prefix.len as ::core::ffi::c_int);
    return start.len == prefix.len && bstrcasecmp(start, prefix) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_case_endswith(mut s: bstr, mut suffix: bstr) -> bool {
    let mut end: bstr = bstr_cut(s, suffix.len.wrapping_neg() as ::core::ffi::c_int);
    return end.len == suffix.len && bstrcasecmp(end, suffix) == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn h_to_i(mut c: ::core::ffi::c_uchar) -> ::core::ffi::c_int {
    if c as ::core::ffi::c_int >= '0' as i32 && c as ::core::ffi::c_int <= '9' as i32 {
        return c as ::core::ffi::c_int - '0' as i32;
    }
    if c as ::core::ffi::c_int >= 'a' as i32 && c as ::core::ffi::c_int <= 'f' as i32 {
        return c as ::core::ffi::c_int - 'a' as i32 + 10 as ::core::ffi::c_int;
    }
    if c as ::core::ffi::c_int >= 'A' as i32 && c as ::core::ffi::c_int <= 'F' as i32 {
        return c as ::core::ffi::c_int - 'A' as i32 + 10 as ::core::ffi::c_int;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn bstr_decode_hex(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut hex: bstr,
    mut out: *mut bstr,
) -> bool {
    if out.is_null() {
        return false_0 != 0;
    }
    let mut arr: *mut ::core::ffi::c_char = ta_oom_p(ta_dbg_set_loc(
        ta_alloc_size(
            talloc_ctx,
            ta_calc_array_size(
                ::core::mem::size_of::<::core::ffi::c_char>() as size_t,
                hex.len.wrapping_div(2 as size_t),
            ),
        ),
        b"../.mpv/misc/bstr.c:444\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char
        as *mut ::core::ffi::c_void)
        as *mut ::core::ffi::c_char;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while hex.len >= 2 as size_t {
        let mut a: ::core::ffi::c_int = h_to_i(*hex.start.offset(0 as ::core::ffi::c_int as isize));
        let mut b: ::core::ffi::c_int = h_to_i(*hex.start.offset(1 as ::core::ffi::c_int as isize));
        hex = bstr_splice(hex, 2 as ::core::ffi::c_int, hex.len as ::core::ffi::c_int);
        if a < 0 as ::core::ffi::c_int || b < 0 as ::core::ffi::c_int {
            ta_free(arr as *mut ::core::ffi::c_void);
            return false_0 != 0;
        }
        let fresh0 = len;
        len = len + 1;
        *arr.offset(fresh0 as isize) = (a << 4 as ::core::ffi::c_int | b) as ::core::ffi::c_char;
    }
    *out = bstr {
        start: arr as *mut ::core::ffi::c_uchar,
        len: len as size_t,
    };
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn bstr_find_in_list0(
    mut str: bstr,
    mut list: *mut *mut ::core::ffi::c_char,
    mut case_sensitive: bool,
) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while !list.is_null() && !(*list.offset(n as isize)).is_null() {
        if if case_sensitive as ::core::ffi::c_int != 0 {
            (bstrcmp0(str, *list.offset(n as isize)) == 0) as ::core::ffi::c_int
        } else {
            (bstrcasecmp0(str, *list.offset(n as isize)) == 0) as ::core::ffi::c_int
        } != 0
        {
            return n;
        }
        n += 1;
    }
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn bstr_in_list0(
    mut str: bstr,
    mut list: *mut *mut ::core::ffi::c_char,
) -> bool {
    return bstr_find_in_list0(str, list, false_0 != 0) != -(1 as ::core::ffi::c_int);
}
