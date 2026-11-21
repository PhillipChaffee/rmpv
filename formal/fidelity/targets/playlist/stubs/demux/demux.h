#pragma once
#include <stdbool.h>
struct mpv_global;
struct mp_cancel;
struct playlist;
struct demuxer_params { bool force_format_set; const char *force_format; int stream_flags; };
struct demuxer { struct playlist *playlist; const char *filetype; const struct { const char *name; } *desc; };
struct demuxer *demux_open_url(const char *url, struct demuxer_params *params, struct mp_cancel *cancel, struct mpv_global *global);
void demux_free(struct demuxer *demuxer);
