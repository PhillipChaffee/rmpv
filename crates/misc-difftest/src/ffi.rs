//! mpv's C functions and the entry points of the C shims.

use std::ffi::{c_char, c_double, c_int, c_long, c_longlong, c_uint, c_void};

/// `struct bstr`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Bstr {
    pub start: *mut u8,
    pub len: usize,
}

impl Bstr {
    pub const NULL: Self = Self {
        start: std::ptr::null_mut(),
        len: 0,
    };

    /// A view of `s` for C code that only reads it.
    #[must_use]
    pub fn of(s: &[u8]) -> Self {
        Self {
            start: s.as_ptr().cast_mut(),
            len: s.len(),
        }
    }

    /// A view of `s` for C code that writes to it.
    #[must_use]
    pub fn of_mut(s: &mut [u8]) -> Self {
        Self {
            start: s.as_mut_ptr(),
            len: s.len(),
        }
    }
}

/// `struct pollfd`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PollFd {
    pub fd: c_int,
    pub events: i16,
    pub revents: i16,
}

/// `mp_rand_state`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MpRandState {
    pub v: [u64; 4],
}

unsafe extern "C" {
    pub fn free(p: *mut c_void);

    // shim/variadic_shim.c: `ints` holds 24 integer-class arguments and
    // `dbls` 8 doubles; `ptrs` holds 24 pointers.
    pub fn dt_snprintf(
        buf: *mut c_char,
        size: usize,
        fmt: *const c_char,
        ints: *const c_long,
        dbls: *const c_double,
        err: *mut c_int,
    ) -> c_int;
    pub fn dt_bstr_xappend_asprintf(
        s: *mut Bstr,
        fmt: *const c_char,
        ints: *const c_long,
        dbls: *const c_double,
    ) -> c_int;
    pub fn dt_sscanf(s: *const c_char, fmt: *const c_char, ptrs: *const *mut c_void) -> c_int;
    pub fn dt_bstr_sscanf(s: Bstr, fmt: *const c_char, ptrs: *const *mut c_void) -> c_int;
    pub fn strtof(s: *const c_char, end: *mut *mut c_char) -> f32;
    pub fn ta_free(p: *mut c_void);

    // misc/bstr.c
    pub fn bstrcmp(a: Bstr, b: Bstr) -> c_int;
    pub fn bstrcasecmp(a: Bstr, b: Bstr) -> c_int;
    pub fn bstrchr(s: Bstr, c: c_int) -> c_int;
    pub fn bstrrchr(s: Bstr, c: c_int) -> c_int;
    pub fn bstrcspn(s: Bstr, reject: *const c_char) -> c_int;
    pub fn bstrspn(s: Bstr, accept: *const c_char) -> c_int;
    pub fn bstr_find(haystack: Bstr, needle: Bstr) -> c_int;
    pub fn bstr_lstrip(s: Bstr) -> Bstr;
    pub fn bstr_strip(s: Bstr) -> Bstr;
    pub fn bstr_split(s: Bstr, sep: *const c_char, rest: *mut Bstr) -> Bstr;
    pub fn bstr_split_tok(s: Bstr, tok: *const c_char, left: *mut Bstr, right: *mut Bstr) -> bool;
    pub fn bstr_splice(s: Bstr, start: c_int, end: c_int) -> Bstr;
    pub fn bstrtoll(s: Bstr, rest: *mut Bstr, base: c_int) -> c_longlong;
    pub fn bstrtod(s: Bstr, rest: *mut Bstr) -> c_double;
    pub fn bstr_splitchar(s: Bstr, rest: *mut Bstr, c: c_char) -> Bstr;
    pub fn bstr_strip_linebreaks(s: Bstr) -> Bstr;
    pub fn bstr_eatstart(s: *mut Bstr, prefix: Bstr) -> bool;
    pub fn bstr_eatend(s: *mut Bstr, prefix: Bstr) -> bool;
    pub fn bstr_lower(s: Bstr);
    pub fn bstr_parse_utf8_code_length(b: u8) -> c_int;
    pub fn bstr_decode_partial_utf8(s: *mut Bstr) -> c_int;
    pub fn bstr_decode_utf8(s: Bstr, out_next: *mut Bstr) -> c_int;
    pub fn bstr_split_utf8(s: Bstr, out_next: *mut Bstr) -> Bstr;
    pub fn bstr_validate_utf8(s: Bstr) -> c_int;
    pub fn bstr_sanitize_utf8_latin1(talloc_ctx: *mut c_void, s: Bstr) -> Bstr;
    pub fn bstr_xappend(talloc_ctx: *mut c_void, s: *mut Bstr, append: Bstr);
    pub fn bstr_case_startswith(s: Bstr, prefix: Bstr) -> bool;
    pub fn bstr_case_endswith(s: Bstr, suffix: Bstr) -> bool;
    pub fn bstr_decode_hex(talloc_ctx: *mut c_void, hex: Bstr, out: *mut Bstr) -> bool;
    pub fn bstr_find_in_list0(s: Bstr, list: *mut *mut c_char, case_sensitive: bool) -> c_int;
    pub fn bstr_in_list0(s: Bstr, list: *mut *mut c_char) -> bool;
    pub fn dt_h_to_i(c: u8) -> c_int;

    // misc/bstr.h and misc/ctype.h
    pub fn dt_mp_isprint(c: c_char) -> c_int;
    pub fn dt_mp_isspace(c: c_char) -> c_int;
    pub fn dt_mp_isupper(c: c_char) -> c_int;
    pub fn dt_mp_islower(c: c_char) -> c_int;
    pub fn dt_mp_isdigit(c: c_char) -> c_int;
    pub fn dt_mp_isalpha(c: c_char) -> c_int;
    pub fn dt_mp_isalnum(c: c_char) -> c_int;
    pub fn dt_mp_tolower(c: c_char) -> c_char;
    pub fn dt_mp_toupper(c: c_char) -> c_char;
    pub fn dt_bstrdup0(s: Bstr) -> *mut c_char;
    pub fn dt_bstrto0(s: Bstr) -> *mut c_char;
    pub fn dt_bstrdup(s: Bstr) -> Bstr;
    pub fn dt_bstr0(s: *const c_char) -> Bstr;
    pub fn dt_bstr_getline(s: Bstr, rest: *mut Bstr) -> Bstr;
    pub fn dt_bstr_cut(s: Bstr, n: c_int) -> Bstr;
    pub fn dt_bstr_startswith(s: Bstr, p: Bstr) -> bool;
    pub fn dt_bstr_startswith0(s: Bstr, p: *const c_char) -> bool;
    pub fn dt_bstr_endswith(s: Bstr, p: Bstr) -> bool;
    pub fn dt_bstr_endswith0(s: Bstr, p: *const c_char) -> bool;
    pub fn dt_bstrcmp0(a: Bstr, b: *const c_char) -> c_int;
    pub fn dt_bstr_equals(a: Bstr, b: Bstr) -> bool;
    pub fn dt_bstr_equals0(a: Bstr, b: *const c_char) -> bool;
    pub fn dt_bstrcasecmp0(a: Bstr, b: *const c_char) -> c_int;
    pub fn dt_bstr_find0(h: Bstr, n: *const c_char) -> c_int;
    pub fn dt_bstr_eatstart0(s: *mut Bstr, p: *const c_char) -> bool;
    pub fn dt_bstr_eatend0(s: *mut Bstr, p: *const c_char) -> bool;
    pub fn dt_bstr_xappend0(s: *mut Bstr, a: *const c_char);

    // The C library, as the port's `clib` module stands in for it.
    pub fn dt_strtoll(
        s: *const c_char,
        base: c_int,
        consumed: *mut usize,
        erange: *mut c_int,
    ) -> c_longlong;
    pub fn dt_strtod(s: *const c_char, consumed: *mut usize, erange: *mut c_int) -> c_double;
    pub fn dt_memcmp(a: *const u8, b: *const u8, n: usize) -> c_int;
    pub fn dt_strncasecmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    pub fn dt_strcasecmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn dt_strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    pub fn dt_strcmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn dt_strchr(s: *const c_char, c: c_int) -> isize;
    pub fn dt_format_f(v: c_double, buf: *mut c_char, size: usize) -> c_int;
    pub fn dt_format_d(v: i64, buf: *mut c_char, size: usize) -> c_int;

    // common/common.c
    pub fn mp_log2(v: u32) -> c_uint;
    pub fn mp_round_next_power_of_2(v: u32) -> u32;
    pub fn mp_append_utf8_bstr(talloc_ctx: *mut c_void, buf: *mut Bstr, codepoint: u32);
    pub fn mp_append_escaped_string(
        talloc_ctx: *mut c_void,
        dst: *mut Bstr,
        src: *mut Bstr,
    ) -> bool;

    // misc/natural_sort.c
    pub fn mp_natural_sort_cmp(name1: *const c_char, name2: *const c_char) -> c_int;

    // misc/random.c
    pub fn mp_rand_seed(seed: u64) -> MpRandState;
    pub fn mp_rand_next(s: *mut MpRandState) -> u64;
    pub fn mp_rand_next_double(s: *mut MpRandState) -> c_double;
    pub fn mp_rand_in_range32(s: *mut MpRandState, min: u32, max: u32) -> u32;
    pub fn dt_rotl_u64(x: u64, k: c_int) -> u64;
    pub fn dt_splitmix64(x: *mut u64) -> u64;
    pub fn dt_fix_random_bytes(bytes: *const u8, count: c_int);

    // misc/codepoint_width.c
    pub fn term_disp_width(s: Bstr, max_width: c_int, cut_pos: *mut *const u8) -> c_int;
    pub fn dt_ucd_lookup(cp: u32) -> c_int;
    pub fn dt_ucd_grapheme_joins(state: c_int, lead: c_int, trail: c_int) -> c_int;
    pub fn dt_ucd_grapheme_done(state: c_int) -> bool;
    pub fn dt_ucd_to_character_width(val: c_int) -> c_int;
    pub fn dt_is_valid_partial_sequence(seq: Bstr) -> bool;

    // misc/hash.c
    pub fn mp_hash_to_bstr(
        talloc_ctx: *mut c_void,
        data: *const u8,
        len: usize,
        algorithm: *const c_char,
    ) -> Bstr;

    // misc/language.c
    pub fn mp_match_lang(langs: *mut *mut c_char, lang: *const c_char) -> c_int;
    pub fn mp_guess_lang_from_filename(
        name: Bstr,
        lang_start: *mut c_int,
        flags: *mut c_uint,
    ) -> Bstr;
    pub fn dt_langmap_len() -> c_int;
    pub fn dt_lang_compare(key: Bstr, index: c_int) -> c_int;
    pub fn dt_canonicalize(lang: Bstr) -> Bstr;

    // misc/path_utils.c
    pub fn mp_basename_bstr(path: Bstr) -> Bstr;
    pub fn mp_basename(path: *const c_char) -> *const c_char;
    pub fn mp_dirname(path: *const c_char) -> Bstr;
    pub fn mp_path_strip_trailing_separator(path: *mut c_char);
    pub fn dt_split_ext(path: Bstr, root: *mut Bstr) -> Bstr;
    pub fn mp_strip_ext(path: Bstr) -> Bstr;
    pub fn mp_get_ext(path: Bstr) -> Bstr;
    pub fn mp_path_is_absolute(path: Bstr) -> bool;
    pub fn mp_path_join_bstr(talloc_ctx: *mut c_void, p1: Bstr, p2: Bstr) -> *mut c_char;
    pub fn mp_path_join(
        talloc_ctx: *mut c_void,
        p1: *const c_char,
        p2: *const c_char,
    ) -> *mut c_char;
    pub fn mp_getcwd(talloc_ctx: *mut c_void) -> *mut c_char;
    pub fn mp_normalize_path(talloc_ctx: *mut c_void, path: *const c_char) -> *mut c_char;
    pub fn mp_path_exists(path: *const c_char) -> bool;
    pub fn mp_path_isdir(path: *const c_char) -> bool;
    pub fn mp_is_url(path: Bstr) -> bool;
    pub fn mp_split_proto(path: Bstr, out_url: *mut Bstr) -> Bstr;
    pub fn mp_mkdirp(dir: *const c_char);

    // misc/json.c and misc/node.c, through serialized nodes
    pub fn dt_json_parse(
        buf: *mut c_char,
        pos: *mut usize,
        max_depth: c_int,
        dump: *mut *mut u8,
        dump_len: *mut usize,
    ) -> c_int;
    pub fn dt_json_skip_whitespace(buf: *mut c_char, pos: usize) -> usize;
    pub fn dt_json_write(
        mode: c_int,
        node_wire: *const u8,
        prefix: *const c_char,
        indent: c_int,
        out: *mut *mut u8,
        out_len: *mut usize,
    ) -> c_int;
    pub fn dt_eat_c(buf: *mut c_char, pos: *mut usize, c: c_char) -> bool;
    pub fn dt_eat_ws(buf: *mut c_char, pos: usize) -> usize;
    pub fn dt_read(
        which: c_int,
        buf: *mut c_char,
        pos: *mut usize,
        max_depth: c_int,
        dump: *mut *mut u8,
        dump_len: *mut usize,
    ) -> c_int;
    pub fn dt_write_json_str(prefix: *const c_char, s: *mut u8, len: *mut usize) -> *mut u8;
    pub fn dt_add_indent(prefix: *const c_char, indent: c_int, len: *mut usize) -> *mut u8;
    pub fn dt_json_append_str(
        prefix: *const c_char,
        node_wire: *const u8,
        indent: c_int,
        r: *mut c_int,
        len: *mut usize,
    ) -> *mut u8;
    pub fn dt_node_add(
        op: c_int,
        node_wire: *const u8,
        key: Bstr,
        format: c_int,
        sval: Bstr,
        ival: i64,
        dval: c_double,
        index: *mut c_int,
        out_len: *mut usize,
    ) -> *mut u8;
    pub fn dt_node_init(format: c_int, parent_format: c_int, out_len: *mut usize) -> *mut u8;
    pub fn dt_node_map_get(node_wire: *const u8, key: Bstr, bkey: c_int) -> c_int;
    pub fn dt_node_equal(a_wire: *const u8, b_wire: *const u8, format: c_int) -> bool;

    // misc/io_utils.c
    pub fn mp_mkostemps(template: *mut c_char, suffixlen: c_int, flags: c_int) -> c_int;
    pub fn mp_save_to_file(filepath: *const c_char, data: *const c_void, size: usize) -> bool;
    pub fn close(fd: c_int) -> c_int;
    pub fn __errno_location() -> *mut c_int;

    // misc/thread_tools.c and misc/rendezvous.c
    pub fn dt_waiter_new() -> *mut c_void;
    pub fn mp_waiter_wait(waiter: *mut c_void) -> usize;
    pub fn mp_waiter_wakeup(waiter: *mut c_void, value: usize);
    pub fn mp_waiter_poll(waiter: *mut c_void) -> bool;
    pub fn mp_rendezvous(tag: *mut c_void, value: isize) -> isize;

    // misc/thread_tools.c: mp_cancel
    pub fn mp_cancel_new(talloc_ctx: *mut c_void) -> *mut c_void;
    pub fn mp_cancel_trigger(c: *mut c_void);
    pub fn mp_cancel_reset(c: *mut c_void);
    pub fn mp_cancel_test(c: *mut c_void) -> bool;
    pub fn mp_cancel_wait(c: *mut c_void, timeout: c_double) -> bool;
    pub fn mp_cancel_set_cb(
        c: *mut c_void,
        cb: Option<extern "C" fn(*mut c_void)>,
        ctx: *mut c_void,
    );
    pub fn mp_cancel_set_parent(slave: *mut c_void, parent: *mut c_void);
    pub fn mp_cancel_get_fd(c: *mut c_void) -> c_int;

    // misc/dispatch.c, and the talloc'd data mp_dispatch_enqueue_autofree
    // needs (shim/thread_shim.c)
    pub fn mp_dispatch_create(talloc_parent: *mut c_void) -> *mut c_void;
    pub fn mp_dispatch_set_wakeup_fn(
        q: *mut c_void,
        f: Option<extern "C" fn(*mut c_void)>,
        ctx: *mut c_void,
    );
    pub fn mp_dispatch_set_onlock_fn(
        q: *mut c_void,
        f: Option<extern "C" fn(*mut c_void)>,
        ctx: *mut c_void,
    );
    pub fn mp_dispatch_enqueue(q: *mut c_void, f: extern "C" fn(*mut c_void), data: *mut c_void);
    pub fn mp_dispatch_enqueue_autofree(
        q: *mut c_void,
        f: extern "C" fn(*mut c_void),
        data: *mut c_void,
    );
    pub fn mp_dispatch_enqueue_notify(
        q: *mut c_void,
        f: extern "C" fn(*mut c_void),
        data: *mut c_void,
    );
    pub fn mp_dispatch_cancel_fn(q: *mut c_void, f: extern "C" fn(*mut c_void), data: *mut c_void);
    pub fn mp_dispatch_run(q: *mut c_void, f: extern "C" fn(*mut c_void), data: *mut c_void);
    pub fn mp_dispatch_queue_process(q: *mut c_void, timeout: c_double);
    pub fn mp_dispatch_interrupt(q: *mut c_void);
    pub fn mp_dispatch_adjust_timeout(q: *mut c_void, until: i64);
    pub fn mp_dispatch_lock(q: *mut c_void);
    pub fn mp_dispatch_unlock(q: *mut c_void);
    pub fn dt_autofree_new(
        ctx: *mut c_void,
        id: usize,
        on_free: extern "C" fn(*mut c_void, usize),
    ) -> *mut c_void;

    // misc/thread_pool.c
    pub fn mp_thread_pool_create(
        ta_parent: *mut c_void,
        init_threads: c_int,
        min_threads: c_int,
        max_threads: c_int,
    ) -> *mut c_void;
    pub fn mp_thread_pool_queue(
        pool: *mut c_void,
        f: extern "C" fn(*mut c_void),
        ctx: *mut c_void,
    ) -> bool;
    pub fn mp_thread_pool_run(
        pool: *mut c_void,
        f: extern "C" fn(*mut c_void),
        ctx: *mut c_void,
    ) -> bool;

    // osdep/timer.c
    pub fn mp_time_init();
    pub fn mp_time_ns() -> i64;
    pub fn mp_time_ns_add(time_ns: i64, timeout_sec: c_double) -> i64;

    // The C library, to look at a wakeup pipe.
    pub fn ioctl(fd: c_int, request: std::ffi::c_ulong, ...) -> c_int;
    pub fn poll(fds: *mut PollFd, nfds: std::ffi::c_ulong, timeout: c_int) -> c_int;

    // misc/charset_conv.c
    pub fn mp_charset_is_utf8(user_cp: *const c_char) -> bool;
    pub fn mp_charset_is_utf16(user_cp: *const c_char) -> bool;
    pub fn dt_ms_bom_guess(buf: Bstr) -> *const c_char;
    pub fn dt_charset_guess(
        buf: Bstr,
        user_cp: *const c_char,
        flags: c_int,
        out: *mut c_char,
        size: usize,
    ) -> isize;
    pub fn dt_null_log() -> *mut c_void;
    pub fn mp_iconv_to_utf8(log: *mut c_void, buf: Bstr, cp: *const c_char, flags: c_int) -> Bstr;
}
