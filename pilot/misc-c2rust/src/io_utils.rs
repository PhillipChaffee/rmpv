extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn abort() -> !;
    fn rename(
        __old: *const ::core::ffi::c_char,
        __new: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn fclose(__stream: *mut FILE) -> ::core::ffi::c_int;
    fn fdopen(__fd: ::core::ffi::c_int, __modes: *const ::core::ffi::c_char) -> *mut FILE;
    fn fwrite(
        __ptr: *const ::core::ffi::c_void,
        __size: size_t,
        __n: size_t,
        __s: *mut FILE,
    ) -> ::core::ffi::c_ulong;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn ta_free(ptr: *mut ::core::ffi::c_void);
    fn ta_asprintf(
        ta_parent: *mut ::core::ffi::c_void,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> *mut ::core::ffi::c_char;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn unlink(__name: *const ::core::ffi::c_char) -> ::core::ffi::c_int;
    fn mp_tprintf_buf(
        buf: *mut ::core::ffi::c_char,
        buf_size: size_t,
        format: *const ::core::ffi::c_char,
        ...
    ) -> *mut ::core::ffi::c_char;
    fn mp_rand_seed(seed: uint64_t) -> mp_rand_state;
    fn mp_rand_next(s: *mut mp_rand_state) -> uint64_t;
    fn open(
        __file: *const ::core::ffi::c_char,
        __oflag: ::core::ffi::c_int,
        ...
    ) -> ::core::ffi::c_int;
}
pub type __uint64_t = u64;
pub type __off_t = ::core::ffi::c_long;
pub type __off64_t = ::core::ffi::c_long;
pub type uint64_t = __uint64_t;
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: ::core::ffi::c_int,
    pub _IO_read_ptr: *mut ::core::ffi::c_char,
    pub _IO_read_end: *mut ::core::ffi::c_char,
    pub _IO_read_base: *mut ::core::ffi::c_char,
    pub _IO_write_base: *mut ::core::ffi::c_char,
    pub _IO_write_ptr: *mut ::core::ffi::c_char,
    pub _IO_write_end: *mut ::core::ffi::c_char,
    pub _IO_buf_base: *mut ::core::ffi::c_char,
    pub _IO_buf_end: *mut ::core::ffi::c_char,
    pub _IO_save_base: *mut ::core::ffi::c_char,
    pub _IO_backup_base: *mut ::core::ffi::c_char,
    pub _IO_save_end: *mut ::core::ffi::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: ::core::ffi::c_int,
    pub _flags2: ::core::ffi::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: ::core::ffi::c_ushort,
    pub _vtable_offset: ::core::ffi::c_schar,
    pub _shortbuf: [::core::ffi::c_char; 1],
    pub _lock: *mut ::core::ffi::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut ::core::ffi::c_void,
    pub __pad5: size_t,
    pub _mode: ::core::ffi::c_int,
    pub _unused2: [::core::ffi::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_rand_state {
    pub v: [uint64_t; 4],
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 58] = unsafe {
    ::core::mem::transmute::<[u8; 58], [::core::ffi::c_char; 58]>(
        *b"_Bool mp_save_to_file(const char *, const void *, size_t)\0",
    )
};
pub const EEXIST: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn ta_oom_s(mut s: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if s.is_null() {
        abort();
    }
    return s;
}
pub const O_RDWR: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_EXCL: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const __O_CLOEXEC: ::core::ffi::c_int = 0o2000000 as ::core::ffi::c_int;
pub const O_CLOEXEC: ::core::ffi::c_int = __O_CLOEXEC;
#[no_mangle]
pub unsafe extern "C" fn mp_mkostemps(
    mut template: *mut ::core::ffi::c_char,
    mut suffixlen: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut len: size_t = strlen(template) as size_t;
    let mut t: *mut ::core::ffi::c_char = if len >= (6 as ::core::ffi::c_int + suffixlen) as size_t
    {
        template.offset(len.wrapping_sub((6 as ::core::ffi::c_int + suffixlen) as size_t) as isize)
            as *mut ::core::ffi::c_char
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    if t.is_null()
        || strncmp(
            t,
            b"XXXXXX\0" as *const u8 as *const ::core::ffi::c_char,
            6 as size_t,
        ) != 0 as ::core::ffi::c_int
    {
        *__errno_location() = EINVAL;
        return -(1 as ::core::ffi::c_int);
    }
    let mut s: mp_rand_state = mp_rand_seed(0 as uint64_t);
    let mut fuckshit: size_t = 0 as size_t;
    while fuckshit < UINT32_MAX as size_t {
        let mut fuckmess: size_t = mp_rand_next(&raw mut s) as size_t;
        let mut crap: [::core::ffi::c_char; 7] =
            ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"\0\0\0\0\0\0\0");
        mp_tprintf_buf(
            &raw mut crap as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>() as size_t,
            b"%06zx\0" as *const u8 as *const ::core::ffi::c_char,
            fuckmess,
        );
        memcpy(
            t as *mut ::core::ffi::c_void,
            &raw mut crap as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            6 as size_t,
        );
        let mut res: ::core::ffi::c_int = open(
            template,
            O_RDWR | O_CREAT | O_EXCL | flags,
            0o600 as ::core::ffi::c_int,
        );
        if res >= 0 as ::core::ffi::c_int || *__errno_location() != EEXIST {
            return res;
        }
        fuckshit = fuckshit.wrapping_add(1);
    }
    *__errno_location() = EEXIST;
    return -(1 as ::core::ffi::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn mp_save_to_file(
    mut filepath: *const ::core::ffi::c_char,
    mut data: *const ::core::ffi::c_void,
    mut size: size_t,
) -> bool {
    let mut fs: *mut FILE = ::core::ptr::null_mut::<FILE>();
    let mut written: size_t = 0;
    let mut ret: ::core::ffi::c_int = 0;
    if !filepath.is_null() {
    } else {
        __assert_fail(
            b"filepath\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/io_utils.c\0" as *const u8 as *const ::core::ffi::c_char,
            63 as ::core::ffi::c_uint,
            __ASSERT_FUNCTION.as_ptr(),
        );
    };
    let mut result: bool = false_0 != 0;
    let mut tmp: *mut ::core::ffi::c_char = ta_oom_s(ta_asprintf(
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        b"%sXXXXXX\0" as *const u8 as *const ::core::ffi::c_char,
        filepath,
    ));
    let mut fd: ::core::ffi::c_int = mp_mkostemps(tmp, 0 as ::core::ffi::c_int, O_CLOEXEC);
    if !(fd < 0 as ::core::ffi::c_int) {
        fs = fdopen(fd, b"wb\0" as *const u8 as *const ::core::ffi::c_char);
        if fs.is_null() {
            close(fd);
            unlink(tmp);
        } else {
            written = if !data.is_null() && size != 0 {
                fwrite(data, size as size_t, 1 as size_t, fs) as size_t
            } else {
                1 as size_t
            };
            ret = fclose(fs);
            if written > 0 as size_t && ret == 0 {
                ret = rename(tmp, filepath);
                result = ret == 0;
            } else {
                unlink(tmp);
            }
        }
    }
    ta_free(tmp as *mut ::core::ffi::c_void);
    return result;
}
