// Runs mpv's real audio.c on the script the Lean oracle reads, printing the
// same lines. No player is started: the functions under test read the options,
// the audio output chain, and the replaygain data of the selected audio track,
// which the script fills in.
#include "player/audio.c"

#include <inttypes.h>
#include <stdio.h>
#include <unistd.h>

static struct MPContext ctx;
static struct MPOpts opts;
static struct ao_chain chain;
static struct track track;
static struct sh_stream stream;
static struct mp_codec_params codec;
static struct replaygain_data replaygain;
// What the replaced ao_get_delay returns.
static double ao_delay;
// An audio output is only ever tested for NULL and handed to ao_get_delay.
static char fake_ao;

double __wrap_ao_get_delay(struct ao *ao)
{
    return ao_delay;
}

// Lean's Float.toBits gives every NaN this one pattern, so the sign and payload
// of a NaN are not compared.
static uint64_t bits(double d)
{
    if (d != d)
        return 0x7ff8000000000000;
    uint64_t u;
    memcpy(&u, &d, sizeof(u));
    return u;
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
    return u;
}

static float from_bits32(uint32_t u)
{
    float f;
    memcpy(&f, &u, sizeof(f));
    return f;
}

static void print_state(void)
{
    printf(" | ac=");
    if (ctx.ao_chain)
        printf("%016" PRIx64 ",%d", bits(chain.last_out_pts), !!chain.ao);
    else
        printf("none");
    printf(" ao=%d as=%016" PRIx64 " dly=%016" PRIx64
           " vol=%08" PRIx32 " sg=%08" PRIx32 " mute=%d mode=%d pre=%08" PRIx32
           " clip=%d fb=%08" PRIx32 " tr=",
           !!ctx.ao, bits(ctx.audio_speed), bits(ao_delay),
           bits32(opts.softvol_volume), bits32(opts.softvol_gain),
           opts.softvol_mute, opts.rgain_mode, bits32(opts.rgain_preamp),
           opts.rgain_clip, bits32(opts.rgain_fallback));
    if (!ctx.current_track[0][STREAM_AUDIO]) {
        printf("none");
    } else if (!codec.replaygain_data) {
        printf("norg");
    } else {
        printf("%08" PRIx32 ",%08" PRIx32 ",%08" PRIx32 ",%08" PRIx32,
               bits32(replaygain.track_gain), bits32(replaygain.track_peak),
               bits32(replaygain.album_gain), bits32(replaygain.album_peak));
    }
    printf("\n");
}

static void reset(void)
{
    // The log with no root prints nothing.
    ctx = (struct MPContext){.opts = &opts, .log = mp_null_log};
    opts = (struct MPOpts){0};
    chain = (struct ao_chain){0};
    codec = (struct mp_codec_params){0};
    stream = (struct sh_stream){.codec = &codec};
    track = (struct track){.stream = &stream};
    replaygain = (struct replaygain_data){0};
    ao_delay = 0;
}

int main(void)
{
    // A planted bug can make mpv loop forever; a correct run takes under a second.
    alarm(20);
    reset();
    char op[16];
    unsigned long long x, y;
    unsigned int f, g, h, i;
    int a, b;

    while (scanf("%15s", op) == 1) {
        if (!strcmp(op, "reset")) {
            reset();
            printf("ok");
        } else if (!strcmp(op, "nochain")) {
            ctx.ao_chain = NULL;
            printf("ok");
        } else if (!strcmp(op, "chain") && scanf("%llx %d", &x, &a) == 2) {
            chain.last_out_pts = from_bits(x);
            chain.ao = a ? (struct ao *)&fake_ao : NULL;
            ctx.ao_chain = &chain;
            printf("ok");
        } else if (!strcmp(op, "ao") &&
                   scanf("%d %llx %llx", &a, &x, &y) == 3) {
            ctx.ao = a ? (struct ao *)&fake_ao : NULL;
            ctx.audio_speed = from_bits(x);
            ao_delay = from_bits(y);
            printf("ok");
        } else if (!strcmp(op, "vol") && scanf("%x %x %d", &f, &g, &a) == 3) {
            opts.softvol_volume = from_bits32(f);
            opts.softvol_gain = from_bits32(g);
            opts.softvol_mute = a;
            printf("ok");
        } else if (!strcmp(op, "rgopts") &&
                   scanf("%d %x %d %x", &a, &f, &b, &g) == 4) {
            opts.rgain_mode = a;
            opts.rgain_preamp = from_bits32(f);
            opts.rgain_clip = b;
            opts.rgain_fallback = from_bits32(g);
            printf("ok");
        } else if (!strcmp(op, "notrack")) {
            ctx.current_track[0][STREAM_AUDIO] = NULL;
            printf("ok");
        } else if (!strcmp(op, "norg")) {
            ctx.current_track[0][STREAM_AUDIO] = &track;
            codec.replaygain_data = NULL;
            printf("ok");
        } else if (!strcmp(op, "rg") &&
                   scanf("%x %x %x %x", &f, &g, &h, &i) == 4) {
            ctx.current_track[0][STREAM_AUDIO] = &track;
            codec.replaygain_data = &replaygain;
            replaygain = (struct replaygain_data){
                .track_gain = from_bits32(f),
                .track_peak = from_bits32(g),
                .album_gain = from_bits32(h),
                .album_peak = from_bits32(i),
            };
            printf("ok");
        } else if (!strcmp(op, "nopts")) {
            printf("%016" PRIx64, bits(MP_NOPTS_VALUE));
        } else if (!strcmp(op, "dbgain") && scanf("%llx", &x) == 1) {
            printf("%016" PRIx64, bits(db_gain(from_bits(x))));
        } else if (!strcmp(op, "replaygain")) {
            printf("%08" PRIx32, bits32(compute_replaygain(&ctx)));
        } else if (!strcmp(op, "gain")) {
            printf("%08" PRIx32, bits32(audio_get_gain(&ctx)));
        } else if (!strcmp(op, "written")) {
            printf("%016" PRIx64, bits(written_audio_pts(&ctx)));
        } else if (!strcmp(op, "playing")) {
            printf("%016" PRIx64, bits(playing_audio_pts(&ctx)));
        } else {
            return 1;
        }
        print_state();
    }
    return 0;
}
