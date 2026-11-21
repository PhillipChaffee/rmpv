//! Port of `misc/language.c`.

#[path = "language_tables.rs"]
mod tables;

use crate::bstr::{bstr_split, bstr_strip, bstr0, bstrcasecmp, bstrcasecmp0};
use crate::ctype::mp_isalpha;
use crate::path_utils::mp_strip_ext;
use tables::LANGMAP;

/// `enum track_flags` values that `mp_guess_lang_from_filename` sets.
pub const TRACK_HEARING_IMPAIRED: u32 = 1 << 2;
pub const TRACK_FORCED: u32 = 1 << 5;
pub const TRACK_DEFAULT: u32 = 1 << 6;

/// `struct lang`: a language code and its canonical three-letter form, each
/// as up to 3 bytes and a length.
#[cfg_attr(not(aeneas), derive(Debug, Clone, Copy))]
pub struct Lang {
    pub match_s: [u8; 3],
    pub match_l: u8,
    pub canonical_s: [u8; 3],
    pub canonical_l: u8,
}

/// `lang_compare`, the `bsearch` comparison: `key` against entry `index`.
#[must_use]
pub fn lang_compare(key: &[u8], index: usize) -> i32 {
    let l = &LANGMAP[index];
    bstrcasecmp(key, &l.match_s[..l.match_l as usize])
}

/// `canonicalize`: the canonical form of a language code, or the code itself
/// if the table does not have it. The loop is glibc's `bsearch`.
#[must_use]
pub fn canonicalize(lang: &[u8]) -> &[u8] {
    let mut lower = 0;
    let mut upper = LANGMAP.len();
    while lower < upper {
        let idx = (lower + upper) / 2;
        let comparison = lang_compare(lang, idx);
        if comparison < 0 {
            upper = idx;
        } else if comparison > 0 {
            lower = idx + 1;
        } else {
            let l = &LANGMAP[idx];
            return &l.canonical_s[..l.canonical_l as usize];
        }
    }
    lang
}

/// `lang_parts[part]`. The C keeps the parts of `lang` as an array of
/// views; here each part is its offset in `lang` and its length.
fn lang_part<'a>(lang: &'a [u8], lang_parts: &[(usize, usize)], part: usize) -> &'a [u8] {
    let (start, len) = lang_parts[part];
    &lang[start..start + len]
}

/// The `while (rest.len)` loop of `mp_match_lang` for one entry of `langs`:
/// its score before the best is taken.
fn match_one(lang: &[u8], lang_parts: &[(usize, usize)], candidate: &[u8], idx: i32) -> i32 {
    let lang_parts_n = lang_parts.len() as i32;
    let mut rest = candidate;
    let mut part: i32 = 0;
    let mut score: i32 = 0;
    let mut done = false;
    while !done && !rest.is_empty() {
        let (s, next) = bstr_split(rest, b"-");
        rest = next;
        if part == 0 {
            if bstrcasecmp(
                canonicalize(lang_part(lang, lang_parts, 0)),
                canonicalize(s),
            ) != 0
            {
                done = true;
            } else {
                score = i32::MAX - idx;
                part += 1;
            }
        } else if part >= lang_parts_n {
            done = true;
        } else {
            if bstrcasecmp(lang_part(lang, lang_parts, part as usize), s) != 0 {
                score -= 1000;
            }

            part += 1;
        }
    }
    score - (lang_parts_n - part) * 1000
}

