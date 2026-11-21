// Replaces libavutil's av_random_bytes for the whole test program, so that a
// test can fix the state mp_rand_seed(0) returns and compare the file names
// mp_mkostemps derives from it. Without a fixed state it hands out bytes from
// the operating system, as libavutil does.
#include <stdint.h>
#include <string.h>
#include <sys/random.h>

static uint8_t dt_fixed[32];
static _Thread_local int dt_fixed_left;

// The next `count` calls on this thread that ask for 32 bytes get `bytes`.
void dt_fix_random_bytes(const uint8_t *bytes, int count)
{
    memcpy(dt_fixed, bytes, 32);
    dt_fixed_left = count;
}

int av_random_bytes(uint8_t *buf, size_t len)
{
    if (dt_fixed_left > 0 && len == 32) {
        dt_fixed_left--;
        memcpy(buf, dt_fixed, 32);
        return 0;
    }
    return getrandom(buf, len, 0) == (ssize_t)len ? 0 : -1;
}
