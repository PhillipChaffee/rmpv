#include "misc/json.c"

#include "dt_node.h"

bool dt_eat_c(char *buf, size_t *pos, char c)
{
    char *src = buf + *pos;
    bool r = eat_c(&src, c);
    *pos = src - buf;
    return r;
}

size_t dt_eat_ws(char *buf, size_t pos)
{
    char *src = buf + pos;
    eat_ws(&src);
    return src - buf;
}

// which: 0 read_id, 1 read_str, 2 read_sub.
int dt_read(int which, char *buf, size_t *pos, int max_depth, uint8_t **dump, size_t *dump_len)
{
    void *ta = talloc_new(NULL);
    struct mpv_node node = {0};
    char *src = buf + *pos;
    int r = which == 0 ? read_id(ta, &node, &src)
          : which == 1 ? read_str(ta, &node, &src)
                       : read_sub(ta, &node, &src, max_depth);
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

uint8_t *dt_write_json_str(const char *prefix, unsigned char *str, size_t *len)
{
    struct bstr b = {0};
    bstr_xappend(NULL, &b, bstr0(prefix));
    write_json_str(&b, str);
    uint8_t *out = dt_copy_out(b.start, b.len, len);
    talloc_free(b.start);
    return out;
}

uint8_t *dt_add_indent(const char *prefix, int indent, size_t *len)
{
    struct bstr b = {0};
    bstr_xappend(NULL, &b, bstr0(prefix));
    add_indent(&b, indent);
    uint8_t *out = dt_copy_out(b.start, b.len, len);
    talloc_free(b.start);
    return out;
}

uint8_t *dt_json_append_str(const char *prefix, const uint8_t *node_wire, int indent, int *r,
                            size_t *len)
{
    void *ta = talloc_new(NULL);
    struct mpv_node node;
    dt_node_build(ta, &node, &node_wire);
    char *dst = talloc_strdup(NULL, prefix);
    *r = json_append_str(&dst, &node, indent);
    uint8_t *out = dt_copy_out(dst, strlen(dst), len);
    talloc_free(dst);
    talloc_free(ta);
    return out;
}
