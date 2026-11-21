// Runs mpv's real video.c on the script the Lean oracle reads, printing the
// same lines. No player is started: the functions under test read the options,
// a few fields of MPContext, the audio and video output chains, and the frame
// timestamps, which the script fills in. written_audio_pts and
// playing_audio_pts are mpv's own, from the unchanged audio.c in the build.
#include "player/video.c"

#include <inttypes.h>
#include <stdio.h>
#include <unistd.h>

#define MAX_PAST 24
#define MAX_NEXT 3

static struct MPContext ctx;
static struct MPOpts opts;
static struct ao_chain audio_chain;
static struct vo_chain video_chain;
static struct mp_output_chain filter;
static struct vo video_out;
static struct mp_vo_opts vo_opts;
static struct frame_info past[MAX_PAST];
static struct mp_image next[MAX_NEXT];
// What the replaced ao_get_delay returns, the format the replaced
// ao_get_format reports, and the one format the replaced af_fmt_is_pcm calls
// PCM.
static double ao_delay;
static int ao_format, pcm_format;
// An audio output is only ever tested for NULL and handed to the replaced
// functions.
static char fake_ao;

double __wrap_ao_get_delay(struct ao *ao)
{
    return ao_delay;
}

void __wrap_ao_get_format(struct ao *ao, int *samplerate, int *format,
                          struct mp_chmap *channels)
{
    // A distinct sample rate, so that a caller mixing up the two is noticed.
    *samplerate = ao_format + 1000;
    *format = ao_format;
}

