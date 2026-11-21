#include "misc/codepoint_width.c"

int dt_ucd_lookup(uint32_t cp) { return ucdLookup(cp); }
int dt_ucd_grapheme_joins(int state, int lead, int trail) { return ucdGraphemeJoins(state, lead, trail); }
bool dt_ucd_grapheme_done(int state) { return ucdGraphemeDone(state); }
int dt_ucd_to_character_width(int val) { return ucdToCharacterWidth(val); }
bool dt_is_valid_partial_sequence(struct bstr seq) { return is_valid_partial_sequence(seq); }
