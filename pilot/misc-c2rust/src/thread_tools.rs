extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn __errno_location() -> *mut ::core::ffi::c_int;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn abort() -> !;
    fn close(__fd: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn read(__fd: ::core::ffi::c_int, __buf: *mut ::core::ffi::c_void, __nbytes: size_t)
        -> ssize_t;
    fn write(__fd: ::core::ffi::c_int, __buf: *const ::core::ffi::c_void, __n: size_t) -> ssize_t;
    fn ta_alloc_size(ta_parent: *mut ::core::ffi::c_void, size: size_t)
        -> *mut ::core::ffi::c_void;
    fn ta_set_destructor(
        ptr: *mut ::core::ffi::c_void,
        destructor: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    );
    fn ta_dbg_set_loc(
        ptr: *mut ::core::ffi::c_void,
        name: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    fn mp_make_wakeup_pipe(pipes: *mut ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn clock_gettime(__clock_id: clockid_t, __tp: *mut timespec) -> ::core::ffi::c_int;
    fn mp_time_ns() -> int64_t;
    fn mp_time_ns_add(time_ns: int64_t, timeout_sec: ::core::ffi::c_double) -> int64_t;
    fn pthread_mutex_init(
        __mutex: *mut pthread_mutex_t,
        __mutexattr: *const pthread_mutexattr_t,
    ) -> ::core::ffi::c_int;
    fn pthread_mutex_destroy(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn pthread_mutex_lock(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn pthread_mutex_unlock(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn pthread_mutexattr_init(__attr: *mut pthread_mutexattr_t) -> ::core::ffi::c_int;
    fn pthread_mutexattr_destroy(__attr: *mut pthread_mutexattr_t) -> ::core::ffi::c_int;
    fn pthread_mutexattr_settype(
        __attr: *mut pthread_mutexattr_t,
        __kind: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn pthread_cond_init(
        __cond: *mut pthread_cond_t,
        __cond_attr: *const pthread_condattr_t,
    ) -> ::core::ffi::c_int;
    fn pthread_cond_destroy(__cond: *mut pthread_cond_t) -> ::core::ffi::c_int;
    fn pthread_cond_signal(__cond: *mut pthread_cond_t) -> ::core::ffi::c_int;
    fn pthread_cond_broadcast(__cond: *mut pthread_cond_t) -> ::core::ffi::c_int;
    fn pthread_cond_wait(
        __cond: *mut pthread_cond_t,
        __mutex: *mut pthread_mutex_t,
    ) -> ::core::ffi::c_int;
    fn pthread_cond_timedwait(
        __cond: *mut pthread_cond_t,
        __mutex: *mut pthread_mutex_t,
        __abstime: *const timespec,
    ) -> ::core::ffi::c_int;
    fn pthread_condattr_init(__attr: *mut pthread_condattr_t) -> ::core::ffi::c_int;
    fn pthread_condattr_destroy(__attr: *mut pthread_condattr_t) -> ::core::ffi::c_int;
    fn pthread_condattr_setclock(
        __attr: *mut pthread_condattr_t,
        __clock_id: __clockid_t,
    ) -> ::core::ffi::c_int;
}
pub type size_t = usize;
pub type __int64_t = i64;
pub type __time_t = ::core::ffi::c_long;
pub type __clockid_t = ::core::ffi::c_int;
pub type __syscall_slong_t = ::core::ffi::c_long;
pub type int64_t = __int64_t;
pub type uintptr_t = usize;
pub type memory_order = ::core::ffi::c_uint;
pub const memory_order_seq_cst: memory_order = 5;
pub const memory_order_acq_rel: memory_order = 4;
pub const memory_order_release: memory_order = 3;
pub const memory_order_acquire: memory_order = 2;
pub const memory_order_consume: memory_order = 1;
pub const memory_order_relaxed: memory_order = 0;
pub type atomic_bool = bool;
pub type ssize_t = isize;
pub type clockid_t = __clockid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union __atomic_wide_counter {
    pub __value64: ::core::ffi::c_ulonglong,
    pub __value32: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub __low: ::core::ffi::c_uint,
    pub __high: ::core::ffi::c_uint,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_mutex_s {
    pub __lock: ::core::ffi::c_int,
    pub __count: ::core::ffi::c_uint,
    pub __owner: ::core::ffi::c_int,
    pub __nusers: ::core::ffi::c_uint,
    pub __kind: ::core::ffi::c_int,
    pub __spins: ::core::ffi::c_short,
    pub __elision: ::core::ffi::c_short,
    pub __list: __pthread_list_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_cond_s {
    pub __wseq: __atomic_wide_counter,
    pub __g1_start: __atomic_wide_counter,
    pub __g_refs: [::core::ffi::c_uint; 2],
    pub __g_size: [::core::ffi::c_uint; 2],
    pub __g1_orig_size: ::core::ffi::c_uint,
    pub __wrefs: ::core::ffi::c_uint,
    pub __g_signals: [::core::ffi::c_uint; 2],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutexattr_t {
    pub __size: [::core::ffi::c_char; 4],
    pub __align: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_condattr_t {
    pub __size: [::core::ffi::c_char; 4],
    pub __align: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_cond_t {
    pub __data: __pthread_cond_s,
    pub __size: [::core::ffi::c_char; 48],
    pub __align: ::core::ffi::c_longlong,
}
pub type C2RustUnnamed_0 = ::core::ffi::c_uint;
pub const PTHREAD_MUTEX_FAST_NP: C2RustUnnamed_0 = 0;
pub const PTHREAD_MUTEX_DEFAULT: C2RustUnnamed_0 = 0;
pub const PTHREAD_MUTEX_ERRORCHECK: C2RustUnnamed_0 = 2;
pub const PTHREAD_MUTEX_RECURSIVE: C2RustUnnamed_0 = 1;
pub const PTHREAD_MUTEX_NORMAL: C2RustUnnamed_0 = 0;
pub const PTHREAD_MUTEX_ADAPTIVE_NP: C2RustUnnamed_0 = 3;
pub const PTHREAD_MUTEX_ERRORCHECK_NP: C2RustUnnamed_0 = 2;
pub const PTHREAD_MUTEX_RECURSIVE_NP: C2RustUnnamed_0 = 1;
pub const PTHREAD_MUTEX_TIMED_NP: C2RustUnnamed_0 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_cond {
    pub cond: pthread_cond_t,
    pub clk_id: clockid_t,
}
pub type mp_mutex = pthread_mutex_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_waiter {
    pub lock: mp_mutex,
    pub wakeup: mp_cond,
    pub done: bool,
    pub value: uintptr_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_cancel {
    pub lock: mp_mutex,
    pub wakeup: mp_cond,
    pub triggered: atomic_bool,
    pub cb: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub cb_ctx: *mut ::core::ffi::c_void,
    pub wakeup_pipe: [::core::ffi::c_int; 2],
    pub win32_event: *mut ::core::ffi::c_void,
    pub slaves: C2RustUnnamed_2,
    pub parent: *mut mp_cancel,
    pub siblings: C2RustUnnamed_1,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_1 {
    pub next: *mut mp_cancel,
    pub prev: *mut mp_cancel,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_2 {
    pub head: *mut mp_cancel,
    pub tail: *mut mp_cancel,
}
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
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
pub const CLOCK_REALTIME: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CLOCK_MONOTONIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
#[inline]
unsafe extern "C" fn mp_mutex_init(mut mutex: *mut mp_mutex) -> ::core::ffi::c_int {
    let mut mutex_type: ::core::ffi::c_int = PTHREAD_MUTEX_ERRORCHECK as ::core::ffi::c_int;
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut attr: pthread_mutexattr_t = pthread_mutexattr_t { __size: [0; 4] };
    ret = pthread_mutexattr_init(&raw mut attr);
    if ret != 0 as ::core::ffi::c_int {
        return ret;
    }
    pthread_mutexattr_settype(&raw mut attr, mutex_type);
    ret = pthread_mutex_init(mutex as *mut pthread_mutex_t, &raw mut attr);
    pthread_mutexattr_destroy(&raw mut attr);
    if ret == 0 {
    } else {
        __assert_fail(
            b"!ret\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/osdep/threads-posix.h\0" as *const u8 as *const ::core::ffi::c_char,
            129 as ::core::ffi::c_uint,
            b"int mp_mutex_init(mp_mutex *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return ret;
}
#[inline]
unsafe extern "C" fn mp_cond_init(mut cond: *mut mp_cond) -> ::core::ffi::c_int {
    if !cond.is_null() {
    } else {
        __assert_fail(
            b"cond\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/osdep/threads-posix.h\0" as *const u8 as *const ::core::ffi::c_char,
            140 as ::core::ffi::c_uint,
            b"int mp_cond_init(mp_cond *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut attr: pthread_condattr_t = pthread_condattr_t { __size: [0; 4] };
    ret = pthread_condattr_init(&raw mut attr);
    if ret != 0 {
        return ret;
    }
    (*cond).clk_id = CLOCK_REALTIME as clockid_t;
    if pthread_condattr_setclock(&raw mut attr, CLOCK_MONOTONIC) == 0 {
        (*cond).clk_id = CLOCK_MONOTONIC as clockid_t;
    }
    ret = pthread_cond_init(&raw mut (*cond).cond, &raw mut attr);
    pthread_condattr_destroy(&raw mut attr);
    return ret;
}
#[inline]
unsafe extern "C" fn mp_cond_destroy(mut cond: *mut mp_cond) -> ::core::ffi::c_int {
    if !cond.is_null() {
    } else {
        __assert_fail(
            b"cond\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/osdep/threads-posix.h\0" as *const u8 as *const ::core::ffi::c_char,
            161 as ::core::ffi::c_uint,
            b"int mp_cond_destroy(mp_cond *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return pthread_cond_destroy(&raw mut (*cond).cond);
}
#[inline]
unsafe extern "C" fn mp_cond_broadcast(mut cond: *mut mp_cond) -> ::core::ffi::c_int {
    if !cond.is_null() {
    } else {
        __assert_fail(
            b"cond\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/osdep/threads-posix.h\0" as *const u8 as *const ::core::ffi::c_char,
            167 as ::core::ffi::c_uint,
            b"int mp_cond_broadcast(mp_cond *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return pthread_cond_broadcast(&raw mut (*cond).cond);
}
#[inline]
unsafe extern "C" fn mp_cond_signal(mut cond: *mut mp_cond) -> ::core::ffi::c_int {
    if !cond.is_null() {
    } else {
        __assert_fail(
            b"cond\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/osdep/threads-posix.h\0" as *const u8 as *const ::core::ffi::c_char,
            173 as ::core::ffi::c_uint,
            b"int mp_cond_signal(mp_cond *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return pthread_cond_signal(&raw mut (*cond).cond);
}
#[inline]
unsafe extern "C" fn mp_cond_wait(
    mut cond: *mut mp_cond,
    mut mutex: *mut mp_mutex,
) -> ::core::ffi::c_int {
    if !cond.is_null() {
    } else {
        __assert_fail(
            b"cond\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/osdep/threads-posix.h\0" as *const u8 as *const ::core::ffi::c_char,
            179 as ::core::ffi::c_uint,
            b"int mp_cond_wait(mp_cond *, mp_mutex *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    return pthread_cond_wait(&raw mut (*cond).cond, mutex as *mut pthread_mutex_t);
}
#[inline]
unsafe extern "C" fn mp_cond_timedwait(
    mut cond: *mut mp_cond,
    mut mutex: *mut mp_mutex,
    mut timeout: int64_t,
) -> ::core::ffi::c_int {
    if !cond.is_null() {
    } else {
        __assert_fail(
            b"cond\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/osdep/threads-posix.h\0" as *const u8 as *const ::core::ffi::c_char,
            185 as ::core::ffi::c_uint,
            b"int mp_cond_timedwait(mp_cond *, mp_mutex *, int64_t)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    timeout = if 0 as int64_t > timeout {
        0 as int64_t
    } else {
        timeout
    };
    if timeout
        > (1000 as ::core::ffi::c_int
            * 24 as ::core::ffi::c_int
            * 60 as ::core::ffi::c_int
            * 60 as ::core::ffi::c_int) as int64_t
            * 1000000000 as int64_t
    {
        return pthread_cond_wait(&raw mut (*cond).cond, mutex as *mut pthread_mutex_t);
    }
    let mut ts: timespec = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    clock_gettime((*cond).clk_id, &raw mut ts);
    ts.tv_sec += (timeout / (1 as int64_t * 1000000000 as int64_t)) as ::core::ffi::c_long;
    ts.tv_nsec += (timeout % (1 as int64_t * 1000000000 as int64_t)) as ::core::ffi::c_long;
    if ts.tv_nsec >= 1 as ::core::ffi::c_long * 1000000000 as ::core::ffi::c_long {
        ts.tv_nsec -= 1 as ::core::ffi::c_long * 1000000000 as ::core::ffi::c_long;
        ts.tv_sec += 1;
    }
    return pthread_cond_timedwait(
        &raw mut (*cond).cond,
        mutex as *mut pthread_mutex_t,
        &raw mut ts,
    );
}
#[inline]
unsafe extern "C" fn mp_cond_timedwait_until(
    mut cond: *mut mp_cond,
    mut mutex: *mut mp_mutex,
    mut until: int64_t,
) -> ::core::ffi::c_int {
    return mp_cond_timedwait(cond, mutex, until - mp_time_ns());
}
#[no_mangle]
pub unsafe extern "C" fn mp_waiter_wait(mut waiter: *mut mp_waiter) -> uintptr_t {
    pthread_mutex_lock(&raw mut (*waiter).lock);
    while !(*waiter).done {
        mp_cond_wait(&raw mut (*waiter).wakeup, &raw mut (*waiter).lock);
    }
    pthread_mutex_unlock(&raw mut (*waiter).lock);
    let mut ret: uintptr_t = (*waiter).value;
    pthread_mutex_destroy(&raw mut (*waiter).lock);
    mp_cond_destroy(&raw mut (*waiter).wakeup);
    memset(
        waiter as *mut ::core::ffi::c_void,
        0xca as ::core::ffi::c_int,
        ::core::mem::size_of::<mp_waiter>() as size_t,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn mp_waiter_wakeup(mut waiter: *mut mp_waiter, mut value: uintptr_t) {
    pthread_mutex_lock(&raw mut (*waiter).lock);
    if !(*waiter).done {
    } else {
        __assert_fail(
            b"!waiter->done\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/thread_tools.c\0" as *const u8 as *const ::core::ffi::c_char,
            63 as ::core::ffi::c_uint,
            b"void mp_waiter_wakeup(struct mp_waiter *, uintptr_t)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*waiter).done = true_0 != 0;
    (*waiter).value = value;
    mp_cond_signal(&raw mut (*waiter).wakeup);
    pthread_mutex_unlock(&raw mut (*waiter).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_waiter_poll(mut waiter: *mut mp_waiter) -> bool {
    pthread_mutex_lock(&raw mut (*waiter).lock);
    let mut r: bool = (*waiter).done;
    pthread_mutex_unlock(&raw mut (*waiter).lock);
    return r;
}
unsafe extern "C" fn cancel_destroy(mut p: *mut ::core::ffi::c_void) {
    let mut c: *mut mp_cancel = p as *mut mp_cancel;
    if (*c).slaves.head.is_null() {
    } else {
        __assert_fail(
            b"!c->slaves.head\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/thread_tools.c\0" as *const u8 as *const ::core::ffi::c_char,
            105 as ::core::ffi::c_uint,
            b"void cancel_destroy(void *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    mp_cancel_set_parent(c, ::core::ptr::null_mut::<mp_cancel>());
    if (*c).wakeup_pipe[0 as ::core::ffi::c_int as usize] >= 0 as ::core::ffi::c_int {
        close((*c).wakeup_pipe[0 as ::core::ffi::c_int as usize]);
        close((*c).wakeup_pipe[1 as ::core::ffi::c_int as usize]);
    }
    pthread_mutex_destroy(&raw mut (*c).lock);
    mp_cond_destroy(&raw mut (*c).wakeup);
}
#[no_mangle]
pub unsafe extern "C" fn mp_cancel_new(mut talloc_ctx: *mut ::core::ffi::c_void) -> *mut mp_cancel {
    let mut c: *mut mp_cancel = ta_oom_p(ta_dbg_set_loc(
        ta_alloc_size(talloc_ctx, ::core::mem::size_of::<mp_cancel>() as size_t),
        b"../.mpv/misc/thread_tools.c:125\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut mp_cancel as *mut ::core::ffi::c_void)
        as *mut mp_cancel;
    ta_set_destructor(
        c as *mut ::core::ffi::c_void,
        Some(cancel_destroy as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
    *c = mp_cancel {
        lock: pthread_mutex_t {
            __data: __pthread_mutex_s {
                __lock: 0,
                __count: 0,
                __owner: 0,
                __nusers: 0,
                __kind: 0,
                __spins: 0,
                __elision: 0,
                __list: __pthread_internal_list {
                    __prev: ::core::ptr::null_mut::<__pthread_internal_list>(),
                    __next: ::core::ptr::null_mut::<__pthread_internal_list>(),
                },
            },
        },
        wakeup: mp_cond {
            cond: pthread_cond_t {
                __data: __pthread_cond_s {
                    __wseq: __atomic_wide_counter { __value64: 0 },
                    __g1_start: __atomic_wide_counter { __value64: 0 },
                    __g_refs: [0; 2],
                    __g_size: [0; 2],
                    __g1_orig_size: 0,
                    __wrefs: 0,
                    __g_signals: [0; 2],
                },
            },
            clk_id: 0,
        },
        triggered: false_0 != 0,
        cb: None,
        cb_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        wakeup_pipe: [-(1 as ::core::ffi::c_int), -(1 as ::core::ffi::c_int)],
        win32_event: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        slaves: C2RustUnnamed_2 {
            head: ::core::ptr::null_mut::<mp_cancel>(),
            tail: ::core::ptr::null_mut::<mp_cancel>(),
        },
        parent: ::core::ptr::null_mut::<mp_cancel>(),
        siblings: C2RustUnnamed_1 {
            next: ::core::ptr::null_mut::<mp_cancel>(),
            prev: ::core::ptr::null_mut::<mp_cancel>(),
        },
    };
    mp_mutex_init(&raw mut (*c).lock);
    mp_cond_init(&raw mut (*c).wakeup);
    return c;
}
unsafe extern "C" fn trigger_locked(mut c: *mut mp_cancel) {
    ::core::intrinsics::atomic_store_seqcst(&raw mut (*c).triggered as *mut u8, (1 as ::core::ffi::c_int != 0) as u8);
    mp_cond_broadcast(&raw mut (*c).wakeup);
    if (*c).cb.is_some() {
        (*c).cb.expect("non-null function pointer")((*c).cb_ctx);
    }
    let mut sub: *mut mp_cancel = (*c).slaves.head;
    while !sub.is_null() {
        mp_cancel_trigger(sub);
        sub = (*sub).siblings.next;
    }
    if (*c).wakeup_pipe[1 as ::core::ffi::c_int as usize] >= 0 as ::core::ffi::c_int {
        let mut fresh0: ::core::ffi::c_char = 0 as ::core::ffi::c_int as ::core::ffi::c_char;
        write(
            (*c).wakeup_pipe[1 as ::core::ffi::c_int as usize],
            &raw mut fresh0 as *const ::core::ffi::c_void,
            1 as size_t,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn mp_cancel_trigger(mut c: *mut mp_cancel) {
    pthread_mutex_lock(&raw mut (*c).lock);
    trigger_locked(c);
    pthread_mutex_unlock(&raw mut (*c).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_cancel_reset(mut c: *mut mp_cancel) {
    pthread_mutex_lock(&raw mut (*c).lock);
    ::core::intrinsics::atomic_store_seqcst(&raw mut (*c).triggered as *mut u8, (0 as ::core::ffi::c_int != 0) as u8);
    if (*c).wakeup_pipe[0 as ::core::ffi::c_int as usize] >= 0 as ::core::ffi::c_int {
        loop {
            let mut fresh1: [::core::ffi::c_char; 256] = [
                0 as ::core::ffi::c_int as ::core::ffi::c_char,
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
                0,
                0,
                0,
            ];
            let mut r: ::core::ffi::c_int = read(
                (*c).wakeup_pipe[0 as ::core::ffi::c_int as usize],
                &raw mut fresh1 as *mut ::core::ffi::c_void,
                256 as size_t,
            ) as ::core::ffi::c_int;
            if r <= 0 as ::core::ffi::c_int
                && !(r < 0 as ::core::ffi::c_int && *__errno_location() == EINTR)
            {
                break;
            }
        }
    }
    pthread_mutex_unlock(&raw mut (*c).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_cancel_test(mut c: *mut mp_cancel) -> bool {
    return if !c.is_null() {
        ::core::intrinsics::atomic_load_relaxed(&raw mut (*c).triggered as *mut u8) as ::core::ffi::c_int
    } else {
        false_0
    } != 0;
}
#[no_mangle]
pub unsafe extern "C" fn mp_cancel_wait(
    mut c: *mut mp_cancel,
    mut timeout: ::core::ffi::c_double,
) -> bool {
    let mut wait_until: int64_t = mp_time_ns_add(mp_time_ns(), timeout);
    pthread_mutex_lock(&raw mut (*c).lock);
    while !mp_cancel_test(c) {
        if mp_cond_timedwait_until(&raw mut (*c).wakeup, &raw mut (*c).lock, wait_until) != 0 {
            break;
        }
    }
    pthread_mutex_unlock(&raw mut (*c).lock);
    return mp_cancel_test(c);
}
unsafe extern "C" fn retrigger_locked(mut c: *mut mp_cancel) {
    if mp_cancel_test(c) {
        trigger_locked(c);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mp_cancel_set_cb(
    mut c: *mut mp_cancel,
    mut cb: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    pthread_mutex_lock(&raw mut (*c).lock);
    (*c).cb = cb;
    (*c).cb_ctx = ctx;
    retrigger_locked(c);
    pthread_mutex_unlock(&raw mut (*c).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_cancel_set_parent(
    mut slave: *mut mp_cancel,
    mut parent: *mut mp_cancel,
) {
    if (*slave).parent == parent {
        return;
    }
    if !(*slave).parent.is_null() {
        pthread_mutex_lock(&raw mut (*(*slave).parent).lock);
        if !(*slave).siblings.prev.is_null() {
            (*(*slave).siblings.prev).siblings.next = (*slave).siblings.next;
        } else {
            (*(*slave).parent).slaves.head = (*slave).siblings.next;
        }
        if !(*slave).siblings.next.is_null() {
            (*(*slave).siblings.next).siblings.prev = (*slave).siblings.prev;
        } else {
            (*(*slave).parent).slaves.tail = (*slave).siblings.prev;
        }
        pthread_mutex_unlock(&raw mut (*(*slave).parent).lock);
    }
    (*slave).parent = parent;
    if !(*slave).parent.is_null() {
        pthread_mutex_lock(&raw mut (*(*slave).parent).lock);
        (*slave).siblings.prev = (*(*slave).parent).slaves.tail;
        (*slave).siblings.next = ::core::ptr::null_mut::<mp_cancel>();
        if !(*slave).siblings.prev.is_null() {
            (*(*slave).siblings.prev).siblings.next = slave;
        } else {
            (*(*slave).parent).slaves.head = slave;
        }
        if !(*slave).siblings.next.is_null() {
            (*(*slave).siblings.next).siblings.prev = slave;
        } else {
            (*(*slave).parent).slaves.tail = slave;
        }
        retrigger_locked((*slave).parent);
        pthread_mutex_unlock(&raw mut (*(*slave).parent).lock);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mp_cancel_get_fd(mut c: *mut mp_cancel) -> ::core::ffi::c_int {
    pthread_mutex_lock(&raw mut (*c).lock);
    if (*c).wakeup_pipe[0 as ::core::ffi::c_int as usize] < 0 as ::core::ffi::c_int {
        mp_make_wakeup_pipe(&raw mut (*c).wakeup_pipe as *mut ::core::ffi::c_int);
        retrigger_locked(c);
    }
    pthread_mutex_unlock(&raw mut (*c).lock);
    return (*c).wakeup_pipe[0 as ::core::ffi::c_int as usize];
}
pub const __ATOMIC_SEQ_CST: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
