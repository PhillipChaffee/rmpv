// Runs mpv's real playlist.c on the script the Lean oracle reads, printing the
// same lines. Including the .c file gives access to its static functions.
#include "common/playlist.c"

#include <stdio.h>
#include <unistd.h>

#include "common/msg_control.h"

#define MAX_PARAMS 4
#define MAX_NUMBERS 16
// mp_get_user_path expands "~/" to this.
#define HOME "/home/u"

// The second playlist, which the transfer functions empty into the first.
static struct playlist *src;
// What the replaced mp_rand_in_range32 draws from.
static int numbers[MAX_NUMBERS];
static int num_numbers, numbers_used;
// What the replaced demux_open_url opens: 0 nothing, 1 a demuxer without a
// playlist, 2 a demuxer that owns src.
static int demux_kind;
static const char *demux_path, *demux_filetype;

// The k-th call maps the k-th scripted number into the requested range, so a
// wrong range shows up in the result.
uint32_t __wrap_mp_rand_in_range32(mp_rand_state *s, uint32_t min, uint32_t max)
{
    int number = numbers_used < num_numbers ? numbers[numbers_used] : 0;
    numbers_used++;
    return max <= min ? min : min + number % (max - min);
}

// Opens demux_path alone, and only as a directly passed playlist, so a wrong
// request shows up as a failed open.
struct demuxer *__wrap_demux_open_url(const char *url,
                                      struct demuxer_params *params,
                                      struct mp_cancel *cancel,
                                      struct mpv_global *global)
{
    if (!demux_kind || strcmp(url, demux_path) || !params->force_format ||
        strcmp(params->force_format, "playlist") ||
        params->stream_flags != STREAM_ORIGIN_DIRECT)
        return NULL;
    struct demuxer *d = talloc_zero(NULL, struct demuxer);
    d->filetype = demux_filetype;
    if (demux_kind == 2)
        d->playlist = src;
    return d;
}

void __wrap_demux_free(struct demuxer *demuxer)
{
    if (!demuxer)
        return;
    if (demuxer->playlist) {
        talloc_free(demuxer->playlist);
        src = talloc_zero(NULL, struct playlist);
    }
    talloc_free(demuxer);
}

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

static void print_entries(struct playlist *pl)
{
    printf(" entries=");
    for (int n = 0; n < pl->num_entries; n++) {
        if (n)
            printf(";");
        print_fields(pl, pl->entries[n]);
    }
}

// The fields the transfer functions can change in a playlist other than pl.
static void print_playlist(struct playlist *other)
{
    printf("cur=");
    print_entry(other->current);
    printf(" c=%d s=%d alloc=%llu", other->playlist_completed,
           other->playlist_started, (unsigned long long)other->id_alloc);
    print_entries(other);
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
    print_entries(pl);
    printf(" src: ");
    print_playlist(src);
    printf("\n");
}

// Reads a count and that many numbers for mp_rand_in_range32 to draw from.
static bool scan_numbers(void)
{
    numbers_used = 0;
    if (scanf("%d", &num_numbers) != 1 || num_numbers < 0 ||
        num_numbers > MAX_NUMBERS)
        return false;
    for (int n = 0; n < num_numbers; n++) {
        if (scanf("%d", &numbers[n]) != 1)
            return false;
    }
    return true;
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
    // A planted bug can make mpv loop forever; a correct run takes under a second.
    alarm(20);
    setenv("HOME", HOME, 1);
    struct mpv_global *global = talloc_zero(NULL, struct mpv_global);
    mp_msg_init(global);
    struct playlist *pl = talloc_zero(NULL, struct playlist);
    src = talloc_zero(NULL, struct playlist);
    detached = playlist_entry_new("/detached");
    char op[16], word[64], name[32], value[32];
    int a, b, c, d;

    while (scanf("%15s", op) == 1) {
        void *tmp = talloc_new(NULL);
        struct playlist_param params[MAX_PARAMS];
        bool restore_indexes = false;
        // mpv's callers free the emptied source of a transfer.
        bool renew_src = false;
        freed = 0;
        if (!strcmp(op, "reset")) {
            if (held && !held->pl)
                talloc_free(held);
            held = NULL;
            talloc_free(pl);
            pl = talloc_zero(NULL, struct playlist);
            talloc_free(src);
            src = talloc_zero(NULL, struct playlist);
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
        } else if (!strcmp(op, "shuffle") && scan_numbers()) {
            playlist_shuffle(pl);
            printf("ok");
        } else if (!strcmp(op, "srcshuffle") && scan_numbers()) {
            playlist_shuffle(src);
            printf("ok");
        } else if (!strcmp(op, "unshuffle")) {
            playlist_unshuffle(pl);
            printf("ok");
        } else if (!strcmp(op, "cmp") && scanf("%d %d", &a, &b) == 2) {
            printf("%d", cmp_unshuffle(&pl->entries[a], &pl->entries[b]));
        } else if (!strcmp(op, "nextpl") && scanf("%d", &a) == 1) {
            print_entry(playlist_get_first_in_next_playlist(pl, a));
        } else if (!strcmp(op, "samepl") && scanf("%d %63s", &a, word) == 2) {
            print_entry(playlist_get_first_in_same_playlist(
                entry_arg(pl, a), strcmp(word, "~") ? word : NULL));
        } else if (!strcmp(op, "srcinsert") && scanf("%d %63s", &a, word) == 2) {
            playlist_insert_at(src, new_entry(word), entry_arg(src, a));
            printf("ok");
        } else if (!strcmp(op, "srcpopulate") && scanf("%63s", word) == 1) {
            playlist_populate_playlist_path(src, word);
            printf("ok");
        } else if (!strcmp(op, "srcflags") && scanf("%d %d", &a, &b) == 2) {
            src->playlist_completed = a;
            src->playlist_started = b;
            printf("ok");
        } else if (!strcmp(op, "srcsetcur") && scanf("%d", &a) == 1) {
            src->current = entry_arg(src, a);
            printf("ok");
        } else if (!strcmp(op, "transferto") && scanf("%d", &a) == 1) {
            printf("%lld", (long long)playlist_transfer_entries_to(pl, a, src));
            renew_src = true;
        } else if (!strcmp(op, "transfer")) {
            printf("%lld", (long long)playlist_transfer_entries(pl, src));
            renew_src = true;
        } else if (!strcmp(op, "appendentries")) {
            printf("%lld", (long long)playlist_append_entries(pl, src));
            renew_src = true;
        } else if (!strcmp(op, "parse") &&
                   scanf("%63s %d %31s", word, &demux_kind, name) == 3) {
            demux_path = strncmp(word, "~/", 2)
                ? word : talloc_asprintf(tmp, "%s/%s", HOME, word + 2);
            demux_filetype = strcmp(name, "~") ? name : NULL;
            struct playlist *parsed = playlist_parse_file(word, NULL, global);
            if (parsed) {
                print_playlist(parsed);
                talloc_free(parsed);
            } else {
                printf("none");
            }
        } else {
            printf("bad");
        }
        print_state(pl);
        if (renew_src) {
            talloc_free(src);
            src = talloc_zero(NULL, struct playlist);
        }
        if (restore_indexes) {
            for (int n = 0; n < pl->num_entries; n++)
                pl->entries[n]->pl_index = n;
        }
        talloc_free(tmp);
    }
    return 0;
}
