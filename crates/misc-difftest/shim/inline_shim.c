// Callable copies of the `static inline` helpers in mpv's headers, and thin
// wrappers around exported functions whose C signature is awkward to call
// from Rust. Everything mpv exports is called in libmpv.a, as mpv built it.
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <strings.h>

#include "common/common.h"
#include "common/msg.h"
#include "misc/bstr.h"
#include "misc/charset_conv.h"
#include "misc/ctype.h"
#include "misc/json.h"
#include "misc/node.h"

#include "dt_node.h"

int dt_mp_isprint(char c) { return mp_isprint(c); }
int dt_mp_isspace(char c) { return mp_isspace(c); }
int dt_mp_isupper(char c) { return mp_isupper(c); }
int dt_mp_islower(char c) { return mp_islower(c); }
int dt_mp_isdigit(char c) { return mp_isdigit(c); }
int dt_mp_isalpha(char c) { return mp_isalpha(c); }
int dt_mp_isalnum(char c) { return mp_isalnum(c); }
char dt_mp_tolower(char c) { return mp_tolower(c); }
char dt_mp_toupper(char c) { return mp_toupper(c); }

char *dt_bstrdup0(struct bstr s) { return bstrdup0(NULL, s); }
char *dt_bstrto0(struct bstr s) { return bstrto0(NULL, s); }
struct bstr dt_bstrdup(struct bstr s) { return bstrdup(NULL, s); }
struct bstr dt_bstr0(const char *s) { return bstr0(s); }
struct bstr dt_bstr_getline(struct bstr s, struct bstr *rest) { return bstr_getline(s, rest); }
struct bstr dt_bstr_cut(struct bstr s, int n) { return bstr_cut(s, n); }
bool dt_bstr_startswith(struct bstr s, struct bstr p) { return bstr_startswith(s, p); }
bool dt_bstr_startswith0(struct bstr s, const char *p) { return bstr_startswith0(s, p); }
bool dt_bstr_endswith(struct bstr s, struct bstr p) { return bstr_endswith(s, p); }
bool dt_bstr_endswith0(struct bstr s, const char *p) { return bstr_endswith0(s, p); }
int dt_bstrcmp0(struct bstr a, const char *b) { return bstrcmp0(a, b); }
bool dt_bstr_equals(struct bstr a, struct bstr b) { return bstr_equals(a, b); }
bool dt_bstr_equals0(struct bstr a, const char *b) { return bstr_equals0(a, b); }
int dt_bstrcasecmp0(struct bstr a, const char *b) { return bstrcasecmp0(a, b); }
int dt_bstr_find0(struct bstr h, const char *n) { return bstr_find0(h, n); }
bool dt_bstr_eatstart0(struct bstr *s, const char *p) { return bstr_eatstart0(s, p); }
bool dt_bstr_eatend0(struct bstr *s, const char *p) { return bstr_eatend0(s, p); }
void dt_bstr_xappend0(struct bstr *s, const char *a) { bstr_xappend0(NULL, s, a); }

struct mp_log *dt_null_log(void) { return mp_null_log; }

// The C library routines that the port replaces with its own code.
long long dt_strtoll(const char *s, int base, size_t *consumed, int *erange)
{
    char *end = (char *)s;
    errno = 0;
    long long r = strtoll(s, &end, base);
    *erange = errno == ERANGE;
    *consumed = end - s;
    return r;
}

double dt_strtod(const char *s, size_t *consumed, int *erange)
{
    char *end = (char *)s;
    errno = 0;
    double r = strtod(s, &end);
    *erange = errno == ERANGE;
    *consumed = end - s;
    return r;
}

int dt_memcmp(const void *a, const void *b, size_t n) { return memcmp(a, b, n); }
int dt_strncasecmp(const char *a, const char *b, size_t n) { return strncasecmp(a, b, n); }
int dt_strcasecmp(const char *a, const char *b) { return strcasecmp(a, b); }
int dt_strncmp(const char *a, const char *b, size_t n) { return strncmp(a, b, n); }
int dt_strcmp(const char *a, const char *b) { return strcmp(a, b); }
ptrdiff_t dt_strchr(const char *s, int c)
{
    const char *p = strchr(s, c);
    return p ? p - s : -1;
}
int dt_format_f(double v, char *buf, size_t size) { return snprintf(buf, size, "%f", v); }
int dt_format_d(int64_t v, char *buf, size_t size) { return snprintf(buf, size, "%" PRId64, v); }

// json.c and node.c through serialized nodes.

// Returns json_parse's result. `buf` is parsed in place from `*pos`; on
// success `*dump` is the serialized node (free() it).
int dt_json_parse(char *buf, size_t *pos, int max_depth, uint8_t **dump, size_t *dump_len)
{
    void *ta = talloc_new(NULL);
    struct mpv_node node = {0};
    char *src = buf + *pos;
    int r = json_parse(ta, &node, &src, max_depth);
    *pos = src - buf;
    *dump = NULL;
    if (r >= 0) {
        struct dt_buf b = {0};
        dt_node_dump(&b, &node);
        *dump = dt_finish(&b, dump_len);
    }
    talloc_free(ta);
    return r;
}

size_t dt_json_skip_whitespace(char *buf, size_t pos)
{
    char *src = buf + pos;
    json_skip_whitespace(&src);
    return src - buf;
}

