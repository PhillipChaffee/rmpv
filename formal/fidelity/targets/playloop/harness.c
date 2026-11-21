// Runs mpv's real playloop.c on the script the Lean oracle reads, printing the
// same lines. No player is started: the functions under test read a few fields
// of MPContext, its options and its demuxer, which the script fills in.
#include "player/playloop.c"

#include <inttypes.h>
#include <stdio.h>
#include <unistd.h>

#define MAX_CHAPTERS 8

static struct MPContext ctx;
static struct MPOpts opts;
static struct demuxer demux;
static struct demux_chapter chapters[MAX_CHAPTERS];
// What the replaced get_play_start_pts and get_play_end_pts return.
static double range_start, range_end;

double __wrap_get_play_start_pts(struct MPContext *mpctx)
{
    return range_start;
}

double __wrap_get_play_end_pts(struct MPContext *mpctx)
{
    return range_end;
}

// The pattern printed for a double. Lean's Float.toBits gives every NaN one
// pattern, so the sign and payload of a NaN are not compared. Nor is the sign
// of a zero: mpv is built with -fno-signed-zeros, so the compiler chooses it,
// not mpv's source. The zero is tested on its bits because under that flag the
// compiler may drop a test on its value.
static uint64_t bits(double d)
{
    if (d != d)
        return 0x7ff8000000000000;
    uint64_t u;
    memcpy(&u, &d, sizeof(u));
    return u == 0x8000000000000000 ? 0 : u;
}

static double from_bits(uint64_t u)
{
    double d;
    memcpy(&d, &u, sizeof(d));
    return d;
}

static void print_double(double d)
{
    printf("%016" PRIx64, bits(d));
}

static void print_state(void)
{
    printf(" | d=");
    if (ctx.demuxer) {
        printf("%016" PRIx64 ",%016" PRIx64 ",%" PRId64 ",%" PRId64,
               bits(demux.duration), bits(demux.start_time), demux.filesize,
               demux.filepos);
    } else {
        printf("none");
    }
    printf(" rb=%d pf=%d pts=%016" PRIx64 " dir=%d seek=%016" PRIx64
           " mf=%d cs=%d cf=%d cb=%d rs=%016" PRIx64 " re=%016" PRIx64 " ch=",
           opts.rebase_start_time, opts.play_frames, bits(ctx.playback_pts),
           ctx.play_dir, bits(ctx.last_seek_pts), ctx.max_frames,
           ctx.last_chapter_seek, ctx.last_chapter_flag, ctx.cache_buffer,
           bits(range_start), bits(range_end));
    for (int n = 0; n < ctx.num_chapters; n++)
        printf("%s%016" PRIx64, n ? "," : "", bits(ctx.chapters[n].pts));
    printf("\n");
}

static void reset(void)
{
    ctx = (struct MPContext){.opts = &opts, .chapters = chapters};
    opts = (struct MPOpts){0};
    demux = (struct demuxer){0};
    range_start = range_end = 0;
}

int main(void)
{
    // A planted bug can make mpv loop forever; a correct run takes under a second.
    alarm(20);
    reset();
    char op[16];
    unsigned long long x, y;
    long long p, q;
    int a, b;

    while (scanf("%15s", op) == 1) {
        if (!strcmp(op, "reset")) {
            reset();
            printf("ok");
        } else if (!strcmp(op, "nodemux")) {
            ctx.demuxer = NULL;
            printf("ok");
        } else if (!strcmp(op, "demux") &&
                   scanf("%llx %llx %lld %lld", &x, &y, &p, &q) == 4) {
            demux = (struct demuxer){.duration = from_bits(x),
                                     .start_time = from_bits(y),
                                     .filesize = p,
                                     .filepos = q};
            ctx.demuxer = &demux;
            printf("ok");
        } else if (!strcmp(op, "opts") && scanf("%d %d", &a, &b) == 2) {
            opts.rebase_start_time = a;
            opts.play_frames = b;
            printf("ok");
        } else if (!strcmp(op, "time") &&
                   scanf("%llx %d %llx %d", &x, &a, &y, &b) == 4) {
            ctx.playback_pts = from_bits(x);
            ctx.play_dir = a;
            ctx.last_seek_pts = from_bits(y);
            ctx.max_frames = b;
            printf("ok");
        } else if (!strcmp(op, "chapters") && scanf("%d", &a) == 1 && a >= 0 &&
                   a <= MAX_CHAPTERS) {
            ctx.num_chapters = a;
            for (int n = 0; n < a; n++) {
                if (scanf("%llx", &x) != 1)
                    return 1;
                chapters[n] = (struct demux_chapter){.pts = from_bits(x)};
            }
            printf("ok");
        } else if (!strcmp(op, "chapseek") && scanf("%d %d", &a, &b) == 2) {
            ctx.last_chapter_seek = a;
            ctx.last_chapter_flag = b;
            printf("ok");
        } else if (!strcmp(op, "cache") && scanf("%d", &a) == 1) {
            ctx.cache_buffer = a;
            printf("ok");
        } else if (!strcmp(op, "range") && scanf("%llx %llx", &x, &y) == 2) {
            range_start = from_bits(x);
            range_end = from_bits(y);
            printf("ok");
        } else if (!strcmp(op, "nopts")) {
            print_double(MP_NOPTS_VALUE);
        } else if (!strcmp(op, "length")) {
            print_double(get_time_length(&ctx));
        } else if (!strcmp(op, "start") && scanf("%d", &a) == 1) {
            print_double(get_start_time(&ctx, a));
        } else if (!strcmp(op, "current")) {
            print_double(get_current_time(&ctx));
        } else if (!strcmp(op, "playback")) {
            print_double(get_playback_time(&ctx));
        } else if (!strcmp(op, "ratio") && scanf("%d", &a) == 1) {
            print_double(get_current_pos_ratio(&ctx, a));
        } else if (!strcmp(op, "chapter")) {
            printf("%d", get_current_chapter(&ctx));
        } else if (!strcmp(op, "chapstart") && scanf("%d", &a) == 1) {
            print_double(chapter_start_time(&ctx, a));
        } else if (!strcmp(op, "chapcount")) {
            printf("%d", get_chapter_count(&ctx));
        } else if (!strcmp(op, "cachepct")) {
            printf("%d", get_cache_buffering_percentage(&ctx));
        } else {
            return 1;
        }
        print_state();
    }
    return 0;
}
