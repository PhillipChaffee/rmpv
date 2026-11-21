#pragma once
#include "misc/bstr.h"
#define STREAM_ORIGIN_DIRECT 1
char *mp_file_url_to_filename(void *talloc_ctx, bstr url);
