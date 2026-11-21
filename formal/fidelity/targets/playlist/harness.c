// Runs mpv's real playlist.c on the script the Lean oracle reads, printing the
// same lines. Including the .c file gives access to its static functions.
#include "common/playlist.c"

#include <stdio.h>

#define MAX_PARAMS 4

static struct playlist_entry *detached;
// An entry the script took a reference on, so it can outlive its removal.
static struct playlist_entry *held;
static int freed;

static void count_free(void *entry)
{
    freed++;
}

static struct playlist_entry *new_entry(const char *filename)
{
    struct playlist_entry *e = playlist_entry_new(filename);
    talloc_set_destructor(e, count_free);
    return e;
}

// Script index to entry: -1 is NULL, -2 the detached entry, -3 the held one.
static struct playlist_entry *entry_arg(struct playlist *pl, int i)
{
    if (i == -3)
        return held;
    if (i == -2)
        return detached;
    return i >= 0 && i < pl->num_entries ? pl->entries[i] : NULL;
}

static void print_entry(struct playlist_entry *e)
{
    if (e) {
        printf("%llu", (unsigned long long)e->id);
    } else {
        printf("none");
    }
}

static void print_fields(struct playlist *pl, struct playlist_entry *e)
{
    printf("%llu:%d:a%d:x%d:r%d:f%d:o%d:%s:%s:", (unsigned long long)e->id,
           e->pl_index, e->pl && e->pl == pl, e->removed, e->reserved,
           e->stream_flags, e->original_index, e->filename,
           e->playlist_path ? e->playlist_path : "~");
    for (int n = 0; n < e->num_params; n++) {
        printf("%s%.*s=%.*s", n ? "," : "", BSTR_P(e->params[n].name),
               BSTR_P(e->params[n].value));
    }
}

static void print_state(struct playlist *pl)
{
    printf(" | cur=");
    print_entry(pl->current);
    printf(" r=%d c=%d s=%d dir=%s alloc=%llu held=", pl->current_was_replaced,
           pl->playlist_completed, pl->playlist_started,
           pl->playlist_dir ? pl->playlist_dir : "~",
           (unsigned long long)pl->id_alloc);
    if (held) {
        print_fields(pl, held);
    } else {
        printf("none");
    }
    printf(" entries=");
    for (int n = 0; n < pl->num_entries; n++) {
        if (n)
            printf(";");
        print_fields(pl, pl->entries[n]);
    }
    printf("\n");
}

// Reads a count and that many name and value words.
static int scan_params(void *ctx, struct playlist_param *params)
{
    int count;
    if (scanf("%d", &count) != 1 || count < 0 || count > MAX_PARAMS)
        return -1;
    for (int n = 0; n < count; n++) {
        char name[32], value[32];
        if (scanf("%31s %31s", name, value) != 2)
            return -1;
        params[n] = (struct playlist_param){bstrdup(ctx, bstr0(name)),
                                            bstrdup(ctx, bstr0(value))};
    }
    return count;
}

