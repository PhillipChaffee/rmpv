extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn abort() -> !;
    fn ta_alloc_size(ta_parent: *mut ::core::ffi::c_void, size: size_t)
        -> *mut ::core::ffi::c_void;
    fn ta_free(ptr: *mut ::core::ffi::c_void);
    fn ta_set_destructor(
        ptr: *mut ::core::ffi::c_void,
        destructor: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    );
    fn ta_steal_(
        ta_parent: *mut ::core::ffi::c_void,
        ptr: *mut ::core::ffi::c_void,
    ) -> *mut ::core::ffi::c_void;
    fn ta_dbg_set_loc(
        ptr: *mut ::core::ffi::c_void,
        name: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    fn clock_gettime(__clock_id: clockid_t, __tp: *mut timespec) -> ::core::ffi::c_int;
    fn pthread_self() -> pthread_t;
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
    fn mp_time_ns() -> int64_t;
    fn mp_time_ns_add(time_ns: int64_t, timeout_sec: ::core::ffi::c_double) -> int64_t;
}
pub type size_t = usize;
pub type __int64_t = i64;
pub type __time_t = ::core::ffi::c_long;
pub type __clockid_t = ::core::ffi::c_int;
pub type __syscall_slong_t = ::core::ffi::c_long;
pub type clockid_t = __clockid_t;
pub type int64_t = __int64_t;
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
pub type pthread_t = ::core::ffi::c_ulong;
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
pub type mp_thread_id = pthread_t;
pub type mp_dispatch_fn = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_dispatch_queue {
    pub head: *mut mp_dispatch_item,
    pub tail: *mut mp_dispatch_item,
    pub lock: mp_mutex,
    pub cond: mp_cond,
    pub wakeup_fn: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub wakeup_ctx: *mut ::core::ffi::c_void,
    pub onlock_fn: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub onlock_ctx: *mut ::core::ffi::c_void,
    pub wait: int64_t,
    pub interrupted: bool,
    pub in_process: bool,
    pub in_process_thread_id: mp_thread_id,
    pub locked: bool,
    pub lock_requests: size_t,
    pub locked_explicit: bool,
    pub locked_explicit_thread_id: mp_thread_id,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_dispatch_item {
    pub fn_0: mp_dispatch_fn,
    pub fn_data: *mut ::core::ffi::c_void,
    pub asynchronous: bool,
    pub mergeable: bool,
    pub completed: bool,
    pub next: *mut mp_dispatch_item,
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
unsafe extern "C" fn queue_dtor(mut p: *mut ::core::ffi::c_void) {
    let mut queue: *mut mp_dispatch_queue = p as *mut mp_dispatch_queue;
    if (*queue).head.is_null() {
    } else {
        __assert_fail(
            b"!queue->head\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            66 as ::core::ffi::c_uint,
            b"void queue_dtor(void *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if !(*queue).in_process {
    } else {
        __assert_fail(
            b"!queue->in_process\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            67 as ::core::ffi::c_uint,
            b"void queue_dtor(void *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if (*queue).lock_requests == 0 {
    } else {
        __assert_fail(
            b"!queue->lock_requests\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            68 as ::core::ffi::c_uint,
            b"void queue_dtor(void *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if !(*queue).locked {
    } else {
        __assert_fail(
            b"!queue->locked\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            69 as ::core::ffi::c_uint,
            b"void queue_dtor(void *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    mp_cond_destroy(&raw mut (*queue).cond);
    pthread_mutex_destroy(&raw mut (*queue).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_create(
    mut ta_parent: *mut ::core::ffi::c_void,
) -> *mut mp_dispatch_queue {
    let mut queue: *mut mp_dispatch_queue = ta_oom_p(ta_dbg_set_loc(
        ta_alloc_size(
            ta_parent,
            ::core::mem::size_of::<mp_dispatch_queue>() as size_t,
        ),
        b"../.mpv/misc/dispatch.c:85\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut mp_dispatch_queue
        as *mut ::core::ffi::c_void)
        as *mut mp_dispatch_queue;
    *queue = mp_dispatch_queue {
        head: ::core::ptr::null_mut::<mp_dispatch_item>(),
        tail: ::core::ptr::null_mut::<mp_dispatch_item>(),
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
        cond: mp_cond {
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
        wakeup_fn: None,
        wakeup_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        onlock_fn: None,
        onlock_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        wait: 0,
        interrupted: false,
        in_process: false,
        in_process_thread_id: 0,
        locked: false,
        lock_requests: 0,
        locked_explicit: false,
        locked_explicit_thread_id: 0,
    };
    ta_set_destructor(
        queue as *mut ::core::ffi::c_void,
        Some(queue_dtor as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
    mp_mutex_init(&raw mut (*queue).lock);
    mp_cond_init(&raw mut (*queue).cond);
    return queue;
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_set_wakeup_fn(
    mut queue: *mut mp_dispatch_queue,
    mut wakeup_fn: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    mut wakeup_ctx: *mut ::core::ffi::c_void,
) {
    (*queue).wakeup_fn = wakeup_fn;
    (*queue).wakeup_ctx = wakeup_ctx;
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_set_onlock_fn(
    mut queue: *mut mp_dispatch_queue,
    mut onlock_fn: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    mut onlock_ctx: *mut ::core::ffi::c_void,
) {
    (*queue).onlock_fn = onlock_fn;
    (*queue).onlock_ctx = onlock_ctx;
}
unsafe extern "C" fn mp_dispatch_append(
    mut queue: *mut mp_dispatch_queue,
    mut item: *mut mp_dispatch_item,
) {
    pthread_mutex_lock(&raw mut (*queue).lock);
    if (*item).mergeable {
        let mut cur: *mut mp_dispatch_item = (*queue).head as *mut mp_dispatch_item;
        while !cur.is_null() {
            if (*cur).mergeable as ::core::ffi::c_int != 0
                && (*cur).fn_0 == (*item).fn_0
                && (*cur).fn_data == (*item).fn_data
            {
                ta_free(item as *mut ::core::ffi::c_void);
                pthread_mutex_unlock(&raw mut (*queue).lock);
                return;
            }
            cur = (*cur).next;
        }
    }
    if !(*queue).tail.is_null() {
        (*(*queue).tail).next = item;
    } else {
        (*queue).head = item as *mut mp_dispatch_item;
    }
    (*queue).tail = item as *mut mp_dispatch_item;
    mp_cond_broadcast(&raw mut (*queue).cond);
    if (*queue).wakeup_fn.is_none() {
        (*queue).interrupted = true_0 != 0;
    }
    pthread_mutex_unlock(&raw mut (*queue).lock);
    if (*queue).wakeup_fn.is_some() {
        (*queue).wakeup_fn.expect("non-null function pointer")((*queue).wakeup_ctx);
    }
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_enqueue(
    mut queue: *mut mp_dispatch_queue,
    mut fn_0: mp_dispatch_fn,
    mut fn_data: *mut ::core::ffi::c_void,
) {
    let mut item: *mut mp_dispatch_item = ta_oom_p(ta_dbg_set_loc(
        ta_alloc_size(
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::mem::size_of::<mp_dispatch_item>() as size_t,
        ),
        b"../.mpv/misc/dispatch.c:171\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut mp_dispatch_item
        as *mut ::core::ffi::c_void)
        as *mut mp_dispatch_item;
    *item = mp_dispatch_item {
        fn_0: fn_0,
        fn_data: fn_data,
        asynchronous: true_0 != 0,
        mergeable: false,
        completed: false,
        next: ::core::ptr::null_mut::<mp_dispatch_item>(),
    };
    mp_dispatch_append(queue, item);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_enqueue_autofree(
    mut queue: *mut mp_dispatch_queue,
    mut fn_0: mp_dispatch_fn,
    mut fn_data: *mut ::core::ffi::c_void,
) {
    let mut item: *mut mp_dispatch_item = ta_oom_p(ta_dbg_set_loc(
        ta_alloc_size(
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::mem::size_of::<mp_dispatch_item>() as size_t,
        ),
        b"../.mpv/misc/dispatch.c:187\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut mp_dispatch_item
        as *mut ::core::ffi::c_void)
        as *mut mp_dispatch_item;
    *item = mp_dispatch_item {
        fn_0: fn_0,
        fn_data: ta_steal_(item as *mut ::core::ffi::c_void, fn_data),
        asynchronous: true_0 != 0,
        mergeable: false,
        completed: false,
        next: ::core::ptr::null_mut::<mp_dispatch_item>(),
    };
    mp_dispatch_append(queue, item);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_enqueue_notify(
    mut queue: *mut mp_dispatch_queue,
    mut fn_0: mp_dispatch_fn,
    mut fn_data: *mut ::core::ffi::c_void,
) {
    let mut item: *mut mp_dispatch_item = ta_oom_p(ta_dbg_set_loc(
        ta_alloc_size(
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::mem::size_of::<mp_dispatch_item>() as size_t,
        ),
        b"../.mpv/misc/dispatch.c:200\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut mp_dispatch_item
        as *mut ::core::ffi::c_void)
        as *mut mp_dispatch_item;
    *item = mp_dispatch_item {
        fn_0: fn_0,
        fn_data: fn_data,
        asynchronous: true_0 != 0,
        mergeable: true_0 != 0,
        completed: false,
        next: ::core::ptr::null_mut::<mp_dispatch_item>(),
    };
    mp_dispatch_append(queue, item);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_cancel_fn(
    mut queue: *mut mp_dispatch_queue,
    mut fn_0: mp_dispatch_fn,
    mut fn_data: *mut ::core::ffi::c_void,
) {
    pthread_mutex_lock(&raw mut (*queue).lock);
    let mut pcur: *mut *mut mp_dispatch_item = &raw mut (*queue).head;
    (*queue).tail = ::core::ptr::null_mut::<mp_dispatch_item>();
    while !(*pcur).is_null() {
        let mut cur: *mut mp_dispatch_item = *pcur;
        if (*cur).fn_0 == fn_0 && (*cur).fn_data == fn_data {
            *pcur = (*cur).next;
            ta_free(cur as *mut ::core::ffi::c_void);
        } else {
            (*queue).tail = cur as *mut mp_dispatch_item;
            pcur = &raw mut (*cur).next;
        }
    }
    pthread_mutex_unlock(&raw mut (*queue).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_run(
    mut queue: *mut mp_dispatch_queue,
    mut fn_0: mp_dispatch_fn,
    mut fn_data: *mut ::core::ffi::c_void,
) {
    let mut item: mp_dispatch_item = mp_dispatch_item {
        fn_0: fn_0,
        fn_data: fn_data,
        asynchronous: false,
        mergeable: false,
        completed: false,
        next: ::core::ptr::null_mut::<mp_dispatch_item>(),
    };
    mp_dispatch_append(queue, &raw mut item);
    pthread_mutex_lock(&raw mut (*queue).lock);
    while !item.completed {
        mp_cond_wait(&raw mut (*queue).cond, &raw mut (*queue).lock);
    }
    pthread_mutex_unlock(&raw mut (*queue).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_queue_process(
    mut queue: *mut mp_dispatch_queue,
    mut timeout: ::core::ffi::c_double,
) {
    pthread_mutex_lock(&raw mut (*queue).lock);
    (*queue).wait = if timeout > 0 as ::core::ffi::c_int as ::core::ffi::c_double {
        mp_time_ns_add(mp_time_ns(), timeout)
    } else {
        0 as int64_t
    };
    if !(*queue).in_process {
    } else {
        __assert_fail(
            b"!queue->in_process\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            276 as ::core::ffi::c_uint,
            b"void mp_dispatch_queue_process(struct mp_dispatch_queue *, double)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*queue).in_process = true_0 != 0;
    (*queue).in_process_thread_id = pthread_self() as mp_thread_id;
    if (*queue).lock_requests != 0 {
        mp_cond_broadcast(&raw mut (*queue).cond);
    }
    loop {
        if (*queue).lock_requests != 0 {
            mp_cond_wait(&raw mut (*queue).cond, &raw mut (*queue).lock);
        } else if !(*queue).head.is_null() {
            let mut item: *mut mp_dispatch_item = (*queue).head as *mut mp_dispatch_item;
            (*queue).head = (*item).next as *mut mp_dispatch_item;
            if (*queue).head.is_null() {
                (*queue).tail = ::core::ptr::null_mut::<mp_dispatch_item>();
            }
            (*item).next = ::core::ptr::null_mut::<mp_dispatch_item>();
            if !(*queue).locked {
            } else {
                __assert_fail(
                    b"!queue->locked\0" as *const u8 as *const ::core::ffi::c_char,
                    b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
                    296 as ::core::ffi::c_uint,
                    b"void mp_dispatch_queue_process(struct mp_dispatch_queue *, double)\0"
                        as *const u8 as *const ::core::ffi::c_char,
                );
            };
            (*queue).locked = true_0 != 0;
            pthread_mutex_unlock(&raw mut (*queue).lock);
            (*item).fn_0.expect("non-null function pointer")((*item).fn_data);
            pthread_mutex_lock(&raw mut (*queue).lock);
            if (*queue).locked as ::core::ffi::c_int != 0 {
            } else {
                __assert_fail(
                    b"queue->locked\0" as *const u8 as *const ::core::ffi::c_char,
                    b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
                    303 as ::core::ffi::c_uint,
                    b"void mp_dispatch_queue_process(struct mp_dispatch_queue *, double)\0"
                        as *const u8 as *const ::core::ffi::c_char,
                );
            };
            (*queue).locked = false_0 != 0;
            mp_cond_broadcast(&raw mut (*queue).cond);
            if (*item).asynchronous {
                ta_free(item as *mut ::core::ffi::c_void);
            } else {
                (*item).completed = true_0 != 0;
            }
        } else {
            if !((*queue).wait > 0 as int64_t && !(*queue).interrupted) {
                break;
            }
            if mp_cond_timedwait_until(
                &raw mut (*queue).cond,
                &raw mut (*queue).lock,
                (*queue).wait,
            ) != 0
            {
                (*queue).wait = 0 as int64_t;
            }
        }
    }
    if !(*queue).locked {
    } else {
        __assert_fail(
            b"!queue->locked\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            319 as ::core::ffi::c_uint,
            b"void mp_dispatch_queue_process(struct mp_dispatch_queue *, double)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*queue).in_process = false_0 != 0;
    (*queue).interrupted = false_0 != 0;
    pthread_mutex_unlock(&raw mut (*queue).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_interrupt(mut queue: *mut mp_dispatch_queue) {
    pthread_mutex_lock(&raw mut (*queue).lock);
    (*queue).interrupted = true_0 != 0;
    mp_cond_broadcast(&raw mut (*queue).cond);
    pthread_mutex_unlock(&raw mut (*queue).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_adjust_timeout(
    mut queue: *mut mp_dispatch_queue,
    mut until: int64_t,
) {
    pthread_mutex_lock(&raw mut (*queue).lock);
    if (*queue).in_process as ::core::ffi::c_int != 0 && (*queue).wait > until {
        (*queue).wait = until;
        mp_cond_broadcast(&raw mut (*queue).cond);
    }
    pthread_mutex_unlock(&raw mut (*queue).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_lock(mut queue: *mut mp_dispatch_queue) {
    pthread_mutex_lock(&raw mut (*queue).lock);
    if (*queue).in_process {
        if !((*queue).in_process_thread_id == pthread_self()) {
        } else {
            __assert_fail(
                b"!mp_thread_id_equal(queue->in_process_thread_id, mp_thread_current_id())\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
                369 as ::core::ffi::c_uint,
                b"void mp_dispatch_lock(struct mp_dispatch_queue *)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        };
    }
    if (*queue).locked_explicit {
        if !((*queue).locked_explicit_thread_id == pthread_self()) {
        } else {
            __assert_fail(
                b"!mp_thread_id_equal(queue->locked_explicit_thread_id, mp_thread_current_id())\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
                372 as ::core::ffi::c_uint,
                b"void mp_dispatch_lock(struct mp_dispatch_queue *)\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        };
    }
    (*queue).lock_requests = (*queue).lock_requests.wrapping_add(1 as size_t);
    if (*queue).onlock_fn.is_some() {
        (*queue).onlock_fn.expect("non-null function pointer")((*queue).onlock_ctx);
    }
    while !(*queue).in_process {
        pthread_mutex_unlock(&raw mut (*queue).lock);
        if (*queue).wakeup_fn.is_some() {
            (*queue).wakeup_fn.expect("non-null function pointer")((*queue).wakeup_ctx);
        }
        pthread_mutex_lock(&raw mut (*queue).lock);
        if (*queue).in_process {
            break;
        }
        mp_cond_wait(&raw mut (*queue).cond, &raw mut (*queue).lock);
    }
    while !(*queue).in_process || (*queue).locked as ::core::ffi::c_int != 0 {
        mp_cond_wait(&raw mut (*queue).cond, &raw mut (*queue).lock);
    }
    if (*queue).lock_requests != 0 {
    } else {
        __assert_fail(
            b"queue->lock_requests\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            392 as ::core::ffi::c_uint,
            b"void mp_dispatch_lock(struct mp_dispatch_queue *)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    if !(*queue).locked {
    } else {
        __assert_fail(
            b"!queue->locked\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            393 as ::core::ffi::c_uint,
            b"void mp_dispatch_lock(struct mp_dispatch_queue *)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    if !(*queue).locked_explicit {
    } else {
        __assert_fail(
            b"!queue->locked_explicit\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            394 as ::core::ffi::c_uint,
            b"void mp_dispatch_lock(struct mp_dispatch_queue *)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*queue).locked = true_0 != 0;
    (*queue).locked_explicit = true_0 != 0;
    (*queue).locked_explicit_thread_id = pthread_self() as mp_thread_id;
    pthread_mutex_unlock(&raw mut (*queue).lock);
}
#[no_mangle]
pub unsafe extern "C" fn mp_dispatch_unlock(mut queue: *mut mp_dispatch_queue) {
    pthread_mutex_lock(&raw mut (*queue).lock);
    if (*queue).locked as ::core::ffi::c_int != 0 {
    } else {
        __assert_fail(
            b"queue->locked\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            405 as ::core::ffi::c_uint,
            b"void mp_dispatch_unlock(struct mp_dispatch_queue *)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    if (*queue).locked_explicit as ::core::ffi::c_int != 0 {
    } else {
        __assert_fail(
            b"queue->locked_explicit\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            407 as ::core::ffi::c_uint,
            b"void mp_dispatch_unlock(struct mp_dispatch_queue *)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    if (*queue).locked_explicit_thread_id == pthread_self() {
    } else {
        __assert_fail(
            b"mp_thread_id_equal(queue->locked_explicit_thread_id, mp_thread_current_id())\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/dispatch.c\0" as *const u8 as *const ::core::ffi::c_char,
            408 as ::core::ffi::c_uint,
            b"void mp_dispatch_unlock(struct mp_dispatch_queue *)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*queue).locked = false_0 != 0;
    (*queue).locked_explicit = false_0 != 0;
    (*queue).lock_requests = (*queue).lock_requests.wrapping_sub(1 as size_t);
    mp_cond_broadcast(&raw mut (*queue).cond);
    pthread_mutex_unlock(&raw mut (*queue).lock);
}
