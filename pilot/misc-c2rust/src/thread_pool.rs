extern "C" {
    fn abort() -> !;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memmove(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn strncpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn ta_zalloc_size(
        ta_parent: *mut ::core::ffi::c_void,
        size: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn ta_get_size(ptr: *mut ::core::ffi::c_void) -> size_t;
    fn ta_free(ptr: *mut ::core::ffi::c_void);
    fn ta_set_destructor(
        ptr: *mut ::core::ffi::c_void,
        destructor: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    );
    fn ta_calc_array_size(element_size: size_t, count: size_t) -> size_t;
    fn ta_calc_prealloc_elems(nextidx: size_t) -> size_t;
    fn ta_xrealloc_size(
        ta_parent: *mut ::core::ffi::c_void,
        ptr: *mut ::core::ffi::c_void,
        size: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn ta_dbg_set_loc(
        ptr: *mut ::core::ffi::c_void,
        name: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    fn clock_gettime(__clock_id: clockid_t, __tp: *mut timespec) -> ::core::ffi::c_int;
    fn pthread_create(
        __newthread: *mut pthread_t,
        __attr: *const pthread_attr_t,
        __start_routine: Option<
            unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void,
        >,
        __arg: *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn pthread_join(
        __th: pthread_t,
        __thread_return: *mut *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int;
    fn pthread_detach(__th: pthread_t) -> ::core::ffi::c_int;
    fn pthread_self() -> pthread_t;
    fn pthread_setname_np(
        __target_thread: pthread_t,
        __name: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
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
    fn mp_time_ns() -> int64_t;
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
pub union pthread_attr_t {
    pub __size: [::core::ffi::c_char; 56],
    pub __align: ::core::ffi::c_long,
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
pub type mp_thread = pthread_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_thread_pool {
    pub min_threads: ::core::ffi::c_int,
    pub max_threads: ::core::ffi::c_int,
    pub lock: mp_mutex,
    pub wakeup: mp_cond,
    pub threads: *mut mp_thread,
    pub num_threads: ::core::ffi::c_int,
    pub busy_threads: ::core::ffi::c_int,
    pub terminate: bool,
    pub work: *mut work,
    pub num_work: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct work {
    pub fn_0: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub fn_ctx: *mut ::core::ffi::c_void,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ERANGE: ::core::ffi::c_int = 34 as ::core::ffi::c_int;
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
#[inline]
unsafe extern "C" fn mp_thread_set_name(mut name: *const ::core::ffi::c_char) {
    if pthread_setname_np(pthread_self(), name) == ERANGE {
        let mut tname: [::core::ffi::c_char; 16] = [
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
        ];
        strncpy(
            &raw mut tname as *mut ::core::ffi::c_char,
            name,
            (::core::mem::size_of::<[::core::ffi::c_char; 16]>() as size_t)
                .wrapping_sub(1 as size_t),
        );
        pthread_setname_np(pthread_self(), &raw mut tname as *mut ::core::ffi::c_char);
    }
}
unsafe extern "C" fn worker_thread(mut arg: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void {
    let mut pool: *mut mp_thread_pool = arg as *mut mp_thread_pool;
    mp_thread_set_name(b"worker\0" as *const u8 as *const ::core::ffi::c_char);
    pthread_mutex_lock(&raw mut (*pool).lock);
    let mut destroy_deadline: int64_t = 0 as int64_t;
    let mut got_timeout: bool = false_0 != 0;
    loop {
        let mut work: work = work {
            fn_0: None,
            fn_ctx: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        if (*pool).num_work > 0 as ::core::ffi::c_int {
            work = *(*pool)
                .work
                .offset(((*pool).num_work - 1 as ::core::ffi::c_int) as isize);
            (*pool).num_work -= 1 as ::core::ffi::c_int;
        }
        if work.fn_0.is_none() {
            if got_timeout as ::core::ffi::c_int != 0
                || (*pool).terminate as ::core::ffi::c_int != 0
            {
                break;
            }
            if (*pool).num_threads > (*pool).min_threads {
                if destroy_deadline == 0 {
                    destroy_deadline = mp_time_ns() + 10 as int64_t * 1000000000 as int64_t;
                }
                if mp_cond_timedwait_until(
                    &raw mut (*pool).wakeup,
                    &raw mut (*pool).lock,
                    destroy_deadline,
                ) != 0
                {
                    got_timeout = (*pool).num_threads > (*pool).min_threads;
                }
            } else {
                mp_cond_wait(&raw mut (*pool).wakeup, &raw mut (*pool).lock);
            }
        } else {
            (*pool).busy_threads += 1 as ::core::ffi::c_int;
            pthread_mutex_unlock(&raw mut (*pool).lock);
            work.fn_0.expect("non-null function pointer")(work.fn_ctx);
            pthread_mutex_lock(&raw mut (*pool).lock);
            (*pool).busy_threads -= 1 as ::core::ffi::c_int;
            destroy_deadline = 0 as int64_t;
            got_timeout = false_0 != 0;
        }
    }
    if !(*pool).terminate {
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while n < (*pool).num_threads {
            if *(*pool).threads.offset(n as isize) == pthread_self() {
                pthread_detach(*(*pool).threads.offset(n as isize) as pthread_t);
                let mut at_: size_t = n as size_t;
                if at_ < (*pool).num_threads as size_t {
                } else {
                    __assert_fail(
                        b"at_ < (pool->num_threads)\0" as *const u8 as *const ::core::ffi::c_char,
                        b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
                        103 as ::core::ffi::c_uint,
                        b"void *worker_thread(void *)\0" as *const u8 as *const ::core::ffi::c_char,
                    );
                };
                memmove(
                    (*pool).threads.offset(at_ as isize) as *mut ::core::ffi::c_void,
                    (*pool)
                        .threads
                        .offset(at_ as isize)
                        .offset(1 as ::core::ffi::c_int as isize)
                        as *const ::core::ffi::c_void,
                    ((*pool).num_threads as size_t)
                        .wrapping_sub(at_)
                        .wrapping_sub(1 as size_t)
                        .wrapping_mul(::core::mem::size_of::<mp_thread>() as size_t),
                );
                (*pool).num_threads -= 1;
                pthread_mutex_unlock(&raw mut (*pool).lock);
                return NULL;
            }
            n += 1;
        }
        if (b"unreachable\0" as *const u8 as *const ::core::ffi::c_char).is_null() {
        } else {
            __assert_fail(
                b"!\"unreachable\"\0" as *const u8 as *const ::core::ffi::c_char,
                b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
                108 as ::core::ffi::c_uint,
                b"void *worker_thread(void *)\0" as *const u8 as *const ::core::ffi::c_char,
            );
        };
        abort();
    }
    pthread_mutex_unlock(&raw mut (*pool).lock);
    return NULL;
}
unsafe extern "C" fn thread_pool_dtor(mut ctx: *mut ::core::ffi::c_void) {
    let mut pool: *mut mp_thread_pool = ctx as *mut mp_thread_pool;
    pthread_mutex_lock(&raw mut (*pool).lock);
    (*pool).terminate = true_0 != 0;
    mp_cond_broadcast(&raw mut (*pool).wakeup);
    let mut threads: *mut mp_thread = (*pool).threads;
    let mut num_threads: ::core::ffi::c_int = (*pool).num_threads;
    (*pool).threads = ::core::ptr::null_mut::<mp_thread>();
    (*pool).num_threads = 0 as ::core::ffi::c_int;
    pthread_mutex_unlock(&raw mut (*pool).lock);
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n < num_threads {
        pthread_join(
            *threads.offset(n as isize) as pthread_t,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_void>(),
        );
        n += 1;
    }
    if (*pool).num_work == 0 as ::core::ffi::c_int {
    } else {
        __assert_fail(
            b"pool->num_work == 0\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
            136 as ::core::ffi::c_uint,
            b"void thread_pool_dtor(void *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    if (*pool).num_threads == 0 as ::core::ffi::c_int {
    } else {
        __assert_fail(
            b"pool->num_threads == 0\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
            137 as ::core::ffi::c_uint,
            b"void thread_pool_dtor(void *)\0" as *const u8 as *const ::core::ffi::c_char,
        );
    };
    mp_cond_destroy(&raw mut (*pool).wakeup);
    pthread_mutex_destroy(&raw mut (*pool).lock);
}
unsafe extern "C" fn add_thread(mut pool: *mut mp_thread_pool) -> bool {
    let mut thread: mp_thread = 0;
    if pthread_create(
        &raw mut thread,
        ::core::ptr::null::<pthread_attr_t>(),
        Some(
            worker_thread
                as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void,
        ),
        pool as *mut ::core::ffi::c_void,
    ) != 0 as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    let mut nextidx_: size_t = (*pool).num_threads as size_t;
    if nextidx_
        >= ta_get_size((*pool).threads as *mut ::core::ffi::c_void)
            .wrapping_div(::core::mem::size_of::<mp_thread>() as size_t)
    {
        (*pool).threads = ta_dbg_set_loc(
            ta_xrealloc_size(
                pool as *mut ::core::ffi::c_void,
                (*pool).threads as *mut ::core::ffi::c_void,
                ta_calc_array_size(
                    ::core::mem::size_of::<mp_thread>() as size_t,
                    ta_calc_prealloc_elems(nextidx_),
                ),
            ),
            b"../.mpv/misc/thread_pool.c:149\0" as *const u8 as *const ::core::ffi::c_char,
        ) as *mut mp_thread;
    }
    *(*pool).threads.offset((*pool).num_threads as isize) = thread;
    (*pool).num_threads += 1;
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn mp_thread_pool_create(
    mut ta_parent: *mut ::core::ffi::c_void,
    mut init_threads: ::core::ffi::c_int,
    mut min_threads: ::core::ffi::c_int,
    mut max_threads: ::core::ffi::c_int,
) -> *mut mp_thread_pool {
    if min_threads >= 0 as ::core::ffi::c_int {
    } else {
        __assert_fail(
            b"min_threads >= 0\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
            156 as ::core::ffi::c_uint,
            b"struct mp_thread_pool *mp_thread_pool_create(void *, int, int, int)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    if init_threads <= min_threads {
    } else {
        __assert_fail(
            b"init_threads <= min_threads\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
            157 as ::core::ffi::c_uint,
            b"struct mp_thread_pool *mp_thread_pool_create(void *, int, int, int)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    if max_threads > 0 as ::core::ffi::c_int && max_threads >= min_threads {
    } else {
        __assert_fail(
            b"max_threads > 0 && max_threads >= min_threads\0" as *const u8
                as *const ::core::ffi::c_char,
            b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
            158 as ::core::ffi::c_uint,
            b"struct mp_thread_pool *mp_thread_pool_create(void *, int, int, int)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    let mut pool: *mut mp_thread_pool = ta_oom_p(ta_dbg_set_loc(
        ta_zalloc_size(
            ta_parent,
            ::core::mem::size_of::<mp_thread_pool>() as size_t,
        ),
        b"../.mpv/misc/thread_pool.c:160\0" as *const u8 as *const ::core::ffi::c_char,
    ) as *mut mp_thread_pool
        as *mut ::core::ffi::c_void) as *mut mp_thread_pool;
    ta_set_destructor(
        pool as *mut ::core::ffi::c_void,
        Some(thread_pool_dtor as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    );
    mp_mutex_init(&raw mut (*pool).lock);
    mp_cond_init(&raw mut (*pool).wakeup);
    (*pool).min_threads = min_threads;
    (*pool).max_threads = max_threads;
    pthread_mutex_lock(&raw mut (*pool).lock);
    let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while n < init_threads {
        add_thread(pool);
        n += 1;
    }
    let mut ok: bool = (*pool).num_threads >= init_threads;
    pthread_mutex_unlock(&raw mut (*pool).lock);
    if !ok {
        ta_free(pool as *mut ::core::ffi::c_void);
        pool = ::core::ptr::null_mut::<mp_thread_pool>();
    }
    return pool;
}
unsafe extern "C" fn thread_pool_add(
    mut pool: *mut mp_thread_pool,
    mut fn_0: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    mut fn_ctx: *mut ::core::ffi::c_void,
    mut allow_queue: bool,
) -> bool {
    let mut ok: bool = true_0 != 0;
    if fn_0.is_some() {
    } else {
        __assert_fail(
            b"fn\0" as *const u8 as *const ::core::ffi::c_char,
            b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
            186 as ::core::ffi::c_uint,
            b"_Bool thread_pool_add(struct mp_thread_pool *, void (*)(void *), void *, _Bool)\0"
                as *const u8 as *const ::core::ffi::c_char,
        );
    };
    pthread_mutex_lock(&raw mut (*pool).lock);
    let mut work: work = work {
        fn_0: fn_0,
        fn_ctx: fn_ctx,
    };
    if (*pool).busy_threads + (*pool).num_work + 1 as ::core::ffi::c_int > (*pool).num_threads
        && (*pool).num_threads < (*pool).max_threads
    {
        if !add_thread(pool) {
            ok = allow_queue as ::core::ffi::c_int != 0
                && (*pool).num_threads > 0 as ::core::ffi::c_int;
        }
    }
    if ok {
        let mut at_: size_t = 0 as size_t;
        if at_ <= (*pool).num_work as size_t {
        } else {
            __assert_fail(
                b"at_ <= (pool->num_work)\0" as *const u8 as *const ::core::ffi::c_char,
                b"../.mpv/misc/thread_pool.c\0" as *const u8 as *const ::core::ffi::c_char,
                205 as ::core::ffi::c_uint,
                b"_Bool thread_pool_add(struct mp_thread_pool *, void (*)(void *), void *, _Bool)\0"
                    as *const u8 as *const ::core::ffi::c_char,
            );
        };
        let mut nextidx_: size_t = (*pool).num_work as size_t;
        if nextidx_
            >= ta_get_size((*pool).work as *mut ::core::ffi::c_void)
                .wrapping_div(::core::mem::size_of::<work>() as size_t)
        {
            (*pool).work = ta_dbg_set_loc(
                ta_xrealloc_size(
                    pool as *mut ::core::ffi::c_void,
                    (*pool).work as *mut ::core::ffi::c_void,
                    ta_calc_array_size(
                        ::core::mem::size_of::<work>() as size_t,
                        ta_calc_prealloc_elems(nextidx_),
                    ),
                ),
                b"../.mpv/misc/thread_pool.c:205\0" as *const u8 as *const ::core::ffi::c_char,
            ) as *mut work;
        }
        memmove(
            (*pool)
                .work
                .offset(at_ as isize)
                .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            (*pool).work.offset(at_ as isize) as *const ::core::ffi::c_void,
            ((*pool).num_work as size_t)
                .wrapping_sub(at_)
                .wrapping_mul(::core::mem::size_of::<work>() as size_t),
        );
        (*pool).num_work += 1;
        *(*pool).work.offset(at_ as isize) = work;
        mp_cond_signal(&raw mut (*pool).wakeup);
    }
    pthread_mutex_unlock(&raw mut (*pool).lock);
    return ok;
}
#[no_mangle]
pub unsafe extern "C" fn mp_thread_pool_queue(
    mut pool: *mut mp_thread_pool,
    mut fn_0: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    mut fn_ctx: *mut ::core::ffi::c_void,
) -> bool {
    return thread_pool_add(pool, fn_0, fn_ctx, true_0 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn mp_thread_pool_run(
    mut pool: *mut mp_thread_pool,
    mut fn_0: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    mut fn_ctx: *mut ::core::ffi::c_void,
) -> bool {
    return thread_pool_add(pool, fn_0, fn_ctx, false_0 != 0);
}
