extern "C" {
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn abort() -> !;
    fn ta_get_size(ptr: *mut ::core::ffi::c_void) -> size_t;
    fn ta_free(ptr: *mut ::core::ffi::c_void);
    fn ta_calc_array_size(element_size: size_t, count: size_t) -> size_t;
    fn ta_calc_prealloc_elems(nextidx: size_t) -> size_t;
    fn ta_new_context(ta_parent: *mut ::core::ffi::c_void) -> *mut ::core::ffi::c_void;
    fn ta_xrealloc_size(
        ta_parent: *mut ::core::ffi::c_void,
        ptr: *mut ::core::ffi::c_void,
        size: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn ta_dbg_set_loc(
        ptr: *mut ::core::ffi::c_void,
        name: *const ::core::ffi::c_char,
    ) -> *mut ::core::ffi::c_void;
    fn bstrcasecmp(str1: bstr, str2: bstr) -> ::core::ffi::c_int;
    fn bstr_strip(str: bstr) -> bstr;
    fn bstr_split(str: bstr, sep: *const ::core::ffi::c_char, rest: *mut bstr) -> bstr;
    fn mp_strip_ext(path: bstr) -> bstr;
}
pub type __uint8_t = u8;
pub type uint8_t = __uint8_t;
pub type size_t = usize;
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bstr {
    pub start: *mut ::core::ffi::c_uchar,
    pub len: size_t,
}
pub type track_flags = ::core::ffi::c_uint;
pub const TRACK_COMMENTARY: track_flags = 128;
pub const TRACK_DEFAULT: track_flags = 64;
pub const TRACK_FORCED: track_flags = 32;
pub const TRACK_ATTACHED_PICTURE: track_flags = 16;
pub const TRACK_VISUAL_IMPAIRED: track_flags = 8;
pub const TRACK_HEARING_IMPAIRED: track_flags = 4;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed {
    pub s: [::core::ffi::c_char; 3],
    pub l: uint8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lang {
    pub match_0: C2RustUnnamed_0,
    pub canonical: C2RustUnnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub s: [::core::ffi::c_char; 3],
    pub l: uint8_t,
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
#[inline]
unsafe extern "C" fn bsearch(
    mut __key: *const ::core::ffi::c_void,
    mut __base: *const ::core::ffi::c_void,
    mut __nmemb: size_t,
    mut __size: size_t,
    mut __compar: __compar_fn_t,
) -> *mut ::core::ffi::c_void {
    let mut __l: size_t = 0;
    let mut __u: size_t = 0;
    let mut __idx: size_t = 0;
    let mut __p: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut __comparison: ::core::ffi::c_int = 0;
    __l = 0 as size_t;
    __u = __nmemb;
    while __l < __u {
        __idx = __l.wrapping_add(__u).wrapping_div(2 as size_t);
        __p = (__base as *const ::core::ffi::c_char).offset(__idx.wrapping_mul(__size) as isize)
            as *const ::core::ffi::c_void;
        __comparison = Some(__compar.expect("non-null function pointer"))
            .expect("non-null function pointer")(__key, __p);
        if __comparison < 0 as ::core::ffi::c_int {
            __u = __idx;
        } else if __comparison > 0 as ::core::ffi::c_int {
            __l = __idx.wrapping_add(1 as size_t);
        } else {
            return __p as *mut ::core::ffi::c_void;
        }
    }
    return NULL;
}
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
unsafe extern "C" fn bstrcasecmp0(
    mut str1: bstr,
    mut str2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    return bstrcasecmp(str1, bstr0(str2));
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
unsafe extern "C" fn mp_isalpha(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (mp_isupper(c) != 0 || mp_islower(c) != 0) as ::core::ffi::c_int;
}
static mut langmap: [lang; 206] = [lang {
    match_0: C2RustUnnamed_0 { s: [0; 3], l: 0 },
    canonical: C2RustUnnamed { s: [0; 3], l: 0 },
}; 206];
unsafe extern "C" fn lang_compare(
    mut key: *const ::core::ffi::c_void,
    mut lang: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut l: *const lang = lang as *const lang;
    return bstrcasecmp(
        *(key as *const bstr),
        bstr {
            start: &raw const (*l).match_0.s as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_uchar,
            len: (*l).match_0.l as size_t,
        },
    );
}
unsafe extern "C" fn canonicalize(mut lang: bstr) -> bstr {
    let mut l: *const lang = bsearch(
        &raw mut lang as *const ::core::ffi::c_void,
        &raw const langmap as *const lang as *const ::core::ffi::c_void,
        (::core::mem::size_of::<[lang; 206]>() as size_t)
            .wrapping_div(::core::mem::size_of::<lang>() as size_t),
        ::core::mem::size_of::<lang>() as size_t,
        Some(
            lang_compare
                as unsafe extern "C" fn(
                    *const ::core::ffi::c_void,
                    *const ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ),
    ) as *const lang;
    return if !l.is_null() {
        bstr {
            start: &raw const (*l).canonical.s as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_uchar,
            len: (*l).canonical.l as size_t,
        }
    } else {
        lang
    };
}
#[no_mangle]
pub unsafe extern "C" fn mp_match_lang(
    mut langs: *mut *mut ::core::ffi::c_char,
    mut lang: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if lang.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    let mut ta_ctx: *mut ::core::ffi::c_void = ta_oom_p(ta_new_context(::core::ptr::null_mut::<
        ::core::ffi::c_void,
    >()));
    let mut lang_parts_n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut lang_parts: *mut bstr = ::core::ptr::null_mut::<bstr>();
    let mut rest: bstr = bstr0(lang) as bstr;
    while rest.len != 0 {
        let mut s: bstr = bstr_split(
            rest as bstr,
            b"-\0" as *const u8 as *const ::core::ffi::c_char,
            &raw mut rest,
        ) as bstr;
        let mut nextidx_: size_t = lang_parts_n as size_t;
        if nextidx_
            >= ta_get_size(lang_parts as *mut ::core::ffi::c_void)
                .wrapping_div(::core::mem::size_of::<bstr>() as size_t)
        {
            lang_parts = ta_dbg_set_loc(
                ta_xrealloc_size(
                    ta_ctx,
                    lang_parts as *mut ::core::ffi::c_void,
                    ta_calc_array_size(
                        ::core::mem::size_of::<bstr>() as size_t,
                        ta_calc_prealloc_elems(nextidx_),
                    ),
                ),
                b"../.mpv/misc/language.c:265\0" as *const u8 as *const ::core::ffi::c_char,
            ) as *mut bstr;
        }
        *lang_parts.offset(lang_parts_n as isize) = s;
        lang_parts_n += 1;
    }
    let mut best_score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if !(lang_parts_n == 0) {
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !langs.is_null() && !(*langs.offset(idx as isize)).is_null() {
            rest = bstr0(*langs.offset(idx as isize)) as bstr;
            let mut part: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while rest.len != 0 {
                let mut s_0: bstr = bstr_split(
                    rest as bstr,
                    b"-\0" as *const u8 as *const ::core::ffi::c_char,
                    &raw mut rest,
                ) as bstr;
                if part == 0 {
                    if bstrcasecmp(
                        canonicalize(*lang_parts.offset(0 as ::core::ffi::c_int as isize)) as bstr,
                        canonicalize(s_0) as bstr,
                    ) != 0
                    {
                        break;
                    }
                    score = INT_MAX - idx;
                    part += 1;
                } else {
                    if part >= lang_parts_n {
                        break;
                    }
                    if bstrcasecmp(*lang_parts.offset(part as isize) as bstr, s_0 as bstr) != 0 {
                        score -= 1000 as ::core::ffi::c_int;
                    }
                    part += 1;
                }
            }
            score -= (lang_parts_n - part) * 1000 as ::core::ffi::c_int;
            best_score = if best_score > score {
                best_score
            } else {
                score
            };
            idx += 1;
        }
    }
    ta_free(ta_ctx);
    return best_score;
}
#[no_mangle]
pub unsafe extern "C" fn mp_guess_lang_from_filename(
    mut name: bstr,
    mut lang_start: *mut ::core::ffi::c_int,
    mut flags: *mut track_flags,
) -> bstr {
    name = bstr_strip(mp_strip_ext(name) as bstr) as bstr;
    if !lang_start.is_null() {
        *lang_start = -(1 as ::core::ffi::c_int);
    }
    if !flags.is_null() {
        *flags = 0 as track_flags;
    }
    if name.len < 2 as size_t {
        return bstr {
            start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
            len: 0,
        };
    }
    let mut lang_length: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = name.len.wrapping_sub(1 as size_t) as ::core::ffi::c_int;
    let mut suffixes_length: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut delimiter: ::core::ffi::c_char = '.' as i32 as ::core::ffi::c_char;
    if *name.start.offset(i as isize) as ::core::ffi::c_int == ')' as i32 {
        delimiter = '(' as i32 as ::core::ffi::c_char;
        i -= 1;
    }
    if *name.start.offset(i as isize) as ::core::ffi::c_int == ']' as i32 {
        delimiter = '[' as i32 as ::core::ffi::c_char;
        i -= 1;
    }
    let mut f: *mut track_flags = if !flags.is_null() {
        flags
    } else {
        let mut fresh0: track_flags = 0 as track_flags;
        &raw mut fresh0
    };
    loop {
        while i >= 0 as ::core::ffi::c_int
            && mp_isalpha(*name.start.offset(i as isize) as ::core::ffi::c_char) != 0
        {
            lang_length += 1;
            i -= 1;
        }
        if i >= 0 as ::core::ffi::c_int
            && lang_length >= 2 as ::core::ffi::c_int
            && *name.start.offset(i as isize) as ::core::ffi::c_int
                == delimiter as ::core::ffi::c_int
        {
            let mut matched: bool = false_0 != 0;
            static mut suffixes: [*const ::core::ffi::c_char; 3] = [
                b"sdh\0" as *const u8 as *const ::core::ffi::c_char,
                b"hi\0" as *const u8 as *const ::core::ffi::c_char,
                b"cc\0" as *const u8 as *const ::core::ffi::c_char,
            ];
            let mut tag: bstr = bstr {
                start: name
                    .start
                    .offset(i as isize)
                    .offset(1 as ::core::ffi::c_int as isize),
                len: lang_length as size_t,
            };
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (n as usize)
                < (::core::mem::size_of::<[*const ::core::ffi::c_char; 3]>() as usize)
                    .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>() as usize)
            {
                if bstrcasecmp0(tag as bstr, suffixes[n as usize]) == 0 {
                    *f = ::core::mem::transmute::<::core::ffi::c_uint, track_flags>(
                        *f as ::core::ffi::c_uint
                            | TRACK_HEARING_IMPAIRED as ::core::ffi::c_int as ::core::ffi::c_uint,
                    );
                    matched = true_0 != 0;
                    break;
                } else {
                    n += 1;
                }
            }
            if bstrcasecmp0(
                tag as bstr,
                b"forced\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0
            {
                *f = ::core::mem::transmute::<::core::ffi::c_uint, track_flags>(
                    *f as ::core::ffi::c_uint
                        | TRACK_FORCED as ::core::ffi::c_int as ::core::ffi::c_uint,
                );
                matched = true_0 != 0;
            }
            if bstrcasecmp0(
                tag as bstr,
                b"default\0" as *const u8 as *const ::core::ffi::c_char,
            ) == 0
            {
                *f = ::core::mem::transmute::<::core::ffi::c_uint, track_flags>(
                    *f as ::core::ffi::c_uint
                        | TRACK_DEFAULT as ::core::ffi::c_int as ::core::ffi::c_uint,
                );
                matched = true_0 != 0;
            }
            if matched {
                lang_length = 0 as ::core::ffi::c_int;
                i -= if delimiter as ::core::ffi::c_int != '.' as i32 {
                    2 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                };
                continue;
            }
        }
        if lang_length < suffixes_length + 1 as ::core::ffi::c_int
            || lang_length > suffixes_length + 8 as ::core::ffi::c_int
        {
            return bstr {
                start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
                len: 0,
            };
        }
        if !(i >= 0 as ::core::ffi::c_int
            && *name.start.offset(i as isize) as ::core::ffi::c_int == '-' as i32)
        {
            break;
        }
        lang_length += 1;
        i -= 1;
        suffixes_length = lang_length;
    }
    if lang_length < suffixes_length + 2 as ::core::ffi::c_int
        || lang_length > suffixes_length + 3 as ::core::ffi::c_int
        || i <= 0 as ::core::ffi::c_int
        || *name.start.offset(i as isize) as ::core::ffi::c_int != delimiter as ::core::ffi::c_int
    {
        return bstr {
            start: ::core::ptr::null_mut::<::core::ffi::c_uchar>(),
            len: 0,
        };
    }
    if !lang_start.is_null() {
        *lang_start = i;
    }
    return bstr {
        start: name
            .start
            .offset(i as isize)
            .offset(1 as ::core::ffi::c_int as isize),
        len: lang_length as size_t,
    };
}
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
unsafe extern "C" fn run_static_initializers() {
    langmap = [
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"aa\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"aar"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ab\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"abk"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ae\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ave"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"af\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"afr"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ak\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"aka"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"am\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"amh"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"an\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"arg"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ar\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ara"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"as\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"asm"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"av\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ava"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ay\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"aym"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"az\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"aze"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ba\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bak"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"be\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bel"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bg\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bul"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bh\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bih"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bi\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bis"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bm\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bam"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bn\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ben"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bo\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tib"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bod"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tib"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"br\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bre"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bs\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bos"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ca\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cat"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ce\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"che"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ces"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cze"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ch\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cha"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"co\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cos"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cr\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cre"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cs\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cze"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cu\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"chu"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cv\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"chv"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cy\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wel"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cym"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wel"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"da\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"dan"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"de\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ger"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"deu"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ger"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"dv\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"div"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"dz\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"dzo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ee\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ewe"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"el\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gre"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ell"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gre"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"en\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"eng"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"eo\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"epo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"es\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"spa"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"et\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"est"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"eu\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"baq"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"eus"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"baq"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fa\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"per"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fas"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"per"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ff\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ful"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fi\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fin"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fj\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fij"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fo\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fao"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fr\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fre"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fra"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fre"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fy\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"fry"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ga\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gle"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gd\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gla"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gl\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"glg"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gn\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"grn"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gu\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"guj"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"gv\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"glv"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ha\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hau"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"he\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"heb"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hi\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hin"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ho\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hmo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hr\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hrv"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ht\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hat"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hu\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hun"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hy\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"arm"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hye"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"arm"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"hz\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"her"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ia\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ina"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"id\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ind"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ie\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ile"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ig\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ibo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ii\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"iii"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ik\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ipk"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"io\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ido"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"is\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ice"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"isl"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ice"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"it\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ita"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"iu\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"iku"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ja\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"jpn"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"jv\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"jav"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ka\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"geo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kat"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"geo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kg\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kon"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ki\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kik"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kj\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kua"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kk\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kaz"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kl\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kal"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"km\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"khm"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kn\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kan"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ko\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kor"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kr\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kau"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ks\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kas"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ku\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kur"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kv\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kom"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kw\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"cor"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ky\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kir"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"la\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lat"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lb\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ltz"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lg\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lug"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"li\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lim"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ln\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lin"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lo\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lao"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lt\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lit"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lu\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lub"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lv\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"lav"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mg\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mlg"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mh\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mah"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mi\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mao"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mk\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mac"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mkd"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mac"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ml\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mal"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mn\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mon"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mr\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mar"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mri"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mao"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ms\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"may"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"msa"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"may"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mt\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mlt"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"my\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bur"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"mya"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"bur"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"na\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nau"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nb\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nob"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nd\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nde"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ne\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nep"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ng\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ndo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nl\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"dut"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nld"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"dut"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nn\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nno"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"no\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nor"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nr\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nbl"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nv\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nav"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ny\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"nya"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"oc\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"oci"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"oj\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"oji"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"om\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"orm"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"or\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ori"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"os\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"oss"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"pa\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"pan"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"pi\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"pli"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"pl\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"pol"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ps\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"pus"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"pt\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"por"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"qu\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"que"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rm\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"roh"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rn\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"run"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ro\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rum"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ron"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rum"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ru\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rus"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rw\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"kin"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sa\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"san"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sc\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"srd"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sd\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"snd"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"se\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sme"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sg\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sag"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"si\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sin"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sk\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"slo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sl\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"slv"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"slk"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"slo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sm\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"smo"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sn\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sna"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"so\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"som"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sq\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"alb"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sqi"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"alb"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sr\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"srp"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ss\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ssw"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"st\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sot"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"su\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sun"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sv\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"swe"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"sw\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"swa"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ta\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tam"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"te\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tel"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tg\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tgk"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"th\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tha"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ti\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tir"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tk\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tuk"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tl\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tgl"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tn\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tsn"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"to\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ton"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tr\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tur"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ts\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tso"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tt\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tat"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tw\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"twi"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ty\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"tah"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ug\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"uig"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"uk\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ukr"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ur\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"urd"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"uz\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"uzb"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ve\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"ven"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"vi\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"vie"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"vo\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"vol"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wa\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wln"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wo\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"wol"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"xh\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"xho"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"yi\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"yid"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"yo\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"yor"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"za\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"zha"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"zh\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"chi"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"zho"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"chi"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"zlm"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"may"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"zsm"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"may"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
        lang {
            match_0: C2RustUnnamed_0 {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"zu\0"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 3]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
            canonical: C2RustUnnamed {
                s: ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"zul"),
                l: (::core::mem::size_of::<[::core::ffi::c_char; 4]>() as usize)
                    .wrapping_sub(1 as usize) as uint8_t,
            },
        },
    ];
}
#[used]
#[cfg_attr(target_os = "linux", link_section = ".init_array")]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XIB")]
#[cfg_attr(target_os = "macos", link_section = "__DATA,__mod_init_func")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];
