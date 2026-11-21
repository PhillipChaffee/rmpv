extern "C" {
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn abort() -> !;
    fn getenv(__name: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char;
    fn realpath(
        __name: *const ::core::ffi::c_char,
        __resolved: *mut ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn memchr(
        __s: *const ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strcmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn strchr(__s: *const ::core::ffi::c_char, __c: ::core::ffi::c_int)
        -> *mut ::core::ffi::c_char;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn stat(__file: *const ::core::ffi::c_char, __buf: *mut stat) -> ::core::ffi::c_int;
    fn mkdir(__path: *const ::core::ffi::c_char, __mode: __mode_t) -> ::core::ffi::c_int;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn ta_alloc_size(ta_parent: *mut ::core::ffi::c_void, size: size_t)
        -> *mut ::core::ffi::c_void;
    fn ta_get_size(ptr: *mut ::core::ffi::c_void) -> size_t;
    fn ta_free(ptr: *mut ::core::ffi::c_void);
    fn ta_calc_array_size(element_size: size_t, count: size_t) -> size_t;
    fn ta_new_context(ta_parent: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void;
    fn ta_steal_(
        ta_parent: *mut ::core::ffi::c_void,
        ptr: *mut ::core::ffi::c_void,
    ) -> *mut ::core::ffi::c_void;
    fn ta_asprintf(
        ta_parent: *mut ::core::ffi::c_void,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> *mut ::core::ffi::c_char;
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
    fn ta_talloc_strdup_append_buffer(
        s: *mut ::core::ffi::c_char,
        a: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_char;
    fn ta_talloc_strndup_append_buffer(
        s: *mut ::core::ffi::c_char,
        a: *const ::core::ffi::c_char,
        n: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn getcwd(__buf: *mut ::core::ffi::c_char, __size: size_t) -> *mut ::core::ffi::c_char;
    fn bstrrchr(str: bstr, c: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn bstrspn(str: bstr, accept: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn bstr_find(haystack: bstr, needle: bstr) -> ::core::ffi::c_int;
    fn bstr_split_tok(
        str: bstr,
        tok: *const ::core::ffi::c_char,
        out_left: *mut bstr,
        out_right: *mut bstr,
    ) -> bool;
    fn bstr_splice(str: bstr, start: ::core::ffi::c_int, end: ::core::ffi::c_int) -> bstr;
}
pub type size_t = usize;
pub type __uint8_t = u8;
pub type __dev_t = ::core::ffi::c_ulong;
pub type __uid_t = ::core::ffi::c_uint;
pub type __gid_t = ::core::ffi::c_uint;
pub type __ino_t = ::core::ffi::c_ulong;
pub type __mode_t = ::core::ffi::c_uint;
pub type __nlink_t = ::core::ffi::c_ulong;
pub type __off_t = ::core::ffi::c_long;
pub type __time_t = ::core::ffi::c_long;
pub type __blksize_t = ::core::ffi::c_long;
pub type __blkcnt_t = ::core::ffi::c_long;
pub type __syscall_slong_t = ::core::ffi::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stat {
    pub st_dev: __dev_t,
    pub st_ino: __ino_t,
    pub st_nlink: __nlink_t,
    pub st_mode: __mode_t,
    pub st_uid: __uid_t,
    pub st_gid: __gid_t,
    pub __pad0: ::core::ffi::c_int,
    pub st_rdev: __dev_t,
    pub st_size: __off_t,
    pub st_blksize: __blksize_t,
    pub st_blocks: __blkcnt_t,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    pub __glibc_reserved: [__syscall_slong_t; 3],
}
pub type uint8_t = __uint8_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bstr {
    pub start: *mut ::core::ffi::c_uchar,
    pub len: size_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const ERANGE: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn ta_oom_p(mut p: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    if p.is_null() {
        abort();
    }
    return p;
}
#[inline]
unsafe extern "C" fn ta_oom_s(mut s: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if s.is_null() {
        abort();
    }
    return s;
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
#[inline]
unsafe extern "C" fn bstrdup0(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut str: bstr,
) -> *mut ::core::ffi::c_char {
    return ta_xstrndup(
        talloc_ctx,
        str.start as *mut ::core::ffi::c_char,
        str.len as size_t,
    );
}
#[inline]
unsafe extern "C" fn bstr0(mut s: *const ::core::ffi::c_char) -> bstr {
    return bstr {
        start: s as *mut ::core::ffi::c_uchar,
        len: if !s.is_null() {
            strlen(s) as size_t
        } else {
            0 as size_t
        },
    };
}
#[inline]
unsafe extern "C" fn bstr_find0(
    mut haystack: bstr,
    mut needle: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return bstr_find(haystack, bstr0(needle));
}
#[no_mangle]
pub unsafe extern "C" fn mp_basename_bstr(mut path: bstr) -> bstr {
    let mut separator_pos: ::core::ffi::c_int = 0;
    separator_pos = bstrrchr(path as bstr, '/' as i32);
    if separator_pos < 0 as ::core::ffi::c_int {
        return path;
    }
    return bstr_splice(
        path as bstr,
        separator_pos + 1 as ::core::ffi::c_int,
        path.len as ::core::ffi::c_int,
    ) as bstr;
}
#[no_mangle]
pub unsafe extern "C" fn mp_basename(
    mut path: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    return mp_basename_bstr(bstr0(path) as bstr).start as *const ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn mp_dirname(mut path: *const ::core::ffi::c_char) -> bstr {
    let mut ret: bstr = bstr {
        start: path as *mut ::core::ffi::c_uchar,
        len: mp_basename(path).offset_from(path) as ::core::ffi::c_long as size_t,
    };
    if ret.len == 0 as size_t {
        return bstr0(b".\0" as *const u8 as *const ::core::ffi::c_char);
    }
    return ret;
}
static mut mp_path_separators: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
#[no_mangle]
pub unsafe extern "C" fn mp_path_strip_trailing_separator(mut path: *mut ::core::ffi::c_char) {
    let mut len: size_t = strlen(path) as size_t;
    if len > 0 as size_t
        && !strchr(
            &raw const mp_path_separators as *const ::core::ffi::c_char,
            *path.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
        )
        .is_null()
    {
        *path.offset(len.wrapping_sub(1 as size_t) as isize) = '\0' as i32 as ::core::ffi::c_char;
    }
}
unsafe extern "C" fn split_ext(mut path: bstr, mut root: *mut bstr) -> bstr {
    let mut basename: bstr = mp_basename_bstr(path);
    basename = bstr_splice(
        basename as bstr,
        bstrspn(
            basename as bstr,
            b".\0" as *const u8 as *const ::core::ffi::c_char,
        ),
        basename.len as ::core::ffi::c_int,
    ) as bstr;
    let mut dot_pos: ::core::ffi::c_int = bstrrchr(basename as bstr, '.' as i32);
    if dot_pos < 0 as ::core::ffi::c_int
        || dot_pos as size_t == basename.len.wrapping_sub(1 as size_t)
    {
        return bstr {
            start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
            len: 0,
        };
    }
    let mut ext: bstr = bstr_splice(
        basename as bstr,
        dot_pos + 1 as ::core::ffi::c_int,
        basename.len as ::core::ffi::c_int,
    ) as bstr;
    if !root.is_null() {
        *root = bstr {
            start: path.start,
            len: ext
                .start
                .offset(-(1 as ::core::ffi::c_int as isize))
                .offset_from(path.start) as ::core::ffi::c_long as size_t,
        };
    }
    return ext;
}
#[no_mangle]
pub unsafe extern "C" fn mp_strip_ext(mut path: bstr) -> bstr {
    let mut root: bstr = bstr {
        start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        len: 0,
    };
    split_ext(path, &raw mut root);
    return if root.len != 0 { root } else { path };
}
#[no_mangle]
pub unsafe extern "C" fn mp_get_ext(mut path: bstr) -> bstr {
    return split_ext(path, ::core::ptr::null_mut::<bstr>());
}
#[no_mangle]
pub unsafe extern "C" fn mp_path_is_absolute(mut path: bstr) -> bool {
    if path.len != 0
        && !strchr(
            &raw const mp_path_separators as *const ::core::ffi::c_char,
            *path.start.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int,
        )
        .is_null()
    {
        return true_0 != 0;
    }
    return false_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn mp_path_join_bstr(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut p1: bstr,
    mut p2: bstr,
) -> *mut ::core::ffi::c_char {
    if p1.len == 0 as size_t {
        return bstrdup0(talloc_ctx, p2);
    }
    if p2.len == 0 as size_t {
        return bstrdup0(talloc_ctx, p1);
    }
    if mp_path_is_absolute(p2) {
        return bstrdup0(talloc_ctx, p2);
    }
    let mut have_separator: bool = !strchr(
        &raw const mp_path_separators as *const ::core::ffi::c_char,
        *p1.start.offset(p1.len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
    )
    .is_null();
    return ta_oom_s(ta_asprintf(
        talloc_ctx,
        b"%.*s%s%.*s\0" as *const u8 as *const ::core::ffi::c_char,
        p1.len as ::core::ffi::c_int,
        if !p1.start.is_null() {
            p1.start as *mut ::core::ffi::c_char as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
        if have_separator as ::core::ffi::c_int != 0 {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        } else {
            b"/\0" as *const u8 as *const ::core::ffi::c_char
        },
        p2.len as ::core::ffi::c_int,
        if !p2.start.is_null() {
            p2.start as *mut ::core::ffi::c_char as *const ::core::ffi::c_char
        } else {
            b"\0" as *const u8 as *const ::core::ffi::c_char
        },
    ));
}
#[no_mangle]
pub unsafe extern "C" fn mp_path_join(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut p1: *const ::core::ffi::c_char,
    mut p2: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return mp_path_join_bstr(talloc_ctx, bstr0(p1), bstr0(p2));
}
#[no_mangle]
pub unsafe extern "C" fn mp_getcwd(
    mut talloc_ctx: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    let mut e_wd: *mut ::core::ffi::c_char =
        getenv(b"PWD\0" as *const u8 as *const ::core::ffi::c_char);
    let mut pwd_st: stat = stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __glibc_reserved: [0; 3],
    };
    let mut dot_st: stat = stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __glibc_reserved: [0; 3],
    };
    if !e_wd.is_null()
        && mp_path_is_absolute(bstr0(e_wd)) as ::core::ffi::c_int != 0
        && stat(e_wd, &raw mut pwd_st) == 0 as ::core::ffi::c_int
        && stat(
            b".\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut dot_st,
        ) == 0 as ::core::ffi::c_int
        && pwd_st.st_dev == dot_st.st_dev
        && pwd_st.st_ino == dot_st.st_ino
    {
        return ta_xstrdup(talloc_ctx, e_wd);
    }
    let mut wd: *mut ::core::ffi::c_char = ta_oom_p(ta_dbg_set_loc(
        ta_alloc_size(
            talloc_ctx,
            ta_calc_array_size(
                ::core::mem::size_of::<::core::ffi::c_char>() as size_t,
                20 as size_t,
            ),
        ),
        b"../.mpv/misc/path_utils.c:179\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut ::core::ffi::c_char
        as *mut ::core::ffi::c_void)
        as *mut ::core::ffi::c_char;
    while getcwd(wd, ta_get_size(wd as *mut ::core::ffi::c_void)).is_null() {
        if *__errno_location() != ERANGE {
            ta_free(wd as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        wd = ta_dbg_set_loc(
            ta_xrealloc_size(
                talloc_ctx,
                wd as *mut ::core::ffi::c_void,
                ta_calc_array_size(
                    ::core::mem::size_of::<::core::ffi::c_char>() as size_t,
                    ta_get_size(wd as *mut ::core::ffi::c_void).wrapping_mul(2 as size_t),
                ),
            ),
            b"../.mpv/misc/path_utils.c:185\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut ::core::ffi::c_char;
    }
    return wd;
}
#[no_mangle]
pub unsafe extern "C" fn mp_normalize_path(
    mut talloc_ctx: *mut ::core::ffi::c_void,
    mut path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut next: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut end: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut current_block: u64;
    if path.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if *path.offset(0 as ::core::ffi::c_int as isize) == 0
        || mp_is_url(bstr0(path) as bstr) as ::core::ffi::c_int != 0
        || strcmp(path, b"-\0" as *const u8 as *const ::core::ffi::c_char) == 0
    {
        return ta_xstrdup(talloc_ctx, path);
    }
    let mut tmp: *mut ::core::ffi::c_void = ta_oom_p(ta_new_context(::core::ptr::null_mut::<
        ::core::ffi::c_void,
    >()));
    let mut result: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !mp_path_is_absolute(bstr0(path)) {
        let mut cwd: *mut ::core::ffi::c_char = mp_getcwd(tmp);
        if cwd.is_null() {
            result = ta_xstrdup(talloc_ctx, path);
            current_block = 5742682845111736798;
        } else {
            path = mp_path_join(tmp, cwd, path);
            current_block = 6937071982253665452;
        }
    } else {
        current_block = 6937071982253665452;
    }
    match current_block {
        6937071982253665452 => {
            result = ta_xstrdup(tmp, b"\0" as *const u8 as *const ::core::ffi::c_char);
            next = ::core::ptr::null::<::core::ffi::c_char>();
            end = path.offset(strlen(path) as isize);
            let mut ptr: *const ::core::ffi::c_char = path;
            loop {
                if !(ptr < end) {
                    current_block = 11042950489265723346;
                    break;
                }
                next = memchr(
                    ptr as *const ::core::ffi::c_void,
                    '/' as i32,
                    end.offset_from(ptr) as ::core::ffi::c_long as size_t,
                ) as *const ::core::ffi::c_char;
                if next.is_null() {
                    next = end;
                }
                match next.offset_from(ptr) as ::core::ffi::c_long {
                    0 => {
                        current_block = 11812396948646013369;
                    }
                    1 => {
                        if *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as i32
                        {
                            current_block = 11812396948646013369;
                        } else {
                            current_block = 4808432441040389987;
                        }
                    }
                    2 => {
                        if *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as i32
                            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '.' as i32
                        {
                            let mut tmp_result: *mut ::core::ffi::c_char =
                                realpath(path, ::core::ptr::null_mut::<::core::ffi::c_char>());
                            result = ta_xstrdup(
                                talloc_ctx,
                                if !tmp_result.is_null() {
                                    tmp_result as *const ::core::ffi::c_char
                                } else {
                                    path
                                },
                            );
                            free(tmp_result as *mut ::core::ffi::c_void);
                            current_block = 5742682845111736798;
                            break;
                        } else {
                            current_block = 4808432441040389987;
                        }
                    }
                    _ => {
                        current_block = 4808432441040389987;
                    }
                }
                match current_block {
                    4808432441040389987 => {
                        result = ta_talloc_strdup_append_buffer(
                            result,
                            b"/\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                        result = ta_talloc_strndup_append_buffer(
                            result,
                            ptr,
                            next.offset_from(ptr) as ::core::ffi::c_long as size_t,
                        );
                    }
                    _ => {}
                }
                ptr = next.offset(1 as ::core::ffi::c_int as isize);
            }
            match current_block {
                5742682845111736798 => {}
                _ => {
                    if *result.offset(0 as ::core::ffi::c_int as isize) == 0 {
                        result = ta_talloc_strdup_append_buffer(
                            result,
                            b"/\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    result = ta_steal_(talloc_ctx, result as *mut ::core::ffi::c_void)
                        as *mut ::core::ffi::c_char
                        as *mut ::core::ffi::c_char;
                }
            }
        }
        _ => {}
    }
    ta_free(tmp);
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn mp_path_exists(mut path: *const ::core::ffi::c_char) -> bool {
    let mut st: stat = stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __glibc_reserved: [0; 3],
    };
    return !path.is_null() && stat(path, &raw mut st) == 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn mp_path_isdir(mut path: *const ::core::ffi::c_char) -> bool {
    let mut st: stat = stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __glibc_reserved: [0; 3],
    };
    return stat(path, &raw mut st) == 0 as ::core::ffi::c_int
        && st.st_mode & __S_IFMT as __mode_t == 0o40000 as __mode_t;
}
#[no_mangle]
pub unsafe extern "C" fn mp_is_url(mut path: bstr) -> bool {
    let mut proto: ::core::ffi::c_int = bstr_find0(
        path as bstr,
        b"://\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if proto < 1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < proto {
        let mut c: ::core::ffi::c_uchar = *path.start.offset(i as isize);
        if i == 0 as ::core::ffi::c_int && mp_isalpha(c as ::core::ffi::c_char) == 0
            || mp_isalnum(c as ::core::ffi::c_char) == 0
                && c as ::core::ffi::c_int != '.' as i32
                && c as ::core::ffi::c_int != '-' as i32
                && c as ::core::ffi::c_int != '+' as i32
        {
            return false_0 != 0;
        }
        i += 1;
    }
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn mp_split_proto(mut path: bstr, mut out_url: *mut bstr) -> bstr {
    if !mp_is_url(path) {
        return bstr {
            start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
            len: 0,
        };
    }
    let mut r: bstr = bstr {
        start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
        len: 0,
    };
    bstr_split_tok(
        path,
        b"://\0" as *const u8 as *const ::core::ffi::c_char,
        &raw mut r,
        if !out_url.is_null() {
            out_url
        } else {
            let mut fresh0: bstr = bstr {
                start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
                len: 0,
            };
            &raw mut fresh0
        },
    );
    return r;
}
#[no_mangle]
pub unsafe extern "C" fn mp_mkdirp(mut dir: *const ::core::ffi::c_char) {
    if mp_path_exists(dir) {
        return;
    }
    let mut path: *mut ::core::ffi::c_char = ta_xstrdup(NULL, dir);
    let mut cdir: *mut ::core::ffi::c_char = path.offset(1 as ::core::ffi::c_int as isize);
    while !cdir.is_null() {
        cdir = strchr(cdir, '/' as i32);
        if !cdir.is_null() {
            *cdir = 0 as ::core::ffi::c_char;
        }
        mkdir(path, 0o777 as __mode_t);
        if !cdir.is_null() {
            let fresh1 = cdir;
            cdir = cdir.offset(1);
            *fresh1 = '/' as i32 as ::core::ffi::c_char;
        }
    }
    ta_free(path as *mut ::core::ffi::c_void);
}