bool __wrap_af_fmt_is_pcm(int format)
{
    return format == pcm_format;
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

// The same for a float and Float32.toBits.
static uint32_t bits32(float f)
{
    if (f != f)
        return 0x7fc00000;
    uint32_t u;
    memcpy(&u, &f, sizeof(u));
    return u == 0x80000000 ? 0 : u;
}

static float from_bits32(uint32_t u)
{
    float f;
    memcpy(&f, &u, sizeof(f));
    return f;
}

static void print_double(double d)
{
    printf("%016" PRIx64, bits(d));
}

static void field(const char *name, double d)
{
    printf(" %s=%016" PRIx64, name, bits(d));
}

static void print_state(void)
{
    printf(" | ac=");
    if (ctx.ao_chain)
        printf("%016" PRIx64 ",%d", bits(audio_chain.last_out_pts), !!audio_chain.ao);
    else
        printf("none");
    printf(" ao=%d", !!ctx.ao);
    field("as", ctx.audio_speed);
    field("dly", ao_delay);
    printf(" st=%d,%d", (int)ctx.audio_status, (int)ctx.video_status);
    field("adel", opts.audio_delay);
    printf(" mpc=%08" PRIx32, bits32(opts.default_max_pts_correction));
    field("d", ctx.delay);
    field("tac", ctx.total_avsync_change);
    printf(" dsa=%d", ctx.display_sync_active);
    field("lav", ctx.last_av_difference);
    if (ctx.vo_chain)
        printf(" vc=%d,%016" PRIx64, video_chain.is_sparse, bits(filter.container_fps));
    else
        printf(" vc=none");
    field("vpts", ctx.video_pts);
    field("vptsu", ctx.video_pts_unrounded);
    printf(" dms=%d", ctx.drop_message_shown);
    field("spd", opts.playback_speed);
    field("smv", opts.sync_max_video_change);
    field("sma", opts.sync_max_audio_change);
    printf(" smf=%d vs=%d", opts.sync_max_factor, vo_opts.video_sync);
    field("sfv", ctx.speed_factor_v);
    field("sfa", ctx.speed_factor_a);
    field("adc", ctx.audio_drift_compensation);
    field("avdf", ctx.avd_filtered);
    printf(" fmt=%d,%d pf=", ao_format, pcm_format);
    for (int n = 0; n < ctx.num_past_frames; n++) {
        printf("%s%016" PRIx64 ":%016" PRIx64, n ? "," : "",
               bits(past[n].duration), bits(past[n].approx_duration));
    }
    printf(" nf=");
    for (int n = 0; n < ctx.num_next_frames; n++)
        printf("%s%016" PRIx64, n ? "," : "", bits(next[n].pts));
    printf("\n");
}

static void reset(void)
{
    // The log with no root prints nothing.
    ctx = (struct MPContext){
        .opts = &opts,
        .log = mp_null_log,
        .video_out = &video_out,
        .past_frames = past,
    };
    opts = (struct MPOpts){0};
    audio_chain = (struct ao_chain){0};
    filter = (struct mp_output_chain){0};
    video_chain = (struct vo_chain){.filter = &filter};
    vo_opts = (struct mp_vo_opts){0};
    video_out = (struct vo){.opts = &vo_opts};
    ao_delay = 0;
    ao_format = pcm_format = 0;
}

int main(void)
{
    // A planted bug can make mpv loop forever; a correct run takes under a second.
    alarm(20);
    reset();
    char op[16];
    unsigned long long w, x, y, z;
    unsigned int f;
    int a, b;

    while (scanf("%15s", op) == 1) {
        if (!strcmp(op, "reset")) {
            reset();
            printf("ok");
        } else if (!strcmp(op, "nochain")) {
            ctx.ao_chain = NULL;
            printf("ok");
        } else if (!strcmp(op, "chain") && scanf("%llx %d", &x, &a) == 2) {
            audio_chain.last_out_pts = from_bits(x);
            audio_chain.ao = a ? (struct ao *)&fake_ao : NULL;
            ctx.ao_chain = &audio_chain;
            printf("ok");
        } else if (!strcmp(op, "ao") &&
                   scanf("%d %llx %llx", &a, &x, &y) == 3) {
            ctx.ao = a ? (struct ao *)&fake_ao : NULL;
            ctx.audio_speed = from_bits(x);
            ao_delay = from_bits(y);
            printf("ok");
        } else if (!strcmp(op, "status") && scanf("%d %d", &a, &b) == 2) {
            ctx.audio_status = a;
            ctx.video_status = b;
            printf("ok");
        } else if (!strcmp(op, "sync") &&
                   scanf("%llx %x %llx %llx %d", &x, &f, &y, &z, &a) == 5) {
            opts.audio_delay = from_bits(x);
            opts.default_max_pts_correction = from_bits32(f);
            ctx.delay = from_bits(y);
            ctx.total_avsync_change = from_bits(z);
            ctx.display_sync_active = a;
            printf("ok");
        } else if (!strcmp(op, "avstate") &&
                   scanf("%llx %llx %llx %d", &x, &y, &z, &a) == 4) {
            ctx.last_av_difference = from_bits(x);
            ctx.video_pts = from_bits(y);
            ctx.video_pts_unrounded = from_bits(z);
            ctx.drop_message_shown = a;
            printf("ok");
        } else if (!strcmp(op, "novo")) {
            ctx.vo_chain = NULL;
            printf("ok");
        } else if (!strcmp(op, "vo") && scanf("%d %llx", &a, &x) == 2) {
            video_chain.is_sparse = a;
            filter.container_fps = from_bits(x);
            ctx.vo_chain = &video_chain;
            printf("ok");
        } else if (!strcmp(op, "speed") &&
                   scanf("%llx %llx %llx %d", &x, &y, &z, &a) == 4) {
            opts.playback_speed = from_bits(x);
            opts.sync_max_video_change = from_bits(y);
            opts.sync_max_audio_change = from_bits(z);
            opts.sync_max_factor = a;
            printf("ok");
        } else if (!strcmp(op, "dsync") &&
                   scanf("%d %llx %llx %llx %llx", &a, &w, &x, &y, &z) == 5) {
            vo_opts.video_sync = a;
            ctx.speed_factor_v = from_bits(w);
            ctx.speed_factor_a = from_bits(x);
            ctx.audio_drift_compensation = from_bits(y);
            ctx.avd_filtered = from_bits(z);
            printf("ok");
        } else if (!strcmp(op, "fmt") && scanf("%d %d", &a, &b) == 2) {
            ao_format = a;
            pcm_format = b;
            printf("ok");
        } else if (!strcmp(op, "past") && scanf("%d", &a) == 1 && a >= 0 &&
                   a <= MAX_PAST) {
            ctx.num_past_frames = a;
            for (int n = 0; n < a; n++) {
                if (scanf("%llx %llx", &x, &y) != 2)
                    return 1;
                past[n] = (struct frame_info){.duration = from_bits(x),
                                              .approx_duration = from_bits(y)};
            }
            printf("ok");
        } else if (!strcmp(op, "next") && scanf("%d", &a) == 1 && a >= 0 &&
                   a <= MAX_NEXT) {
            ctx.num_next_frames = a;
            for (int n = 0; n < a; n++) {
                if (scanf("%llx", &x) != 1)
                    return 1;
                next[n].pts = from_bits(x);
                ctx.next_frames[n] = &next[n];
            }
            printf("ok");
        } else if (!strcmp(op, "consts")) {
            printf("%d,%d,%d,%016" PRIx64 ",%016" PRIx64, (int)STATUS_PLAYING,
                   (int)VS_DISP_RESAMPLE, (int)VS_DISP_TEMPO,
                   bits(MP_PTS_ROUNDING_TOLERANCE), bits(MP_NOPTS_VALUE));
        } else if (!strcmp(op, "rint") && scanf("%llx", &x) == 1) {
            // Not an mpv function: checks the model's rint against libm's.
            print_double(rint(from_bits(x)));
        } else if (!strcmp(op, "adjsync") && scanf("%llx %llx", &x, &y) == 2) {
            adjust_sync(&ctx, from_bits(x), from_bits(y));
            printf("ok");
        } else if (!strcmp(op, "avdiff") && scanf("%llx", &x) == 1) {
            update_av_diff(&ctx, from_bits(x));
            printf("ok");
        } else if (!strcmp(op, "avgdur")) {
            print_double(calc_average_frame_duration(&ctx));
        } else if (!strcmp(op, "bestspeed") &&
                   scanf("%llx %llx %llx %d", &x, &y, &z, &a) == 4) {
            print_double(calc_best_speed(from_bits(x), from_bits(y),
                                         from_bits(z), a));
        } else if (!strcmp(op, "findspeed") && scanf("%llx", &x) == 1) {
            print_double(find_best_speed(&ctx, from_bits(x)));
        } else if (!strcmp(op, "spdif")) {
            printf("%d", using_spdif_passthrough(&ctx));
        } else if (!strcmp(op, "drift") && scanf("%llx", &x) == 1) {
            adjust_audio_drift_compensation(&ctx, from_bits(x));
            printf("ok");
        } else if (!strcmp(op, "framedur") && ctx.vo_chain &&
                   ctx.num_past_frames >= 1 && ctx.num_next_frames >= 1) {
            calculate_frame_duration(&ctx);
            printf("ok");
        } else {
            return 1;
        }
        print_state();
    }
    return 0;
}
