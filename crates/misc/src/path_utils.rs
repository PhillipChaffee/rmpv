//! Port of `misc/path_utils.c` as built without `HAVE_DOS_PATHS`.
//!
//! Paths are bytes, as in C. They become `OsStr` only where the operating
//! system is asked about them.

use std::ffi::OsStr;
use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _};

use crate::bstr::{bstr_find0, bstr_splice, bstr_split_tok, bstr0, bstrdup0, bstrrchr, bstrspn};
use crate::clib::{cstr_at, memchr, strchr, strcmp, strlen};
use crate::ctype::{mp_isalnum, mp_isalpha};

/// `mp_path_separators`.
const MP_PATH_SEPARATORS: &[u8] = b"/";

/// `mp_basename_bstr`.
#[must_use]
pub fn mp_basename_bstr(path: &[u8]) -> &[u8] {
    let separator_pos = bstrrchr(path, b'/' as i32);
    if separator_pos < 0 {
        return path;
    }
    bstr_splice(path, separator_pos + 1, path.len() as i32)
}

/// `mp_basename`: the tail of the C string `path` after its last `/`.
#[must_use]
pub fn mp_basename(path: &[u8]) -> &[u8] {
    mp_basename_bstr(bstr0(path))
}

/// `mp_dirname`: `path` up to and including its last `/`, or `"."`.
#[must_use]
pub fn mp_dirname(path: &[u8]) -> &[u8] {
    let path = bstr0(path);
    let len = path.len() - mp_basename(path).len();
    if len == 0 {
        return b".";
    }
    &path[..len]
}

/// `mp_path_strip_trailing_separator`: overwrites a trailing `/` of the C
/// string in `path` with NUL.
pub fn mp_path_strip_trailing_separator(path: &mut [u8]) {
    let len = strlen(path);
    if len > 0 && strchr(MP_PATH_SEPARATORS, path[len - 1]).is_some() {
        path[len - 1] = 0;
    }
}

/// `split_ext`: the extension without its dot, and the path without it.
///
/// `root` is the caller's current value of the C out-parameter; it comes
/// back unchanged, and the extension empty, if there is none.
#[must_use]
pub fn split_ext<'a>(path: &'a [u8], root: &'a [u8]) -> (&'a [u8], &'a [u8]) {
    let basename = mp_basename_bstr(path);

    // All leading dots are skipped, not only the one of a hidden file.
    let basename = bstr_splice(basename, bstrspn(basename, b"."), basename.len() as i32);

    let dot_pos = bstrrchr(basename, b'.' as i32);
    if dot_pos < 0 || dot_pos as usize == basename.len().wrapping_sub(1) {
        return (&path[..0], root);
    }
    let ext = bstr_splice(basename, dot_pos + 1, basename.len() as i32);
    (ext, &path[..path.len() - ext.len() - 1])
}

/// `mp_strip_ext`.
#[must_use]
pub fn mp_strip_ext(path: &[u8]) -> &[u8] {
    let (_, root) = split_ext(path, &path[..0]);
    if root.is_empty() { path } else { root }
}

/// `mp_get_ext`: empty where the C returns a NULL `bstr`.
#[must_use]
pub fn mp_get_ext(path: &[u8]) -> &[u8] {
    split_ext(path, &path[..0]).0
}

/// `mp_path_is_absolute`. `strchr` finds the terminator, so a path whose
/// first byte is NUL counts as absolute.
#[must_use]
pub fn mp_path_is_absolute(path: &[u8]) -> bool {
    !path.is_empty() && strchr(MP_PATH_SEPARATORS, path[0]).is_some()
}

/// `mp_path_join_bstr`. The result is a C string, so each part ends at its
/// first NUL byte.
#[must_use]
pub fn mp_path_join_bstr(p1: &[u8], p2: &[u8]) -> Vec<u8> {
    if p1.is_empty() {
        return bstrdup0(p2);
    }
    if p2.is_empty() {
        return bstrdup0(p1);
    }

    if mp_path_is_absolute(p2) {
        return bstrdup0(p2);
    }

    let have_separator = strchr(MP_PATH_SEPARATORS, p1[p1.len() - 1]).is_some();

    let mut out = bstrdup0(p1);
    if !have_separator {
        out.push(b'/');
    }
    out.extend_from_slice(&bstrdup0(p2));
    out
}

/// `mp_path_join` on two C strings.
#[must_use]
pub fn mp_path_join(p1: &[u8], p2: &[u8]) -> Vec<u8> {
    mp_path_join_bstr(bstr0(p1), bstr0(p2))
}

