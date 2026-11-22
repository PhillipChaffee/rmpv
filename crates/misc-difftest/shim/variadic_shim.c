// Calls of C variadic functions with an argument list built at run time.
//
// A test cannot build a va_list. It can rely on the x86-64 System V calling
// convention instead: every integer-class argument (int, long, pointer)
// travels in its own 64-bit slot, general-purpose registers first and then
// the stack, and the first 8 doubles travel in xmm0 to xmm7. va_arg takes
// integer-class values from one sequence and doubles from the other, in the
// order the callee asks for them. So passing DT_INTS integer slots followed
// by DT_DOUBLES doubles lets the format decide which it reads, and in which
// interleaving, as long as it reads at most DT_DOUBLES doubles. Unread
// arguments are ignored, as C allows. An int read from a slot is its low 32
// bits, which is where a caller's int would be.
#include <errno.h>
#include <stdio.h>

#include "misc/bstr.h"

#define DT_INTS 24
#define DT_DOUBLES 8

#define DT_I(a) a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7], a[8], a[9], \
    a[10], a[11], a[12], a[13], a[14], a[15], a[16], a[17], a[18], a[19], \
    a[20], a[21], a[22], a[23]
#define DT_D(d) d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]

// Also reports errno, which is what tells a failure's cause.
int dt_snprintf(char *buf, size_t size, const char *fmt, const long *ints,
                const double *dbls, int *err)
{
    errno = 0;
    int r = snprintf(buf, size, fmt, DT_I(ints), DT_D(dbls));
    *err = errno;
    return r;
}

int dt_bstr_xappend_asprintf(bstr *s, const char *fmt, const long *ints,
                             const double *dbls)
{
    return bstr_xappend_asprintf(NULL, s, fmt, DT_I(ints), DT_D(dbls));
}

// Every scanf argument is a pointer, so only integer slots are needed.
int dt_sscanf(const char *str, const char *fmt, void *const *ptrs)
{
    return sscanf(str, fmt, DT_I(ptrs));
}

int dt_bstr_sscanf(struct bstr str, const char *fmt, void *const *ptrs)
{
    return bstr_sscanf(str, fmt, DT_I(ptrs));
}
