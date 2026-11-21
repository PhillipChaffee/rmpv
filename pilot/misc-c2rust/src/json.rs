extern "C" {
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
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn ta_zalloc_size(
        ta_parent: *mut ::core::ffi::c_void,
        size: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn ta_get_size(ptr: *mut ::core::ffi::c_void) -> size_t;
    fn ta_calc_array_size(element_size: size_t, count: size_t) -> size_t;
    fn ta_calc_prealloc_elems(nextidx: size_t) -> size_t;
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
    fn mp_append_escaped_string(
        talloc_ctx: *mut ::core::ffi::c_void,
        dst: *mut bstr,
        src: *mut bstr,
    ) -> bool;
    fn bstr_xappend(talloc_ctx: *mut ::core::ffi::c_void, s: *mut bstr, append: bstr);
    fn bstr_xappend_asprintf(
        talloc_ctx: *mut ::core::ffi::c_void,
        s: *mut bstr,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}
pub type size_t = usize;
pub type __int64_t = i64;
pub type int64_t = __int64_t;
pub type mpv_format = ::core::ffi::c_uint;
pub const MPV_FORMAT_BYTE_ARRAY: mpv_format = 9;
pub const MPV_FORMAT_NODE_MAP: mpv_format = 8;
pub const MPV_FORMAT_NODE_ARRAY: mpv_format = 7;
pub const MPV_FORMAT_NODE: mpv_format = 6;
pub const MPV_FORMAT_DOUBLE: mpv_format = 5;
pub const MPV_FORMAT_INT64: mpv_format = 4;
pub const MPV_FORMAT_FLAG: mpv_format = 3;
pub const MPV_FORMAT_OSD_STRING: mpv_format = 2;
pub const MPV_FORMAT_STRING: mpv_format = 1;
pub const MPV_FORMAT_NONE: mpv_format = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpv_node {
    pub u: C2RustUnnamed,
    pub format: mpv_format,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub string: *mut ::core::ffi::c_char,
    pub flag: ::core::ffi::c_int,
    pub int64: int64_t,
    pub double_: ::core::ffi::c_double,
    pub list: *mut mpv_node_list,
    pub ba: *mut mpv_byte_array,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpv_byte_array {
    pub data: *mut ::core::ffi::c_void,
    pub size: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpv_node_list {
    pub num: ::core::ffi::c_int,
    pub values: *mut mpv_node,
    pub keys: *mut *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bstr {
    pub start: *mut ::core::ffi::c_uchar,
    pub len: size_t,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 45] = unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"void write_json_str(bstr *, unsigned char *)\0",
    )
};
#[inline]
unsafe extern "C" fn ta_oom_p(mut p: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    if p.is_null() {
        abort();
    }
    return p;
}
#[inline]
unsafe extern "C" fn bstr0(mut s: *const ::core::ffi::c_char) -> bstr {
    return bstr {
        start: s as *mut ::core::ffi::c_uchar,
        len: if !s.is_null() { strlen(s) } else { 0 as size_t },
    };
}
#[inline]
unsafe extern "C" fn mp_isupper(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int >= 'A' as i32 && c as ::core::ffi::c_int <= 'Z' as i32)
        as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn mp_islower(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int >= 'a' as i32 && c as ::core::ffi::c_int <= 'z' as i32)
        as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn mp_isdigit(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int >= '0' as i32 && c as ::core::ffi::c_int <= '9' as i32)
        as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn mp_isalpha(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (mp_isupper(c) != 0 || mp_islower(c) != 0) as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn mp_isalnum(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (mp_isalpha(c) != 0 || mp_isdigit(c) != 0) as ::core::ffi::c_int;
}
unsafe extern "C" fn eat_c(
    mut s: *mut *mut ::core::ffi::c_char,
    mut c: ::core::ffi::c_char,
) -> bool {
    if **s as ::core::ffi::c_int == c as ::core::ffi::c_int {
        *s = (*s).offset(1 as ::core::ffi::c_int as isize);
        return true_0 != 0;
    }
    return false_0 != 0;
}
unsafe extern "C" fn eat_ws(mut src: *mut *mut ::core::ffi::c_char) {
    loop {
        let mut c: ::core::ffi::c_char = **src;
        if c as ::core::ffi::c_int != ' ' as i32
            && c as ::core::ffi::c_int != '\t' as i32
            && c as ::core::ffi::c_int != '\n' as i32
            && c as ::core::ffi::c_int != '\r' as i32
        {
            return;
        }
        *src = (*src).offset(1 as ::core::ffi::c_int as isize);
    }
}
#[no_mangle]
pub unsafe extern "C" fn json_skip_whitespace(mut src: *mut *mut ::core::ffi::c_char) {
    eat_ws(src);
}
unsafe extern "C" fn read_id(
    mut ta_parent: *mut ::core::ffi::c_void,
    mut dst: *mut mpv_node,
    mut src: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut start: *mut ::core::ffi::c_char = *src;
    if mp_isalpha(**src) == 0 && **src as ::core::ffi::c_int != '_' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    while mp_isalnum(**src) != 0 || **src as ::core::ffi::c_int == '_' as i32 {
        *src = (*src).offset(1 as ::core::ffi::c_int as isize);
    }
    if **src as ::core::ffi::c_int == ' ' as i32 {
        **src = '\0' as i32 as ::core::ffi::c_char;
        *src = (*src).offset(1 as ::core::ffi::c_int as isize);
    } else {
        start = ta_xstrndup(
            ta_parent,
            start,
            (*src).offset_from(start) as ::core::ffi::c_long as size_t,
        );
    }
    (*dst).format = MPV_FORMAT_STRING;
    (*dst).u.string = start;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_str(
    mut ta_parent: *mut ::core::ffi::c_void,
    mut dst: *mut mpv_node,
    mut src: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if !eat_c(src, '"' as i32 as ::core::ffi::c_char) {
        return -(1 as ::core::ffi::c_int);
    }
    let mut str: *mut ::core::ffi::c_char = *src;
    let mut cur: *mut ::core::ffi::c_char = str;
    let mut has_escapes: bool = false_0 != 0;
    while *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
        && *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '"' as i32
    {
        if *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\\' as i32 {
            has_escapes = true_0 != 0;
            if *cur.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '"' as i32
                || *cur.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as i32
            {
                cur = cur.offset(1);
            }
        }
        cur = cur.offset(1);
    }
    if *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '"' as i32 {
        return -(1 as ::core::ffi::c_int);
    }
    *cur.offset(0 as ::core::ffi::c_int as isize) = '\0' as i32 as ::core::ffi::c_char;
    *src = cur.offset(1 as ::core::ffi::c_int as isize);
    if has_escapes {
        let mut unescaped: bstr = bstr {
            start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
            len: 0,
        };
        let mut r: bstr = bstr0(str) as bstr;
        if !mp_append_escaped_string(ta_parent, &raw mut unescaped, &raw mut r) {
            return -(1 as ::core::ffi::c_int);
        }
        str = unescaped.start as *mut ::core::ffi::c_char;
    }
    (*dst).format = MPV_FORMAT_STRING;
    (*dst).u.string = str;
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn read_sub(
    mut ta_parent: *mut ::core::ffi::c_void,
    mut dst: *mut mpv_node,
    mut src: *mut *mut ::core::ffi::c_char,
    mut max_depth: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut is_arr: bool = eat_c(src, '[' as i32 as ::core::ffi::c_char);
    let mut is_obj: bool =
        !is_arr && eat_c(src, '{' as i32 as ::core::ffi::c_char) as ::core::ffi::c_int != 0;
    if !is_arr && !is_obj {
        return -(1 as ::core::ffi::c_int);
    }
    let mut term: ::core::ffi::c_char = (if is_obj as ::core::ffi::c_int != 0 {
        '}' as i32
    } else {
        ']' as i32
    }) as ::core::ffi::c_char;
    let mut list: *mut mpv_node_list = ta_oom_p(ta_dbg_set_loc(
        ta_zalloc_size(ta_parent, ::core::mem::size_of::<mpv_node_list>() as size_t),
        b"../.mpv/misc/json.c:143\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut mpv_node_list
        as *mut ::core::ffi::c_void) as *mut mpv_node_list;
    loop {
        eat_ws(src);
        if eat_c(src, term) {
            break;
        }
        if (*list).num > 0 as ::core::ffi::c_int && !eat_c(src, ',' as i32 as ::core::ffi::c_char) {
            return -(1 as ::core::ffi::c_int);
        }
        eat_ws(src);
        if eat_c(src, term) {
            break;
        }
        if is_obj {
            let mut keynode: mpv_node = mpv_node {
                u: C2RustUnnamed {
                    string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                format: MPV_FORMAT_NONE,
            };
            if read_id(list as *mut ::core::ffi::c_void, &raw mut keynode, src)
                < 0 as ::core::ffi::c_int
                && read_str(list as *mut ::core::ffi::c_void, &raw mut keynode, src)
                    < 0 as ::core::ffi::c_int
            {
                return -(1 as ::core::ffi::c_int);
            }
            eat_ws(src);
            if !eat_c(src, ':' as i32 as ::core::ffi::c_char)
                && !eat_c(src, '=' as i32 as ::core::ffi::c_char)
            {
                return -(1 as ::core::ffi::c_int);
            }
            eat_ws(src);
            let mut nextidx_: size_t = (*list).num as size_t;
            if nextidx_
                >= ta_get_size((*list).keys as *mut ::core::ffi::c_void)
                    .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t)
            {
                (*list).keys = ta_dbg_set_loc(
                    ta_xrealloc_size(
                        list as *mut ::core::ffi::c_void,
                        (*list).keys as *mut ::core::ffi::c_void,
                        ta_calc_array_size(
                            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
                            ta_calc_prealloc_elems(nextidx_),
                        ),
                    ),
                    b"../.mpv/misc/json.c:165\0" as *const u8 as *const ::core::ffi::c_char,
                ) as *mut *mut ::core::ffi::c_char;
            }
            let ref mut fresh0 = *(*list).keys.offset((*list).num as isize);
            *fresh0 = keynode.u.string;
        }
        let mut nextidx__0: size_t = (*list).num as size_t;
        if nextidx__0
            >= ta_get_size((*list).values as *mut ::core::ffi::c_void)
                .wrapping_div(::core::mem::size_of::<mpv_node>() as size_t)
        {
            (*list).values = ta_dbg_set_loc(
                ta_xrealloc_size(
                    list as *mut ::core::ffi::c_void,
                    (*list).values as *mut ::core::ffi::c_void,
                    ta_calc_array_size(
                        ::core::mem::size_of::<mpv_node>() as size_t,
                        ta_calc_prealloc_elems(nextidx__0),
                    ),
                ),
                b"../.mpv/misc/json.c:168\0" as *const u8 as *const ::core::ffi::c_char,
            ) as *mut mpv_node;
        }
        if json_parse(
            ta_parent,
            (*list).values.offset((*list).num as isize) as *mut mpv_node,
            src,
            max_depth,
        ) < 0 as ::core::ffi::c_int
        {
            return -(1 as ::core::ffi::c_int);
        }
        (*list).num += 1;
    }
    (*dst).format = (if is_obj as ::core::ffi::c_int != 0 {
        MPV_FORMAT_NODE_MAP as ::core::ffi::c_int
    } else {
        MPV_FORMAT_NODE_ARRAY as ::core::ffi::c_int
    }) as mpv_format;
    (*dst).u.list = list as *mut mpv_node_list;
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn json_parse(
    mut ta_parent: *mut ::core::ffi::c_void,
    mut dst: *mut mpv_node,
    mut src: *mut *mut ::core::ffi::c_char,
    mut max_depth: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    max_depth -= 1 as ::core::ffi::c_int;
    if max_depth < 0 as ::core::ffi::c_int {
        return -(1 as ::core::ffi::c_int);
    }
    eat_ws(src);
    let mut c: ::core::ffi::c_char = **src;
    if c == 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if c as ::core::ffi::c_int == 'n' as i32
        && strncmp(
            *src,
            b"null\0" as *const u8 as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        *src = (*src).offset(4 as ::core::ffi::c_int as isize);
        (*dst).format = MPV_FORMAT_NONE;
        return 0 as ::core::ffi::c_int;
    } else if c as ::core::ffi::c_int == 't' as i32
        && strncmp(
            *src,
            b"true\0" as *const u8 as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        *src = (*src).offset(4 as ::core::ffi::c_int as isize);
        (*dst).format = MPV_FORMAT_FLAG;
        (*dst).u.flag = 1 as ::core::ffi::c_int;
        return 0 as ::core::ffi::c_int;
    } else if c as ::core::ffi::c_int == 'f' as i32
        && strncmp(
            *src,
            b"false\0" as *const u8 as *const ::core::ffi::c_char,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
    {
        *src = (*src).offset(5 as ::core::ffi::c_int as isize);
        (*dst).format = MPV_FORMAT_FLAG;
        (*dst).u.flag = 0 as ::core::ffi::c_int;
        return 0 as ::core::ffi::c_int;
    } else if c as ::core::ffi::c_int == '"' as i32 {
        return read_str(ta_parent, dst, src);
    } else if c as ::core::ffi::c_int == '[' as i32 || c as ::core::ffi::c_int == '{' as i32 {
        return read_sub(ta_parent, dst, src, max_depth);
    } else if c as ::core::ffi::c_int == '-' as i32
        || c as ::core::ffi::c_int >= '0' as i32 && c as ::core::ffi::c_int <= '9' as i32
    {
        let mut nsrci: *mut ::core::ffi::c_char = *src;
        let mut nsrcf: *mut ::core::ffi::c_char = *src;
        *__errno_location() = 0 as ::core::ffi::c_int;
        let mut numi: ::core::ffi::c_longlong =
            strtoll(*src, &raw mut nsrci, 0 as ::core::ffi::c_int);
        if *__errno_location() != 0 {
            nsrci = *src;
        }
        *__errno_location() = 0 as ::core::ffi::c_int;
        let mut numf: ::core::ffi::c_double = strtod(*src, &raw mut nsrcf);
        if *__errno_location() != 0 {
            nsrcf = *src;
        }
        if nsrci >= nsrcf {
            *src = nsrci;
            (*dst).format = MPV_FORMAT_INT64;
            (*dst).u.int64 = numi as int64_t;
            return 0 as ::core::ffi::c_int;
        }
        if nsrcf > *src && numf.is_finite() as i32 != 0 {
            *src = nsrcf;
            (*dst).format = MPV_FORMAT_DOUBLE;
            (*dst).u.double_ = numf;
            return 0 as ::core::ffi::c_int;
        }
        return -(1 as ::core::ffi::c_int);
    }
    return -(1 as ::core::ffi::c_int);
}
static mut special_escape: [::core::ffi::c_char; 14] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    'b' as i32 as ::core::ffi::c_char,
    't' as i32 as ::core::ffi::c_char,
    'n' as i32 as ::core::ffi::c_char,
    0,
    'f' as i32 as ::core::ffi::c_char,
    'r' as i32 as ::core::ffi::c_char,
];
unsafe extern "C" fn write_json_str(mut b: *mut bstr, mut str: *mut ::core::ffi::c_uchar) {
    if !str.is_null() {
    } else {
        __assert_fail(
            b"str\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/json.c\0" as *const u8 as *const ::core::ffi::c_char,
            260 as ::core::ffi::c_uint,
            __ASSERT_FUNCTION.as_ptr(),
        );
    };
    bstr_xappend(
        NULL,
        b,
        bstr0(b"\"\0" as *const u8 as *const ::core::ffi::c_char) as bstr,
    );
    loop {
        let mut cur: *mut ::core::ffi::c_uchar = str;
        while *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            >= 32 as ::core::ffi::c_int
            && *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '"' as i32
            && *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != '\\' as i32
        {
            cur = cur.offset(1);
        }
        if *cur.offset(0 as ::core::ffi::c_int as isize) == 0 {
            break;
        }
        bstr_xappend(
            NULL,
            b,
            bstr {
                start: str,
                len: cur.offset_from(str) as ::core::ffi::c_long as size_t,
            },
        );
        if *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '"' as i32 {
            bstr_xappend(
                NULL,
                b,
                bstr {
                    start: b"\\\"\0" as *const u8 as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_uchar,
                    len: 2 as size_t,
                },
            );
        } else if *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == '\\' as i32
        {
            bstr_xappend(
                NULL,
                b,
                bstr {
                    start: b"\\\\\0" as *const u8 as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_uchar,
                    len: 2 as size_t,
                },
            );
        } else if (*cur.offset(0 as ::core::ffi::c_int as isize) as usize)
            < ::core::mem::size_of::<[::core::ffi::c_char; 14]>() as usize
            && special_escape[*cur.offset(0 as ::core::ffi::c_int as isize) as usize]
                as ::core::ffi::c_int
                != 0
        {
            bstr_xappend_asprintf(
                NULL,
                b,
                b"\\%c\0" as *const u8 as *const ::core::ffi::c_char,
                special_escape[*cur.offset(0 as ::core::ffi::c_int as isize) as usize]
                    as ::core::ffi::c_int,
            );
        } else {
            bstr_xappend_asprintf(
                NULL,
                b,
                b"\\u%04x\0" as *const u8 as *const ::core::ffi::c_char,
                *cur.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
            );
        }
        str = cur.offset(1 as ::core::ffi::c_int as isize);
    }
    bstr_xappend(NULL, b, bstr0(str as *const ::core::ffi::c_char) as bstr);
    bstr_xappend(
        NULL,
        b,
        bstr0(b"\"\0" as *const u8 as *const ::core::ffi::c_char) as bstr,
    );
}
unsafe extern "C" fn add_indent(mut b: *mut bstr, mut indent: ::core::ffi::c_int) {
    if indent < 0 as ::core::ffi::c_int {
        return;
    }
    bstr_xappend(
        NULL,
        b,
        bstr0(b"\n\0" as *const u8 as *const ::core::ffi::c_char) as bstr,
    );
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n < indent {
        bstr_xappend(
            NULL,
            b,
            bstr0(b" \0" as *const u8 as *const ::core::ffi::c_char) as bstr,
        );
        n += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn json_append(
    mut b: *mut bstr,
    mut src: *const mpv_node,
    mut indent: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    match (*src).format as ::core::ffi::c_uint {
        0 => {
            bstr_xappend(
                NULL,
                b,
                bstr0(b"null\0" as *const u8 as *const ::core::ffi::c_char) as bstr,
            );
            return 0 as ::core::ffi::c_int;
        }
        3 => {
            bstr_xappend(
                NULL,
                b,
                bstr0(if (*src).u.flag != 0 {
                    b"true\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"false\0" as *const u8 as *const ::core::ffi::c_char
                }) as bstr,
            );
            return 0 as ::core::ffi::c_int;
        }
        4 => {
            bstr_xappend_asprintf(
                NULL,
                b,
                b"%ld\0" as *const u8 as *const ::core::ffi::c_char,
                (*src).u.int64,
            );
            return 0 as ::core::ffi::c_int;
        }
        5 => {
            let mut px: *const ::core::ffi::c_char =
                if (*src).u.double_.is_finite() as i32 != 0 || indent == 0 as ::core::ffi::c_int {
                    b"\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"\"\0" as *const u8 as *const ::core::ffi::c_char
                };
            bstr_xappend_asprintf(
                NULL,
                b,
                b"%s%f%s\0" as *const u8 as *const ::core::ffi::c_char,
                px,
                (*src).u.double_,
                px,
            );
            return 0 as ::core::ffi::c_int;
        }
        1 => {
            if indent == 0 as ::core::ffi::c_int {
                bstr_xappend(NULL, b, bstr0((*src).u.string) as bstr);
            } else {
                write_json_str(b, (*src).u.string as *mut ::core::ffi::c_uchar);
            }
            return 0 as ::core::ffi::c_int;
        }
        7 | 8 => {
            let mut list: *mut mpv_node_list = (*src).u.list as *mut mpv_node_list;
            let mut is_obj: bool = (*src).format as ::core::ffi::c_uint
                == MPV_FORMAT_NODE_MAP as ::core::ffi::c_int as ::core::ffi::c_uint;
            bstr_xappend(
                NULL,
                b,
                bstr0(if is_obj as ::core::ffi::c_int != 0 {
                    b"{\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"[\0" as *const u8 as *const ::core::ffi::c_char
                }) as bstr,
            );
            let mut next_indent: ::core::ffi::c_int = if indent >= 0 as ::core::ffi::c_int {
                indent + 1 as ::core::ffi::c_int
            } else {
                -(1 as ::core::ffi::c_int)
            };
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while n < (*list).num {
                if n != 0 {
                    bstr_xappend(
                        NULL,
                        b,
                        bstr0(b",\0" as *const u8 as *const ::core::ffi::c_char) as bstr,
                    );
                }
                add_indent(b, next_indent);
                if is_obj {
                    write_json_str(
                        b,
                        *(*list).keys.offset(n as isize) as *mut ::core::ffi::c_uchar,
                    );
                    bstr_xappend(
                        NULL,
                        b,
                        bstr0(b":\0" as *const u8 as *const ::core::ffi::c_char) as bstr,
                    );
                }
                json_append(
                    b,
                    (*list).values.offset(n as isize) as *mut mpv_node,
                    next_indent,
                );
                n += 1;
            }
            add_indent(b, indent);
            bstr_xappend(
                NULL,
                b,
                bstr0(if is_obj as ::core::ffi::c_int != 0 {
                    b"}\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"]\0" as *const u8 as *const ::core::ffi::c_char
                }) as bstr,
            );
            return 0 as ::core::ffi::c_int;
        }
        _ => {}
    }
    return -(1 as ::core::ffi::c_int);
}
unsafe extern "C" fn json_append_str(
    mut dst: *mut *mut ::core::ffi::c_char,
    mut src: *mut mpv_node,
    mut indent: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buffer: bstr = bstr0(*dst) as bstr;
    let mut r: ::core::ffi::c_int = json_append(&raw mut buffer, src, indent);
    *dst = buffer.start as *mut ::core::ffi::c_char;
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn json_write(
    mut dst: *mut *mut ::core::ffi::c_char,
    mut src: *mut mpv_node,
) -> ::core::ffi::c_int {
    return json_append_str(dst, src, -(1 as ::core::ffi::c_int));
}
#[no_mangle]
pub unsafe extern "C" fn json_write_pretty(
    mut dst: *mut *mut ::core::ffi::c_char,
    mut src: *mut mpv_node,
) -> ::core::ffi::c_int {
    return json_append_str(dst, src, 0 as ::core::ffi::c_int);
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
