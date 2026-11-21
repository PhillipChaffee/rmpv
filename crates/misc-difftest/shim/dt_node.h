// Serialization of mpv_node trees for the differential tests. The Rust side
// writes and reads the same format (src/node_wire.rs):
//   u8 format, then
//   STRING: u32 length, bytes        FLAG: i32          INT64: i64
//   DOUBLE: u64 bit pattern          BYTE_ARRAY: u32 length, bytes
//   NODE_ARRAY: u32 count, nodes     NODE_MAP: u32 count, (u32 length, key, node)...
// All integers are little-endian, as the host is.
#pragma once

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include <mpv/client.h>

#include "mpv_talloc.h"

struct dt_buf {
    uint8_t *data;
    size_t len, cap;
};

static void dt_put(struct dt_buf *b, const void *p, size_t n)
{
    if (b->len + n > b->cap) {
        b->cap = (b->len + n) * 2 + 64;
        b->data = realloc(b->data, b->cap);
    }
    if (n)
        memcpy(b->data + b->len, p, n);
    b->len += n;
}

static void dt_put_u32(struct dt_buf *b, uint32_t v) { dt_put(b, &v, 4); }

static void dt_put_str(struct dt_buf *b, const char *s)
{
    size_t n = strlen(s);
    dt_put_u32(b, n);
    dt_put(b, s, n);
}

static void dt_node_dump(struct dt_buf *b, const struct mpv_node *n)
{
    uint8_t tag = n->format;
    dt_put(b, &tag, 1);
    switch (n->format) {
    case MPV_FORMAT_STRING:
        dt_put_str(b, n->u.string);
        break;
    case MPV_FORMAT_FLAG:
        dt_put(b, &n->u.flag, 4);
        break;
    case MPV_FORMAT_INT64:
        dt_put(b, &n->u.int64, 8);
        break;
    case MPV_FORMAT_DOUBLE:
        dt_put(b, &n->u.double_, 8);
        break;
    case MPV_FORMAT_BYTE_ARRAY:
        dt_put_u32(b, n->u.ba->size);
        dt_put(b, n->u.ba->data, n->u.ba->size);
        break;
    case MPV_FORMAT_NODE_ARRAY:
    case MPV_FORMAT_NODE_MAP:
        dt_put_u32(b, n->u.list->num);
        for (int i = 0; i < n->u.list->num; i++) {
            if (n->format == MPV_FORMAT_NODE_MAP)
                dt_put_str(b, n->u.list->keys[i]);
            dt_node_dump(b, &n->u.list->values[i]);
        }
        break;
    default:
        break;
    }
}

static uint32_t dt_get_u32(const uint8_t **p)
{
    uint32_t v;
    memcpy(&v, *p, 4);
    *p += 4;
    return v;
}

static char *dt_get_str(void *ta, const uint8_t **p)
{
    uint32_t n = dt_get_u32(p);
    char *s = talloc_strndup(ta, (const char *)*p, n);
    *p += n;
    return s;
}

// Builds a node the way mpv's own code lays one out: the list is allocated
// under `ta`, and its arrays under the list.
static void dt_node_build(void *ta, struct mpv_node *n, const uint8_t **p)
{
    *n = (struct mpv_node){ .format = *(*p)++ };
    switch (n->format) {
    case MPV_FORMAT_STRING:
        n->u.string = dt_get_str(ta, p);
        break;
    case MPV_FORMAT_FLAG:
        memcpy(&n->u.flag, *p, 4);
        *p += 4;
        break;
    case MPV_FORMAT_INT64:
        memcpy(&n->u.int64, *p, 8);
        *p += 8;
        break;
    case MPV_FORMAT_DOUBLE:
        memcpy(&n->u.double_, *p, 8);
        *p += 8;
        break;
    case MPV_FORMAT_BYTE_ARRAY: {
        n->u.ba = talloc_zero(ta, struct mpv_byte_array);
        n->u.ba->size = dt_get_u32(p);
        n->u.ba->data = talloc_memdup(n->u.ba, *p, n->u.ba->size);
        *p += n->u.ba->size;
        break;
    }
    case MPV_FORMAT_NODE_ARRAY:
    case MPV_FORMAT_NODE_MAP: {
        struct mpv_node_list *l = talloc_zero(ta, struct mpv_node_list);
        n->u.list = l;
        int num = dt_get_u32(p);
        for (int i = 0; i < num; i++) {
            MP_TARRAY_GROW(l, l->values, l->num);
            if (n->format == MPV_FORMAT_NODE_MAP) {
                MP_TARRAY_GROW(l, l->keys, l->num);
                l->keys[l->num] = dt_get_str(l, p);
            }
            dt_node_build(l, &l->values[l->num], p);
            l->num++;
        }
        break;
    }
    default:
        break;
    }
}

static uint8_t *dt_finish(struct dt_buf *b, size_t *len)
{
    *len = b->len;
    return b->data ? b->data : malloc(1);
}

static uint8_t *dt_copy_out(const void *p, size_t n, size_t *len)
{
    uint8_t *out = malloc(n + 1);
    if (n)
        memcpy(out, p, n);
    *len = n;
    return out;
}
