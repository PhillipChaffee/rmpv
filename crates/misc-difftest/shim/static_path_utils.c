#include "misc/path_utils.c"

struct bstr dt_split_ext(struct bstr path, struct bstr *root) { return split_ext(path, root); }