/// `mp_match_lang`: how well the language tag `lang` matches the preference
/// list `langs`, 0 for no match. Both hold C strings; a NULL `lang` or
/// `langs` in C is the empty value here, with the same result.
#[must_use]
pub fn mp_match_lang(langs: &[Vec<u8>], lang: &[u8]) -> i32 {
    let lang = bstr0(lang);
    let mut lang_parts: Vec<(usize, usize)> = Vec::new();
    let mut rest = lang;
    while !rest.is_empty() {
        let (s, next) = bstr_split(rest, b"-");
        lang_parts.push((lang.len() - next.len() - s.len(), s.len()));
        rest = next;
    }

    let mut best_score = 0;
    if lang_parts.is_empty() {
        return best_score;
    }

    let mut idx: i32 = 0;
    while (idx as usize) < langs.len() {
        let score = match_one(lang, &lang_parts, bstr0(&langs[idx as usize]), idx);
        best_score = if best_score > score {
            best_score
        } else {
            score
        };
        idx += 1;
    }

    best_score
}

/// The `suffixes` loop of `mp_guess_lang_from_filename`.
fn is_hearing_impaired_tag(tag: &[u8]) -> bool {
    bstrcasecmp0(tag, b"sdh") == 0
        || bstrcasecmp0(tag, b"hi") == 0
        || bstrcasecmp0(tag, &[b'c', b'c']) == 0
}

/// The innermost loop of `mp_guess_lang_from_filename`: walks `i` left over
/// letters. Returns the new `i` and how many letters it passed.
fn skip_alpha_backwards(name: &[u8], i: i32) -> (i32, i32) {
    let mut i = i;
    let mut count = 0;
    while i >= 0 && mp_isalpha(name[i as usize]) {
        count += 1;
        i -= 1;
    }
    (i, count)
}

/// `mp_guess_lang_from_filename`: returns the language tag found at the end
/// of a file name (empty if none), the offset of the delimiter before it or
/// -1, and the track flags found on the way.
///
/// The flags are set from suffixes such as `.forced` even when no language
/// follows and the tag comes back empty, as in the C.
#[must_use]
pub fn mp_guess_lang_from_filename(name: &[u8]) -> (&[u8], i32, u32) {
    let name = bstr_strip(mp_strip_ext(name));
    let mut flags: u32 = 0;
    let none = &name[..0];

    if name.len() < 2 {
        return (none, -1, flags);
    }

    let mut lang_length: i32 = 0;
    let mut i: i32 = name.len() as i32 - 1;
    let mut suffixes_length: i32 = 0;

    let mut delimiter = b'.';
    if name[i as usize] == b')' {
        delimiter = b'(';
        i -= 1;
    }
    if name[i as usize] == b']' {
        delimiter = b'[';
        i -= 1;
    }

    let mut failed = false;
    let mut done = false;
    while !done {
        let (next_i, letters) = skip_alpha_backwards(name, i);
        i = next_i;
        lang_length += letters;

        let mut matched = false;
        if i >= 0 && lang_length >= 2 && name[i as usize] == delimiter {
            let tag = &name[(i + 1) as usize..(i + 1 + lang_length) as usize];
            if is_hearing_impaired_tag(tag) {
                flags |= TRACK_HEARING_IMPAIRED;
                matched = true;
            }
            if bstrcasecmp0(tag, b"forced") == 0 {
                flags |= TRACK_FORCED;
                matched = true;
            }
            if bstrcasecmp0(tag, b"default") == 0 {
                flags |= TRACK_DEFAULT;
                matched = true;
            }
        }
        if matched {
            lang_length = 0;
            i -= if delimiter == b'.' { 1 } else { 2 };
        } else if lang_length < suffixes_length + 1 || lang_length > suffixes_length + 8 {
            // Subtags after the first have 1 to 8 letters.
            failed = true;
            done = true;
        } else if i >= 0 && name[i as usize] == b'-' {
            lang_length += 1;
            i -= 1;
            suffixes_length = lang_length;
        } else {
            done = true;
        }
    }
    if failed {
        return (none, -1, flags);
    }

    // The primary subtag has 2 or 3 letters.
    if lang_length < suffixes_length + 2
        || lang_length > suffixes_length + 3
        || i <= 0
        || name[i as usize] != delimiter
    {
        return (none, -1, flags);
    }

    (
        &name[(i + 1) as usize..(i + 1 + lang_length) as usize],
        i,
        flags,
    )
}