/// `mp_getcwd`: `$PWD` if it names the current directory, else what the
/// system reports. `None` where the C returns NULL.
#[must_use]
pub fn mp_getcwd() -> Option<Vec<u8>> {
    if let Some(e_wd) = std::env::var_os("PWD") {
        let e_wd = e_wd.into_vec();
        if mp_path_is_absolute(&e_wd)
            && let Ok(pwd_st) = std::fs::metadata(OsStr::from_bytes(&e_wd))
            && let Ok(dot_st) = std::fs::metadata(".")
            && pwd_st.dev() == dot_st.dev()
            && pwd_st.ino() == dot_st.ino()
        {
            return Some(e_wd);
        }
    }

    std::env::current_dir()
        .ok()
        .map(|wd| wd.into_os_string().into_vec())
}

/// The component loop of `mp_normalize_path`. `None` if a `..` component
/// was found, for which the C resolves the whole path with `realpath`.
fn normalize_components(path: &[u8]) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let end = path.len();
    let mut ptr = 0;
    while ptr < end {
        let next = match memchr(&path[ptr..], b'/', end - ptr) {
            Some(offset) => ptr + offset,
            None => end,
        };
        let component = &path[ptr..next];
        ptr = next + 1;

        if component.is_empty() || component == b"." {
            continue;
        }
        if component == b".." {
            return None;
        }

        result.push(b'/');
        result.extend_from_slice(component);
    }

    if result.is_empty() {
        result.push(b'/');
    }
    Some(result)
}

/// `mp_normalize_path` on a C string.
///
/// Returns an absolute path without `.`, empty or `..` components. URLs,
/// `"-"` and the empty string come back as they are. A path with a `..`
/// component is resolved through the file system, symbolic links included.
#[must_use]
pub fn mp_normalize_path(path: &[u8]) -> Vec<u8> {
    let path = bstr0(path);
    if cstr_at(path, 0) == 0 || mp_is_url(path) || strcmp(path, b"-") == 0 {
        return path.to_vec();
    }

    let joined;
    let mut full = path;
    if !mp_path_is_absolute(path) {
        let Some(cwd) = mp_getcwd() else {
            return path.to_vec();
        };
        joined = mp_path_join(&cwd, path);
        full = &joined;
    }

    match normalize_components(full) {
        Some(result) => result,
        None => match std::fs::canonicalize(OsStr::from_bytes(full)) {
            Ok(resolved) => resolved.into_os_string().into_vec(),
            Err(_) => full.to_vec(),
        },
    }
}

/// `mp_path_exists` on a C string.
#[must_use]
pub fn mp_path_exists(path: &[u8]) -> bool {
    std::fs::metadata(OsStr::from_bytes(bstr0(path))).is_ok()
}

/// `mp_path_isdir` on a C string.
#[must_use]
pub fn mp_path_isdir(path: &[u8]) -> bool {
    std::fs::metadata(OsStr::from_bytes(bstr0(path))).is_ok_and(|st| st.is_dir())
}

/// `mp_is_url`: whether `path` starts with a protocol and `://`.
#[must_use]
pub fn mp_is_url(path: &[u8]) -> bool {
    let proto = bstr_find0(path, b"://");
    if proto < 1 {
        return false;
    }
    // RFC 3986: a letter first, then letters, digits, '-', '+' and '.'.
    let mut i: i32 = 0;
    while i < proto {
        let c = path[i as usize];
        if (i == 0 && !mp_isalpha(c)) || (!mp_isalnum(c) && c != b'.' && c != b'-' && c != b'+') {
            return false;
        }
        i += 1;
    }
    true
}

/// `mp_split_proto`: the protocol of a URL, empty if `path` is not one.
/// `out_url` is the caller's current value of the C out-parameter; for a URL
/// it comes back as the part after `://`.
#[must_use]
pub fn mp_split_proto<'a>(path: &'a [u8], out_url: &'a [u8]) -> (&'a [u8], &'a [u8]) {
    if !mp_is_url(path) {
        return (&path[..0], out_url);
    }
    let (_, r, url) = bstr_split_tok(path, b"://");
    (r, url)
}

/// `mp_mkdirp` on a C string: creates `dir` and its missing parents with
/// mode 0777, ignoring errors.
///
/// For an empty `dir` the C searches from one byte past its copy of the
/// string. The port does nothing in that case.
pub fn mp_mkdirp(dir: &[u8]) {
    let dir = bstr0(dir);
    if mp_path_exists(dir) || dir.is_empty() {
        return;
    }

    let mut builder = std::fs::DirBuilder::new();
    builder.mode(0o777);
    let mut cdir = Some(1);
    while let Some(from) = cdir {
        let slash = strchr(&dir[from..], b'/').map(|offset| from + offset);
        let prefix = match slash {
            Some(pos) => &dir[..pos],
            None => dir,
        };

        // The C ignores the result of mkdir() as well.
        let _ = builder.create(OsStr::from_bytes(prefix));

        cdir = slash.map(|pos| pos + 1);
    }
}
