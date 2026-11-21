#include "misc/language.c"

int dt_langmap_len(void) { return MP_ARRAY_SIZE(langmap); }
int dt_lang_compare(struct bstr key, int index) { return lang_compare(&key, &langmap[index]); }
struct bstr dt_canonicalize(struct bstr lang) { return canonicalize(lang); }
