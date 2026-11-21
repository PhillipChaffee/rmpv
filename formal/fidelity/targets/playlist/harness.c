// Runs mpv's real playlist.c on the script the Lean oracle reads, printing the
// same lines. Including the .c file gives access to its static functions.
#include "common/playlist.c"

#include <stdio.h>

// Path handling is outside the checked functions; entries only need a filename.
char *mp_file_url_to_filename(void *talloc_ctx, bstr url)
{
    return NULL;
}

char *mp_normalize_path(void *talloc_ctx, const char *path)
{
    return talloc_strdup(talloc_ctx, path);
}

static struct playlist_entry *detached;

// Script index to entry: -1 is NULL, -2 is the detached entry.
static struct playlist_entry *entry_arg(struct playlist *pl, int i)
{
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

static void print_state(struct playlist *pl)
{
    printf(" | ids=");
    for (int n = 0; n < pl->num_entries; n++)
        printf("%s%llu", n ? "," : "", (unsigned long long)pl->entries[n]->id);
    printf(" idx=");
    for (int n = 0; n < pl->num_entries; n++)
        printf("%s%d", n ? "," : "", pl->entries[n]->pl_index);
    printf(" cur=");
    print_entry(pl->current);
    printf(" r=%d c=%d s=%d alloc=%llu\n", pl->current_was_replaced,
           pl->playlist_completed, pl->playlist_started,
           (unsigned long long)pl->id_alloc);
}

int main(void)
{
    struct playlist *pl = talloc_zero(NULL, struct playlist);
    detached = playlist_entry_new("detached");
    char op[16];
    int a, b, c, d;

    while (scanf("%15s", op) == 1) {
        bool restore_indexes = false;
        if (!strcmp(op, "reset")) {
            talloc_free(pl);
            pl = talloc_zero(NULL, struct playlist);
            printf("ok");
        } else if (!strcmp(op, "insert") && scanf("%d", &a) == 1) {
            playlist_insert_at(pl, playlist_entry_new("file"), entry_arg(pl, a));
            printf("ok");
        } else if (!strcmp(op, "remove") && scanf("%d", &a) == 1) {
            playlist_remove(pl, entry_arg(pl, a));
            printf("ok");
        } else if (!strcmp(op, "setcur") && scanf("%d", &a) == 1) {
            pl->current = entry_arg(pl, a);
            printf("ok");
        } else if (!strcmp(op, "flags") && scanf("%d %d %d", &a, &b, &c) == 3) {
            pl->current_was_replaced = a;
            pl->playlist_completed = b;
            pl->playlist_started = c;
            printf("ok");
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
    }
    return 0;
}