// mode 0: json_append with `indent` onto `prefix`; 1: json_write; 2:
// json_write_pretty. The output is a malloc()ed copy of the resulting buffer.
int dt_json_write(int mode, const uint8_t *node_wire, const char *prefix, int indent,
                  uint8_t **out, size_t *out_len)
{
    void *ta = talloc_new(NULL);
    struct mpv_node node;
    dt_node_build(ta, &node, &node_wire);
    int r;
    if (mode == 0) {
        struct bstr b = {0};
        bstr_xappend(ta, &b, bstr0(prefix));
        r = json_append(&b, &node, indent);
        *out_len = b.len;
        *out = malloc(b.len + 1);
        if (b.len)
            memcpy(*out, b.start, b.len);
        talloc_free(b.start);
    } else {
        char *dst = talloc_strdup(NULL, prefix);
        r = mode == 1 ? json_write(&dst, &node) : json_write_pretty(&dst, &node);
        *out_len = strlen(dst);
        *out = malloc(*out_len + 1);
        memcpy(*out, dst, *out_len);
        talloc_free(dst);
    }
    talloc_free(ta);
    return r;
}

// One node.c building call on a serialized container. op selects the
// function; the result is the serialized container afterwards and the index
// of the entry the C returned a pointer to.
enum {
    DT_ARRAY_ADD, DT_MAP_ADD, DT_MAP_BADD, DT_MAP_ADD_STRING, DT_MAP_ADD_BSTR,
    DT_MAP_ADD_INT64, DT_MAP_ADD_DOUBLE, DT_MAP_ADD_FLAG,
};

uint8_t *dt_node_add(int op, const uint8_t *node_wire, struct bstr key, int format,
                     struct bstr sval, int64_t ival, double dval, int *index, size_t *out_len)
{
    void *ta = talloc_new(NULL);
    struct mpv_node node;
    dt_node_build(ta, &node, &node_wire);
    struct mpv_node *entry = NULL;
    switch (op) {
    case DT_ARRAY_ADD: entry = node_array_add(&node, format); break;
    case DT_MAP_ADD: entry = node_map_add(&node, (char *)key.start, format); break;
    case DT_MAP_BADD: entry = node_map_badd(&node, key, format); break;
    case DT_MAP_ADD_STRING: node_map_add_string(&node, (char *)key.start, (char *)sval.start); break;
    case DT_MAP_ADD_BSTR: node_map_add_bstr(&node, (char *)key.start, sval); break;
    case DT_MAP_ADD_INT64: node_map_add_int64(&node, (char *)key.start, ival); break;
    case DT_MAP_ADD_DOUBLE: node_map_add_double(&node, (char *)key.start, dval); break;
    case DT_MAP_ADD_FLAG: node_map_add_flag(&node, (char *)key.start, ival); break;
    }
    *index = entry ? (int)(entry - node.u.list->values) : -1;
    struct dt_buf b = {0};
    dt_node_dump(&b, &node);
    talloc_free(ta);
    return dt_finish(&b, out_len);
}

uint8_t *dt_node_init(int format, int parent_format, size_t *out_len)
{
    void *ta = talloc_new(NULL);
    struct mpv_node parent, node;
    if (parent_format >= 0)
        node_init(&parent, parent_format, NULL);
    node_init(&node, format, parent_format >= 0 ? &parent : NULL);
    struct dt_buf b = {0};
    dt_node_dump(&b, &node);
    if (parent_format >= 0) {
        // The node's allocation is a child of the parent's list.
        talloc_free(parent.u.list);
    } else if (format == MPV_FORMAT_BYTE_ARRAY) {
        talloc_free(node.u.ba);
    } else if (format == MPV_FORMAT_NODE_ARRAY || format == MPV_FORMAT_NODE_MAP) {
        talloc_free(node.u.list);
    }
    talloc_free(ta);
    return dt_finish(&b, out_len);
}

// Index of the entry node_map_get (bkey == 0) or node_map_bget returns, -1
// for NULL.
int dt_node_map_get(const uint8_t *node_wire, struct bstr key, int bkey)
{
    void *ta = talloc_new(NULL);
    struct mpv_node node;
    dt_node_build(ta, &node, &node_wire);
    struct mpv_node *r = bkey ? node_map_bget(&node, key) : node_map_get(&node, (char *)key.start);
    int index = r ? (int)(r - node.u.list->values) : -1;
    talloc_free(ta);
    return index;
}

// format < 0: equal_mpv_node; else equal_mpv_value on the unions.
bool dt_node_equal(const uint8_t *a_wire, const uint8_t *b_wire, int format)
{
    void *ta = talloc_new(NULL);
    struct mpv_node a, b;
    dt_node_build(ta, &a, &a_wire);
    dt_node_build(ta, &b, &b_wire);
    bool r;
    if (format < 0) {
        r = equal_mpv_node(&a, &b);
    } else if (format == MPV_FORMAT_NODE) {
        r = equal_mpv_value(&a, &b, format);
    } else if (format == MPV_FORMAT_BYTE_ARRAY) {
        r = equal_mpv_value(a.u.ba, b.u.ba, format);
    } else {
        r = equal_mpv_value(&a.u, &b.u, format);
    }
    talloc_free(ta);
    return r;
}

// mp_charset_guess: copies the result out and says where it points:
// >= 0 is an offset into user_cp, -1 a string of mpv's own.
ptrdiff_t dt_charset_guess(struct bstr buf, const char *user_cp, int flags, char *out, size_t size)
{
    void *ta = talloc_new(NULL);
    const char *r = mp_charset_guess(ta, mp_null_log, buf, user_cp, flags);
    snprintf(out, size, "%s", r);
    ptrdiff_t where = (r >= user_cp && r <= user_cp + strlen(user_cp)) ? r - user_cp : -1;
    talloc_free(ta);
    return where;
}
