// misc/thread_tools.c: an mp_waiter on the heap, set up as
// MP_WAITER_INITIALIZER does it.
#include <stdlib.h>

#include "misc/thread_tools.h"

struct mp_waiter *dt_waiter_new(void)
{
    struct mp_waiter *w = malloc(sizeof(*w));
    *w = (struct mp_waiter)MP_WAITER_INITIALIZER;
    return w;
}
