extern "C" {
    fn abort() -> !;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
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
    fn ta_zalloc_size(
        ta_parent: *mut ::core::ffi::c_void,
        size: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn ta_get_size(ptr: *mut ::core::ffi::c_void) -> size_t;
    fn ta_calc_array_size(element_size: size_t, count: size_t) -> size_t;
    fn ta_calc_prealloc_elems(nextidx: size_t) -> size_t;
    fn ta_xstrdup(
        ta_parent: *mut ::core::ffi::c_void,
        str: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
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
    fn bstrcmp(str1: bstr, str2: bstr) -> ::core::ffi::c_int;
}
pub type __int64_t = i64;
pub type int64_t = __int64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bstr {
    pub start: *mut ::core::ffi::c_uchar,
    pub len: size_t,
}
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpv_node {
    pub u: C2RustUnnamed,
    pub format: mpv_format,
}
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
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe extern "C" fn ta_oom_p(mut p: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    if p.is_null() {
        abort();
    }
    return p;
}
#[inline]
unsafe extern "C" fn bstrdup0(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut str: bstr,
) -> *mut ::core::ffi::c_char {
    return ta_xstrndup(talloc_ctx, str.start as *mut ::core::ffi::c_char, str.len);
}
#[inline]
unsafe extern "C" fn bstrto0(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut str: bstr,
) -> *mut ::core::ffi::c_char {
    return if !str.start.is_null() {
        bstrdup0(talloc_ctx, str)
    } else {
        ta_xstrdup(talloc_ctx, b"\0" as *const u8 as *const ::core::ffi::c_char)
    };
}
#[inline]
unsafe extern "C" fn bstr0(mut s: *const ::core::ffi::c_char) -> bstr {
    return bstr {
        start: s as *mut ::core::ffi::c_uchar,
        len: if !s.is_null() { strlen(s) } else { 0 as size_t },
    };
}
#[inline]
unsafe extern "C" fn bstr_equals(mut str1: bstr, mut str2: bstr) -> bool {
    if str1.len != str2.len {
        return false_0 != 0;
    }
    return str1.start == str2.start || bstrcmp(str1, str2) == 0 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn bstr_equals0(mut str1: bstr, mut str2: *const ::core::ffi::c_char) -> bool {
    return bstr_equals(str1, bstr0(str2));
}
#[no_mangle]
pub unsafe extern "C" fn node_init(
    mut dst: *mut mpv_node,
    mut format: ::core::ffi::c_int,
    mut parent: *mut mpv_node,
) {
    if format == MPV_FORMAT_NODE_MAP as ::core::ffi::c_int
        || format == MPV_FORMAT_NODE_ARRAY as ::core::ffi::c_int
        || format == MPV_FORMAT_FLAG as ::core::ffi::c_int
        || format == MPV_FORMAT_INT64 as ::core::ffi::c_int
        || format == MPV_FORMAT_DOUBLE as ::core::ffi::c_int
        || format == MPV_FORMAT_BYTE_ARRAY as ::core::ffi::c_int
        || format == MPV_FORMAT_NONE as ::core::ffi::c_int
    {
    } else {
        __assert_fail(
            b"format == MPV_FORMAT_NODE_MAP || format == MPV_FORMAT_NODE_ARRAY || format == MPV_FORMAT_FLAG || format == MPV_FORMAT_INT64 || format == MPV_FORMAT_DOUBLE || format == MPV_FORMAT_BYTE_ARRAY || format == MPV_FORMAT_NONE\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
            18 as ::core::ffi::c_uint,
            b"void node_init(struct mpv_node *, int, struct mpv_node *)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    let mut ta_parent: *mut ::core::ffi::c_void = NULL;
    if !parent.is_null() {
        if (*parent).format as ::core::ffi::c_uint
            == MPV_FORMAT_NODE_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*parent).format as ::core::ffi::c_uint
                == MPV_FORMAT_NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
        } else {
            __assert_fail(
                b"parent->format == MPV_FORMAT_NODE_MAP || parent->format == MPV_FORMAT_NODE_ARRAY\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
                23 as ::core::ffi::c_uint,
                b"void node_init(struct mpv_node *, int, struct mpv_node *)\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        };
        ta_parent = (*parent).u.list as *mut ::core::ffi::c_void;
    }
    *dst = mpv_node {
        u: C2RustUnnamed {
            string: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        },
        format: format as mpv_format,
    };
    if format == MPV_FORMAT_NODE_MAP as ::core::ffi::c_int
        || format == MPV_FORMAT_NODE_ARRAY as ::core::ffi::c_int
    {
        (*dst).u.list = ta_oom_p(ta_dbg_set_loc(
            ta_zalloc_size(ta_parent, ::core::mem::size_of::<mpv_node_list>() as size_t),
            b"../.mpv/misc/node.c:29\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut mpv_node_list as *mut ::core::ffi::c_void)
            as *mut mpv_node_list as *mut mpv_node_list;
    }
    if format == MPV_FORMAT_BYTE_ARRAY as ::core::ffi::c_int {
        (*dst).u.ba = ta_oom_p(ta_dbg_set_loc(
            ta_zalloc_size(
                ta_parent,
                ::core::mem::size_of::<mpv_byte_array>() as size_t,
            ),
            b"../.mpv/misc/node.c:31\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut mpv_byte_array as *mut ::core::ffi::c_void)
            as *mut mpv_byte_array as *mut mpv_byte_array;
    }
}
#[no_mangle]
pub unsafe extern "C" fn node_array_add(
    mut dst: *mut mpv_node,
    mut format: ::core::ffi::c_int,
) -> *mut mpv_node {
    let mut list: *mut mpv_node_list = (*dst).u.list as *mut mpv_node_list;
    if (*dst).format as ::core::ffi::c_uint
        == MPV_FORMAT_NODE_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*dst).u.list.is_null()
    {
    } else {
        __assert_fail(
            b"dst->format == MPV_FORMAT_NODE_ARRAY && dst->u.list\0" as *const u8
                as *const ::core::ffi::c_char,
            b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
            39 as ::core::ffi::c_uint,
            b"struct mpv_node *node_array_add(struct mpv_node *, int)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    let mut nextidx_: size_t = (*list).num as size_t;
    if nextidx_
        >= ta_get_size((*list).values as *mut ::core::ffi::c_void)
            .wrapping_div(::core::mem::size_of::<mpv_node>() as size_t)
    {
        (*list).values = ta_dbg_set_loc(
            ta_xrealloc_size(
                list as *mut ::core::ffi::c_void,
                (*list).values as *mut ::core::ffi::c_void,
                ta_calc_array_size(
                    ::core::mem::size_of::<mpv_node>() as size_t,
                    ta_calc_prealloc_elems(nextidx_),
                ),
            ),
            b"../.mpv/misc/node.c:40\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut mpv_node;
    }
    node_init(
        (*list).values.offset((*list).num as isize) as *mut mpv_node,
        format,
        dst,
    );
    let fresh0 = (*list).num;
    (*list).num = (*list).num + 1;
    return (*list).values.offset(fresh0 as isize) as *mut mpv_node;
}
#[no_mangle]
pub unsafe extern "C" fn node_map_add(
    mut dst: *mut mpv_node,
    mut key: *const ::core::ffi::c_char,
    mut format: ::core::ffi::c_int,
) -> *mut mpv_node {
    if !key.is_null() {
    } else {
        __assert_fail(
            b"key\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
            50 as ::core::ffi::c_uint,
            b"struct mpv_node *node_map_add(struct mpv_node *, const char *, int)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    return node_map_badd(dst, bstr0(key), format);
}
#[no_mangle]
pub unsafe extern "C" fn node_map_badd(
    mut dst: *mut mpv_node,
    mut key: bstr,
    mut format: ::core::ffi::c_int,
) -> *mut mpv_node {
    if !key.start.is_null() {
    } else {
        __assert_fail(
            b"key.start\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
            56 as ::core::ffi::c_uint,
            b"struct mpv_node *node_map_badd(struct mpv_node *, struct bstr, int)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    let mut list: *mut mpv_node_list = (*dst).u.list as *mut mpv_node_list;
    if (*dst).format as ::core::ffi::c_uint
        == MPV_FORMAT_NODE_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*dst).u.list.is_null()
    {
    } else {
        __assert_fail(
            b"dst->format == MPV_FORMAT_NODE_MAP && dst->u.list\0" as *const u8
                as *const ::core::ffi::c_char,
            b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
            59 as ::core::ffi::c_uint,
            b"struct mpv_node *node_map_badd(struct mpv_node *, struct bstr, int)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    let mut nextidx_: size_t = (*list).num as size_t;
    if nextidx_
        >= ta_get_size((*list).values as *mut ::core::ffi::c_void)
            .wrapping_div(::core::mem::size_of::<mpv_node>() as size_t)
    {
        (*list).values = ta_dbg_set_loc(
            ta_xrealloc_size(
                list as *mut ::core::ffi::c_void,
                (*list).values as *mut ::core::ffi::c_void,
                ta_calc_array_size(
                    ::core::mem::size_of::<mpv_node>() as size_t,
                    ta_calc_prealloc_elems(nextidx_),
                ),
            ),
            b"../.mpv/misc/node.c:60\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut mpv_node;
    }
    let mut nextidx__0: size_t = (*list).num as size_t;
    if nextidx__0
        >= ta_get_size((*list).keys as *mut ::core::ffi::c_void)
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t)
    {
        (*list).keys = ta_dbg_set_loc(
            ta_xrealloc_size(
                list as *mut ::core::ffi::c_void,
                (*list).keys as *mut ::core::ffi::c_void,
                ta_calc_array_size(
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>() as size_t,
                    ta_calc_prealloc_elems(nextidx__0),
                ),
            ),
            b"../.mpv/misc/node.c:61\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut *mut ::core::ffi::c_char;
    }
    let ref mut fresh1 = *(*list).keys.offset((*list).num as isize);
    *fresh1 = bstrdup0(list as *mut ::core::ffi::c_void, key);
    node_init(
        (*list).values.offset((*list).num as isize) as *mut mpv_node,
        format,
        dst,
    );
    let fresh2 = (*list).num;
    (*list).num = (*list).num + 1;
    return (*list).values.offset(fresh2 as isize) as *mut mpv_node;
}
#[no_mangle]
pub unsafe extern "C" fn node_map_add_string(
    mut dst: *mut mpv_node,
    mut key: *const ::core::ffi::c_char,
    mut val: *const ::core::ffi::c_char,
) {
    if !val.is_null() {
    } else {
        __assert_fail(
            b"val\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
            72 as ::core::ffi::c_uint,
            b"void node_map_add_string(struct mpv_node *, const char *, const char *)\0"
                as *const u8 as *const ::core::ffi::c_char,
        );
    };
    let mut entry: *mut mpv_node = node_map_add(dst, key, MPV_FORMAT_NONE as ::core::ffi::c_int);
    (*entry).format = MPV_FORMAT_STRING;
    (*entry).u.string = ta_xstrdup((*dst).u.list as *mut ::core::ffi::c_void, val);
}
#[no_mangle]
pub unsafe extern "C" fn node_map_add_bstr(
    mut dst: *mut mpv_node,
    mut key: *const ::core::ffi::c_char,
    mut val: bstr,
) {
    if !val.start.is_null() {
    } else {
        __assert_fail(
            b"val.start\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
            81 as ::core::ffi::c_uint,
            b"void node_map_add_bstr(struct mpv_node *, const char *, bstr)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    let mut entry: *mut mpv_node = node_map_add(dst, key, MPV_FORMAT_NONE as ::core::ffi::c_int);
    (*entry).format = MPV_FORMAT_STRING;
    (*entry).u.string = bstrto0((*dst).u.list as *mut ::core::ffi::c_void, val as bstr);
}
#[no_mangle]
pub unsafe extern "C" fn node_map_add_int64(
    mut dst: *mut mpv_node,
    mut key: *const ::core::ffi::c_char,
    mut v: int64_t,
) {
    (*node_map_add(dst, key, MPV_FORMAT_INT64 as ::core::ffi::c_int))
        .u
        .int64 = v;
}
#[no_mangle]
pub unsafe extern "C" fn node_map_add_double(
    mut dst: *mut mpv_node,
    mut key: *const ::core::ffi::c_char,
    mut v: ::core::ffi::c_double,
) {
    (*node_map_add(dst, key, MPV_FORMAT_DOUBLE as ::core::ffi::c_int))
        .u
        .double_ = v;
}
#[no_mangle]
pub unsafe extern "C" fn node_map_add_flag(
    mut dst: *mut mpv_node,
    mut key: *const ::core::ffi::c_char,
    mut v: bool,
) {
    (*node_map_add(dst, key, MPV_FORMAT_FLAG as ::core::ffi::c_int))
        .u
        .flag = v as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn node_map_get(
    mut src: *mut mpv_node,
    mut key: *const ::core::ffi::c_char,
) -> *mut mpv_node {
    return node_map_bget(src, bstr0(key));
}
#[no_mangle]
pub unsafe extern "C" fn node_map_bget(mut src: *mut mpv_node, mut key: bstr) -> *mut mpv_node {
    if (*src).format as ::core::ffi::c_uint
        != MPV_FORMAT_NODE_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return ::core::ptr::null_mut::<mpv_node>();
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*(*src).u.list).num {
        if bstr_equals0(key, *(*(*src).u.list).keys.offset(i as isize)) {
            return (*(*src).u.list).values.offset(i as isize) as *mut mpv_node;
        }
        i += 1;
    }
    return ::core::ptr::null_mut::<mpv_node>();
}
#[no_mangle]
pub unsafe extern "C" fn equal_mpv_value(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
    mut format: ::core::ffi::c_int,
) -> bool {
    match format {
        0 => return true_0 != 0,
        1 | 2 => {
            return strcmp(
                *(a as *mut *mut ::core::ffi::c_char),
                *(b as *mut *mut ::core::ffi::c_char),
            ) == 0 as ::core::ffi::c_int;
        }
        3 => return *(a as *mut ::core::ffi::c_int) == *(b as *mut ::core::ffi::c_int),
        4 => return *(a as *mut int64_t) == *(b as *mut int64_t),
        5 => {
            return *(a as *mut ::core::ffi::c_double) == *(b as *mut ::core::ffi::c_double);
        }
        6 => return equal_mpv_node(a as *const mpv_node, b as *const mpv_node),
        9 => {
            let mut a_r: *const mpv_byte_array = a as *const mpv_byte_array;
            let mut b_r: *const mpv_byte_array = b as *const mpv_byte_array;
            if (*a_r).size != (*b_r).size {
                return false_0 != 0;
            }
            return memcmp((*a_r).data, (*b_r).data, (*a_r).size) == 0 as ::core::ffi::c_int;
        }
        7 | 8 => {
            let mut l_a: *mut mpv_node_list = *(a as *mut *mut mpv_node_list);
            let mut l_b: *mut mpv_node_list = *(b as *mut *mut mpv_node_list);
            if (*l_a).num != (*l_b).num {
                return false_0 != 0;
            }
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while n < (*l_a).num {
                if format == MPV_FORMAT_NODE_MAP as ::core::ffi::c_int {
                    if strcmp(
                        *(*l_a).keys.offset(n as isize),
                        *(*l_b).keys.offset(n as isize),
                    ) != 0 as ::core::ffi::c_int
                    {
                        return false_0 != 0;
                    }
                }
                if !equal_mpv_node(
                    (*l_a).values.offset(n as isize) as *mut mpv_node,
                    (*l_b).values.offset(n as isize) as *mut mpv_node,
                ) {
                    return false_0 != 0;
                }
                n += 1;
            }
            return true_0 != 0;
        }
        _ => {}
    }
    if (b"unreachable\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
    } else {
        __assert_fail(
            b"!\"unreachable\"\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/node.c\0" as *const u8 as *const ::core::ffi::c_char,
            162 as ::core::ffi::c_uint,
            b"_Bool equal_mpv_value(const void *, const void *, int)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    abort();
    panic!("Reached end of non-void function without returning");
}
#[no_mangle]
pub unsafe extern "C" fn equal_mpv_node(mut a: *const mpv_node, mut b: *const mpv_node) -> bool {
    if (*a).format as ::core::ffi::c_uint != (*b).format as ::core::ffi::c_uint {
        return false_0 != 0;
    }
    return equal_mpv_value(
        &raw const (*a).u as *const ::core::ffi::c_void,
        &raw const (*b).u as *const ::core::ffi::c_void,
        (*a).format as ::core::ffi::c_int,
    );
}
