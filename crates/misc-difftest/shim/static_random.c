#include "misc/random.c"

uint64_t dt_rotl_u64(uint64_t x, int k) { return rotl_u64(x, k); }
uint64_t dt_splitmix64(uint64_t *x) { return splitmix64(x); }
