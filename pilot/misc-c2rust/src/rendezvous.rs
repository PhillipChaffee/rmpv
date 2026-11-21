extern "C" {
    fn pthread_mutex_lock(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn pthread_mutex_unlock(__mutex: *mut pthread_mutex_t) -> ::core::ffi::c_int;
    fn pthread_cond_broadcast(__cond: *mut pthread_cond_t) -> ::core::ffi::c_int;
    fn pthread_cond_wait(
        __cond: *mut pthread_cond_t,
        __mutex: *mut pthread_mutex_t,
    ) -> ::core::ffi::c_int;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
}
pub type __clockid_t = ::core::ffi::c_int;
pub type intptr_t = isize;
pub type pthread_mutex_t = mp_static_mutex;
#[derive(Copy, Clone)]
#[repr(C)]
pub union mp_static_mutex {
    pub __data: __pthread_mutex_s,
    pub __size: [::core::ffi::c_char; 40],
    pub __align: ::core::ffi::c_long,
}
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
pub type __pthread_list_t = __pthread_internal_list;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}
pub const PTHREAD_MUTEX_TIMED_NP: C2RustUnnamed_0 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mp_cond {
    pub cond: pthread_cond_t,
    pub clk_id: clockid_t,
}
pub type clockid_t = __clockid_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub union pthread_cond_t {
    pub __data: __pthread_cond_s,
    pub __size: [::core::ffi::c_char; 48],
    pub __align: ::core::ffi::c_longlong,
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
pub type mp_mutex = pthread_mutex_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct waiter {
    pub tag: *mut ::core::ffi::c_void,
    pub next: *mut waiter,
    pub value: *mut intptr_t,
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
pub const CLOCK_REALTIME: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
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
static mut lock: mp_static_mutex = mp_static_mutex {
    __data: __pthread_mutex_s {
        __lock: 0 as ::core::ffi::c_int,
        __count: 0 as ::core::ffi::c_uint,
        __owner: 0 as ::core::ffi::c_int,
        __nusers: 0 as ::core::ffi::c_uint,
        __kind: PTHREAD_MUTEX_TIMED_NP as ::core::ffi::c_int,
        __spins: 0 as ::core::ffi::c_short,
        __elision: 0 as ::core::ffi::c_short,
        __list: __pthread_internal_list {
            __prev: ::core::ptr::null::<__pthread_internal_list>() as *mut __pthread_internal_list,
            __next: ::core::ptr::null::<__pthread_internal_list>() as *mut __pthread_internal_list,
        },
    },
};
static mut wakeup: mp_cond = mp_cond {
    cond: pthread_cond_t {
        __data: __pthread_cond_s {
            __wseq: __atomic_wide_counter {
                __value64: 0 as ::core::ffi::c_int as ::core::ffi::c_ulonglong,
            },
            __g1_start: __atomic_wide_counter {
                __value64: 0 as ::core::ffi::c_int as ::core::ffi::c_ulonglong,
            },
            __g_refs: [
                0 as ::core::ffi::c_int as ::core::ffi::c_uint,
                0 as ::core::ffi::c_int as ::core::ffi::c_uint,
            ],
            __g_size: [
                0 as ::core::ffi::c_int as ::core::ffi::c_uint,
                0 as ::core::ffi::c_int as ::core::ffi::c_uint,
            ],
            __g1_orig_size: 0 as ::core::ffi::c_uint,
            __wrefs: 0 as ::core::ffi::c_uint,
            __g_signals: [
                0 as ::core::ffi::c_int as ::core::ffi::c_uint,
                0 as ::core::ffi::c_int as ::core::ffi::c_uint,
            ],
        },
    },
    clk_id: CLOCK_REALTIME,
};
static mut waiters: *mut waiter = ::core::ptr::null::<waiter>() as *mut waiter;
#[no_mangle]
pub unsafe extern "C" fn mp_rendezvous(
    mut tag: *mut ::core::ffi::c_void,
    mut value: intptr_t,
) -> intptr_t {
    let mut current_block: u64;
    let mut wait: waiter = waiter {
        tag: tag,
        next: ::core::ptr::null_mut::<waiter>(),
        value: &raw mut value,
    };
    pthread_mutex_lock(&raw mut lock);
    let mut prev: *mut *mut waiter = &raw mut waiters;
    loop {
        if (*prev).is_null() {
            current_block = 14523784380283086299;
            break;
        }
        if (**prev).tag == tag {
            let mut tmp: intptr_t = *(**prev).value;
            *(**prev).value = value;
            value = tmp;
            (**prev).value = ::core::ptr::null_mut::<intptr_t>();
            *prev = (**prev).next;
            mp_cond_broadcast(&raw mut wakeup);
            current_block = 1583998504835368619;
            break;
        } else {
            prev = &raw mut (**prev).next;
        }
    }
    match current_block {
        14523784380283086299 => {
            *prev = &raw mut wait;
            while !wait.value.is_null() {
                mp_cond_wait(&raw mut wakeup, &raw mut lock);
            }
        }
        _ => {}
    }
    pthread_mutex_unlock(&raw mut lock);
    return value;
}