int main(void)
{
    struct playlist *pl = talloc_zero(NULL, struct playlist);
    detached = playlist_entry_new("/detached");
    char op[16], word[64], name[32], value[32];
    int a, b, c, d;

    while (scanf("%15s", op) == 1) {
        void *tmp = talloc_new(NULL);
        struct playlist_param params[MAX_PARAMS];
        bool restore_indexes = false;
        freed = 0;
        if (!strcmp(op, "reset")) {
            if (held && !held->pl)
                talloc_free(held);
            held = NULL;
            talloc_free(pl);
            pl = talloc_zero(NULL, struct playlist);
            printf("ok");
        } else if (!strcmp(op, "new") && scanf("%63s", word) == 1) {
            struct playlist_entry *e = playlist_entry_new(word);
            print_fields(pl, e);
            talloc_free(e);
        } else if (!strcmp(op, "insert") && scanf("%d %63s", &a, word) == 2) {
            playlist_insert_at(pl, new_entry(word), entry_arg(pl, a));
            printf("ok");
        } else if (!strcmp(op, "append") && scanf("%63s", word) == 1) {
            playlist_append_file(pl, word);
            talloc_set_destructor(playlist_get_last(pl), count_free);
            printf("ok");
        } else if (!strcmp(op, "populate") && scanf("%63s", word) == 1) {
            playlist_populate_playlist_path(pl, word);
            printf("ok");
        } else if (!strcmp(op, "addparam") &&
                   scanf("%d %31s %31s", &a, name, value) == 3) {
            playlist_entry_add_param(entry_arg(pl, a), bstr0(name), bstr0(value));
            printf("ok");
        } else if (!strcmp(op, "addparams") && scanf("%d", &a) == 1 &&
                   (b = scan_params(tmp, params)) >= 0) {
            playlist_entry_add_params(entry_arg(pl, a), params, b);
            printf("ok");
        } else if (!strcmp(op, "setparams") &&
                   (b = scan_params(tmp, params)) >= 0) {
            playlist_set_params(pl, params, b);
            printf("ok");
        } else if (!strcmp(op, "remove") && scanf("%d", &a) == 1) {
            playlist_remove(pl, entry_arg(pl, a));
            printf("freed=%d", freed);
        } else if (!strcmp(op, "clear")) {
            playlist_clear(pl);
            printf("freed=%d", freed);
        } else if (!strcmp(op, "clearx")) {
            playlist_clear_except_current(pl);
            printf("freed=%d", freed);
        } else if (!strcmp(op, "move") && scanf("%d %d", &a, &b) == 2) {
            playlist_move(pl, entry_arg(pl, a), entry_arg(pl, b));
            printf("ok");
        } else if (!strcmp(op, "setcur") && scanf("%d", &a) == 1) {
            pl->current = entry_arg(pl, a);
            printf("ok");
        } else if (!strcmp(op, "flags") && scanf("%d %d %d", &a, &b, &c) == 3) {
            pl->current_was_replaced = a;
            pl->playlist_completed = b;
            pl->playlist_started = c;
            printf("ok");
        } else if (!strcmp(op, "setdir") && scanf("%d", &a) == 1) {
            TA_FREEP(&pl->playlist_dir);
            if (a)
                pl->playlist_dir = talloc_strdup(pl, "/dir");
            printf("ok");
        } else if (!strcmp(op, "streamflags") && scanf("%d", &a) == 1) {
            playlist_set_stream_flags(pl, a);
            printf("ok");
        } else if (!strcmp(op, "setcurrent")) {
            playlist_set_current(pl);
            printf("ok");
        } else if (!strcmp(op, "hold") && scanf("%d", &a) == 1) {
            held = entry_arg(pl, a);
            held->reserved += 1;
            printf("ok");
        } else if (!strcmp(op, "ref")) {
            held->reserved += 1;
            printf("ok");
        } else if (!strcmp(op, "unref")) {
            playlist_entry_unref(held);
            printf("freed=%d", freed);
            // With no reference left the playlist is the only owner again.
            if (freed || (held->pl && held->reserved <= 0))
                held = NULL;
        } else if (!strcmp(op, "count")) {
            printf("%d", playlist_entry_count(pl));
        } else if (!strcmp(op, "fromidx") && scanf("%d", &a) == 1) {
            print_entry(playlist_entry_from_index(pl, a));
        } else if (!strcmp(op, "toidx") && scanf("%d", &a) == 1) {
            printf("%d", playlist_entry_to_index(pl, entry_arg(pl, a)));
        } else if (!strcmp(op, "first")) {
            print_entry(playlist_get_first(pl));
        } else if (!strcmp(op, "last")) {
            print_entry(playlist_get_last(pl));
        } else if (!strcmp(op, "rel") && scanf("%d %d", &a, &b) == 2) {
            print_entry(playlist_entry_get_rel(entry_arg(pl, a), b));
        } else if (!strcmp(op, "next") && scanf("%d", &a) == 1) {
            print_entry(playlist_get_next(pl, a));
        } else if (!strcmp(op, "reindex") &&
                   scanf("%d %d %d %d", &a, &b, &c, &d) == 4) {
            if (a >= 0)
                pl->entries[a]->pl_index = b;
            playlist_update_indexes(pl, c, d);
            restore_indexes = true;
            printf("ok");
        } else {
            printf("bad");
        }
        print_state(pl);
        if (restore_indexes) {
            for (int n = 0; n < pl->num_entries; n++)
                pl->entries[n]->pl_index = n;
        }
        talloc_free(tmp);
    }
    return 0;
}
