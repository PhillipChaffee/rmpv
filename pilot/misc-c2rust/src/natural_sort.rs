#[inline]
unsafe extern "C" fn mp_isupper(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int >= 'A' as i32 && c as ::core::ffi::c_int <= 'Z' as i32)
        as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn mp_isdigit(mut c: ::core::ffi::c_char) -> ::core::ffi::c_int {
    return (c as ::core::ffi::c_int >= '0' as i32 && c as ::core::ffi::c_int <= '9' as i32)
        as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn mp_tolower(mut c: ::core::ffi::c_char) -> ::core::ffi::c_char {
    return (if mp_isupper(c) != 0 {
        c as ::core::ffi::c_int - 'A' as i32 + 'a' as i32
    } else {
        c as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn mp_natural_sort_cmp(
    mut name1: *const ::core::ffi::c_char,
    mut name2: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    while *name1.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
        && *name2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
    {
        if mp_isdigit(*name1.offset(0 as ::core::ffi::c_int as isize)) != 0
            && mp_isdigit(*name2.offset(0 as ::core::ffi::c_int as isize)) != 0
        {
            while *name1.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '0' as i32
            {
                name1 = name1.offset(1);
            }
            while *name2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '0' as i32
            {
                name2 = name2.offset(1);
            }
            let mut end1: *const ::core::ffi::c_char = name1;
            let mut end2: *const ::core::ffi::c_char = name2;
            while mp_isdigit(*end1) != 0 {
                end1 = end1.offset(1);
            }
            while mp_isdigit(*end2) != 0 {
                end2 = end2.offset(1);
            }
            if (end1.offset_from(name1) as ::core::ffi::c_long)
                < end2.offset_from(name2) as ::core::ffi::c_long
            {
                return -(1 as ::core::ffi::c_int);
            }
            if end1.offset_from(name1) as ::core::ffi::c_long
                > end2.offset_from(name2) as ::core::ffi::c_long
            {
                return 1 as ::core::ffi::c_int;
            }
            while name1 < end1 {
                if (*name1.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    < *name2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                {
                    return -(1 as ::core::ffi::c_int);
                }
                if *name1.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    > *name2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                {
                    return 1 as ::core::ffi::c_int;
                }
                name1 = name1.offset(1);
                name2 = name2.offset(1);
            }
        } else {
            if (mp_tolower(*name1.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_uchar
                as ::core::ffi::c_int)
                < mp_tolower(*name2.offset(0 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_uchar as ::core::ffi::c_int
            {
                return -(1 as ::core::ffi::c_int);
            }
            if mp_tolower(*name1.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_uchar
                as ::core::ffi::c_int
                > mp_tolower(*name2.offset(0 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_uchar as ::core::ffi::c_int
            {
                return 1 as ::core::ffi::c_int;
            }
            name1 = name1.offset(1);
            name2 = name2.offset(1);
        }
    }
    if *name2.offset(0 as ::core::ffi::c_int as isize) != 0 {
        return -(1 as ::core::ffi::c_int);
    }
    if *name1.offset(0 as ::core::ffi::c_int as isize) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
