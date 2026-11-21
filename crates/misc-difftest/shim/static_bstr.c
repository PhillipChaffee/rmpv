// The file-local functions of misc/bstr.c, compiled from mpv's source with
// mpv's flags. build.rs makes every symbol but dt_* local to this object.
#include "misc/bstr.c"

int dt_h_to_i(unsigned char c) { return h_to_i(c); }
