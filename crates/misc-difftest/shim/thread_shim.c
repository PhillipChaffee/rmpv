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

// misc/dispatch.c: mp_dispatch_enqueue_autofree takes a talloc allocation as
// fn_data. This one starts with the same two fields as the tests' other
// callback data (context, id), and reports its own free.
#include "mpv_talloc.h"

struct dt_autofree {
    void *ctx;
    size_t id;
    void (*on_free)(void *ctx, size_t id);
};

static void dt_autofree_dtor(void *p)
{
    struct dt_autofree *a = p;
    a->on_free(a->ctx, a->id);
}

void *dt_autofree_new(void *ctx, size_t id, void (*on_free)(void *, size_t))
{
    struct dt_autofree *a = talloc_ptrtype(NULL, a);
    *a = (struct dt_autofree){ctx, id, on_free};
    talloc_set_destructor(a, dt_autofree_dtor);
    return a;
}
