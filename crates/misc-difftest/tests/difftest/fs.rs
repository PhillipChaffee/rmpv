//! `path_utils.c` and `io_utils.c`.
//!
//! The functions that ask the file system run on fewer inputs than the
//! pure ones, as each input costs system calls; the count is at each test.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::sync::Mutex;

use rmpv_misc::{io_utils, path_utils, random};
use rmpv_misc_difftest::ffi::Bstr;
use rmpv_misc_difftest::input::{self, Rng, cstring};
use rmpv_misc_difftest::{c, c_view, copy_cstr, r_view, same_view};

use crate::{N, run};

/// Tests that read or change the process's environment or directory tree
/// take this, so they do not see each other's changes.
static PROCESS_STATE: Mutex<()> = Mutex::new(());

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn path_text(rng: &mut Rng) -> Vec<u8> {
    let mut s = Vec::new();
    if rng.chance(3) {
        s.extend_from_slice(rng.pick(&[
            &b"/"[..],
            b"//",
            b"http://",
            b"a+b.c-d://",
            b"1ab://",
            b"://",
            b"file:/",
            b"x:/",
            b"./",
            b"../",
        ]));
    }
    for _ in 0..rng.below(5) {
        match rng.below(8) {
            0 => s.push(b'/'),
            1 => s.push(b'.'),
            2 => s.extend_from_slice(rng.pick(&[
                &b".."[..],
                b".",
                b"...",
                b".hidden",
                b"a.b",
                b"a.",
                b".a.b",
                b"://",
                b"\\",
                b":",
            ])),
            3 => s.push(input::byte(rng)),
            _ => s.extend_from_slice(&input::text(rng, 3)),
        }
        if rng.chance(3) {
            s.push(b'/');
        }
    }
    s
}

#[test]
fn views() {
    run("mp_basename_bstr", 300, N, |rng| {
        let s = path_text(rng);
        let want = c_view(&s, c!(mp_basename_bstr(Bstr::of(&s))));
        assert!(
            same_view(want, r_view(&s, path_utils::mp_basename_bstr(&s))),
            "{s:?}"
        );
    });
    run("mp_basename", 301, N, |rng| {
        let s = path_text(rng);
        let s0 = cstring(&s);
        let p = c!(mp_basename(s0.as_ptr().cast()));
        let got = path_utils::mp_basename(&s);
        assert_eq!(
            p as usize - s0.as_ptr() as usize,
            got.as_ptr() as usize - s.as_ptr() as usize,
            "{s:?}"
        );
        assert_eq!(copy_cstr(p), got, "{s:?}");
    });
    run("mp_dirname", 302, N, |rng| {
        let s = path_text(rng);
        let s0 = cstring(&s);
        let ret = c!(mp_dirname(s0.as_ptr().cast()));
        let got = path_utils::mp_dirname(&s);
        assert_eq!(
            rmpv_misc_difftest::copy_out(ret.start, ret.len),
            got,
            "{s:?}"
        );
        // Either both are the start of the path or both are the constant ".".
        assert_eq!(
            ret.start.cast_const() == s0.as_ptr(),
            got.as_ptr() == s.as_ptr(),
            "{s:?}"
        );
    });
    run("mp_path_strip_trailing_separator", 303, N, |rng| {
        let s = path_text(rng);
        let mut want = cstring(&s);
        c!(mp_path_strip_trailing_separator(want.as_mut_ptr().cast()));
        let mut got = s.clone();
        path_utils::mp_path_strip_trailing_separator(&mut got);
        assert_eq!(&want[..s.len()], got, "{s:?}");
    });
    run("split_ext", 304, N, |rng| {
        let s = path_text(rng);
        // The out-parameter starts as the whole path, to see it is left
        // alone when there is no extension.
        let mut root = Bstr::of(&s);
        let ext = c!(dt_split_ext(Bstr::of(&s), &raw mut root));
        let (r_ext, r_root) = path_utils::split_ext(&s, &s);
        assert!(same_view(c_view(&s, ext), r_view(&s, r_ext)), "{s:?}");
        assert!(same_view(c_view(&s, root), r_view(&s, r_root)), "{s:?}");
    });
    run("mp_strip_ext", 305, N, |rng| {
        let s = path_text(rng);
        let want = c_view(&s, c!(mp_strip_ext(Bstr::of(&s))));
        assert!(
            same_view(want, r_view(&s, path_utils::mp_strip_ext(&s))),
            "{s:?}"
        );
    });
    run("mp_get_ext", 306, N, |rng| {
        let s = path_text(rng);
        let want = c_view(&s, c!(mp_get_ext(Bstr::of(&s))));
        assert!(
            same_view(want, r_view(&s, path_utils::mp_get_ext(&s))),
            "{s:?}"
        );
    });
    run("mp_path_is_absolute", 307, N, |rng| {
        let s = path_text(rng);
        assert_eq!(
            c!(mp_path_is_absolute(Bstr::of(&s))),
            path_utils::mp_path_is_absolute(&s),
            "{s:?}"
        );
    });
    run("mp_is_url", 308, N, |rng| {
        let s = path_text(rng);
        assert_eq!(
            c!(mp_is_url(Bstr::of(&s))),
            path_utils::mp_is_url(&s),
            "{s:?}"
        );
    });
    run("mp_split_proto", 309, N, |rng| {
        let s = path_text(rng);
        let mut url = Bstr::of(&s);
        let proto = c!(mp_split_proto(Bstr::of(&s), &raw mut url));
        let (r_proto, r_url) = path_utils::mp_split_proto(&s, &s);
        assert!(same_view(c_view(&s, proto), r_view(&s, r_proto)), "{s:?}");
        assert!(same_view(c_view(&s, url), r_view(&s, r_url)), "{s:?}");
    });
}

#[test]
fn joins() {
    run("mp_path_join_bstr", 310, N, |rng| {
        let (a, b) = (path_text(rng), path_text(rng));
        let p = c!(mp_path_join_bstr(null_mut(), Bstr::of(&a), Bstr::of(&b)));
        assert_eq!(
            copy_cstr(p),
            path_utils::mp_path_join_bstr(&a, &b),
            "{a:?} {b:?}"
        );
        c!(ta_free(p.cast()));
    });
    run("mp_path_join", 311, N, |rng| {
        let (a, b) = (path_text(rng), path_text(rng));
        let (a0, b0) = (cstring(&a), cstring(&b));
        let p = c!(mp_path_join(
            null_mut(),
            a0.as_ptr().cast(),
            b0.as_ptr().cast()
        ));
        assert_eq!(
            copy_cstr(p),
            path_utils::mp_path_join(&a, &b),
            "{a:?} {b:?}"
        );
        c!(ta_free(p.cast()));
    });
}

fn c_normalize(path: &[u8]) -> Vec<u8> {
    let path0 = cstring(path);
    let p = c!(mp_normalize_path(null_mut(), path0.as_ptr().cast()));
    let out = copy_cstr(p);
    c!(ta_free(p.cast()));
    out
}

fn c_getcwd() -> Vec<u8> {
    let p = c!(mp_getcwd(null_mut()));
    let out = copy_cstr(p);
    c!(ta_free(p.cast()));
    out
}

fn set_pwd(value: Option<&[u8]>) {
    // SAFETY: every test that reads or writes the environment holds
    // PROCESS_STATE, so no other thread touches it during this call.
    unsafe {
        match value {
            Some(v) => std::env::set_var("PWD", OsStr::from_bytes(v)),
            None => std::env::remove_var("PWD"),
        }
    }
}

#[test]
fn cwd_and_normalize() {
    let _guard = PROCESS_STATE.lock().unwrap();
    let dir = scratch("normalize");
    std::fs::create_dir_all(dir.join("real/sub")).unwrap();
    std::fs::write(dir.join("real/file"), b"x").unwrap();
    std::os::unix::fs::symlink(dir.join("real/sub"), dir.join("link")).unwrap();
    std::os::unix::fs::symlink("nowhere", dir.join("dangling")).unwrap();
    let base = dir.as_os_str().as_bytes().to_vec();
    let cwd = std::env::current_dir()
        .unwrap()
        .into_os_string()
        .into_encoded_bytes();
    let old_pwd = std::env::var_os("PWD");

    // mp_getcwd has no argument; its inputs are $PWD and the directory.
    // 9 settings of $PWD cover each branch: unset, right, wrong, relative,
    // another name for the same directory, and names that do not exist.
    let mut via_link = base.clone();
    via_link.extend_from_slice(b"/link/..");
    let mut dotted = cwd.clone();
    dotted.extend_from_slice(b"/.");
    let pwds: [Option<&[u8]>; 9] = [
        None,
        Some(&cwd),
        Some(b"/"),
        Some(b"relative"),
        Some(b""),
        Some(&dotted),
        Some(b"/nonexistent/x"),
        Some(&via_link),
        Some(b"."),
    ];
    for pwd in pwds {
        set_pwd(pwd);
        assert_eq!(Some(c_getcwd()), path_utils::mp_getcwd(), "PWD={pwd:?}");
    }
    println!("difftest mp_getcwd: {} inputs agree", pwds.len());

    // 20,000 inputs: a relative path costs two stat() calls for the working
    // directory, and a path with ".." a realpath().
    run("mp_normalize_path", 320, 20_000, |rng| {
        if rng.chance(50) {
            set_pwd(rng.pick(&pwds));
        }
        let mut s = Vec::new();
        match rng.below(4) {
            0 => s.extend_from_slice(&base),
            1 => s.extend_from_slice(rng.pick(&[
                &b"/"[..],
                b"//",
                b"http://x/",
                b"-",
                b"",
                b"file:///",
            ])),
            2 => {}
            _ => s.push(b'/'),
        }
        for _ in 0..rng.below(6) {
            s.extend_from_slice(rng.pick(&[
                &b"/"[..],
                b"real",
                b"sub",
                b"link",
                b"dangling",
                b"file",
                b"..",
                b".",
                b"...",
                b"x",
                b"//",
                b"/./",
                b"/../",
                b"a.b",
            ]));
            if !rng.chance(4) {
                s.push(b'/');
            }
        }
        assert_eq!(
            c_normalize(&s),
            path_utils::mp_normalize_path(&s),
            "{:?}",
            String::from_utf8_lossy(&s)
        );
    });

    // 20,000 inputs: one stat() per call on each side.
    let entry = |rng: &mut Rng| {
        let mut s = base.clone();
        s.extend_from_slice(rng.pick(&[
            &b""[..],
            b"/",
            b"/real",
            b"/real/",
            b"/real/sub",
            b"/real/file",
            b"/real/file/",
            b"/link",
            b"/link/",
            b"/dangling",
            b"/missing",
            b"/real/../real",
            b"/real/file/..",
        ]));
        match rng.below(6) {
            0 => b"".to_vec(),
            1 => input::text(rng, 4),
            _ => s,
        }
    };
    run("mp_path_exists", 321, 20_000, |rng| {
        let s = entry(rng);
        let s0 = cstring(&s);
        assert_eq!(
            c!(mp_path_exists(s0.as_ptr().cast())),
            path_utils::mp_path_exists(&s),
            "{:?}",
            String::from_utf8_lossy(&s)
        );
    });
    run("mp_path_isdir", 322, 20_000, |rng| {
        let s = entry(rng);
        let s0 = cstring(&s);
        assert_eq!(
            c!(mp_path_isdir(s0.as_ptr().cast())),
            path_utils::mp_path_isdir(&s),
            "{:?}",
            String::from_utf8_lossy(&s)
        );
    });

    set_pwd(old_pwd.as_deref().map(OsStr::as_bytes));
}

/// Every directory below `root`, relative to it, sorted.
fn tree(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            out.push(path.strip_prefix(root).unwrap().to_path_buf());
            if path.is_dir() {
                walk(&path, root, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

#[test]
fn mp_mkdirp() {
    let dir = scratch("mkdirp");
    // 2,000 inputs: each creates and removes two directory trees.
    run("mp_mkdirp", 330, 2000, |rng| {
        let (c_root, r_root) = (dir.join("c"), dir.join("r"));
        std::fs::create_dir(&c_root).unwrap();
        std::fs::create_dir(&r_root).unwrap();
        if rng.chance(4) {
            // Something in the way: a file where a directory is wanted.
            std::fs::write(c_root.join("a"), b"").unwrap();
            std::fs::write(r_root.join("a"), b"").unwrap();
        }
        let mut rel = Vec::new();
        for _ in 0..=rng.below(4) {
            rel.extend_from_slice(rng.pick(&[&b"a"[..], b"b", b"cc", b".", b"a b", b"\xff"]));
            rel.extend_from_slice(rng.pick(&[&b"/"[..], b"/", b"//", b""]));
        }
        let full = |root: &Path| {
            let mut p = root.as_os_str().as_bytes().to_vec();
            p.push(b'/');
            p.extend_from_slice(&rel);
            p
        };
        let c_path = cstring(&full(&c_root));
        c!(mp_mkdirp(c_path.as_ptr().cast()));
        path_utils::mp_mkdirp(&full(&r_root));
        assert_eq!(
            tree(&c_root),
            tree(&r_root),
            "{:?}",
            String::from_utf8_lossy(&rel)
        );
        std::fs::remove_dir_all(&c_root).unwrap();
        std::fs::remove_dir_all(&r_root).unwrap();
    });
}

fn errno() -> i32 {
    // SAFETY: __errno_location returns the calling thread's errno, which is
    // always valid to read.
    unsafe { *rmpv_misc_difftest::ffi::__errno_location() }
}

#[test]
fn mp_mkostemps() {
    let dir = scratch("mkostemps");
    let (c_dir, r_dir) = (dir.join("c"), dir.join("r"));
    std::fs::create_dir(&c_dir).unwrap();
    std::fs::create_dir(&r_dir).unwrap();
    // 5,000 inputs: each creates and removes files. libavutil's random
    // bytes are replaced by the bytes of a known state, which the port is
    // given directly, so both sides must pick the same names.
    run("mp_mkostemps", 340, 5000, |rng| {
        let mut v = [
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
            rng.next_u64(),
        ];
        if rng.chance(3) {
            // A state whose first output is small, so that "%06zx" pads.
            v[1] = rng.below(1 << 20) as u64;
        }
        let mut seed_bytes = Vec::new();
        for word in v {
            seed_bytes.extend_from_slice(&word.to_ne_bytes());
        }
        let suffix = rng.pick(&[&b""[..], b".tmp", b"X", b"XXXXXX"]);
        let mut name = input::text(rng, 3);
        name.retain(|&b| b != b'/');
        name.extend_from_slice(rng.pick(&[
            &b"XXXXXX"[..],
            b"XXXXXX",
            b"XXXXXX",
            b"XXXXX",
            b"XXXXXXX",
            b"xxxxxx",
            b"",
        ]));
        name.extend_from_slice(suffix);
        let suffixlen = if rng.chance(5) {
            rng.int()
        } else {
            suffix.len() as i32
        };
        let subdir = if rng.chance(10) { "missing/" } else { "" };
        let flags = rng.pick(&[0, 0o2_000_000, 0o2000]);
        let template = |root: &Path| {
            let mut p = root.as_os_str().as_bytes().to_vec();
            p.push(b'/');
            p.extend_from_slice(subdir.as_bytes());
            p.extend_from_slice(&name);
            p
        };

        // Sometimes the first name is taken on both sides.
        if rng.chance(4) && name.ends_with(suffix) && name.len() >= 6 + suffix.len() {
            let mut state = random::MpRandState { v };
            let first = io_utils::mkostemps_name(random::mp_rand_next(&mut state));
            for root in [&c_dir, &r_dir] {
                let mut taken = template(root);
                let at = taken.len() - suffix.len() - 6;
                taken[at..at + 6].copy_from_slice(&first);
                let _ = std::fs::write(OsStr::from_bytes(&taken), b"");
            }
        }

        let mut c_template = cstring(&template(&c_dir));
        c!(dt_fix_random_bytes(seed_bytes.as_ptr(), 1));
        let fd = c!(mp_mkostemps(
            c_template.as_mut_ptr().cast(),
            suffixlen,
            flags
        ));
        let c_errno = errno();
        c!(dt_fix_random_bytes(seed_bytes.as_ptr(), 0));
        let mut r_template = template(&r_dir);
        let mut state = random::MpRandState { v };
        let got = io_utils::mp_mkostemps_with(&mut r_template, suffixlen, flags, &mut state);

        let shown = String::from_utf8_lossy(&name).into_owned();
        let prefix = c_dir.as_os_str().len();
        assert_eq!(
            &c_template[prefix..c_template.len() - 1],
            &r_template[prefix..],
            "{shown:?} {suffixlen}"
        );
        match got {
            Ok(file) => {
                assert!(
                    fd >= 0,
                    "{shown:?} {suffixlen}: C failed with errno {c_errno}"
                );
                let c_meta =
                    std::fs::metadata(OsStr::from_bytes(&c_template[..c_template.len() - 1]))
                        .unwrap();
                let r_meta = file.metadata().unwrap();
                assert_eq!(
                    c_meta.permissions().mode(),
                    r_meta.permissions().mode(),
                    "{shown:?}"
                );
                c!(close(fd));
            }
            Err(r_errno) => {
                assert!(fd < 0, "{shown:?} {suffixlen}: C succeeded");
                assert_eq!(c_errno, r_errno, "{shown:?} {suffixlen}");
            }
        }
        for root in [&c_dir, &r_dir] {
            for entry in std::fs::read_dir(root).unwrap() {
                std::fs::remove_file(entry.unwrap().path()).unwrap();
            }
        }
    });
}

#[test]
fn mp_save_to_file() {
    let dir = scratch("save");
    std::fs::create_dir(dir.join("ro")).unwrap();
    // 2,000 inputs: each writes and renames a file on both sides.
    run("mp_save_to_file", 350, 2000, |rng| {
        let data = if rng.chance(4) {
            Vec::new()
        } else {
            input::bytes(rng, 40)
        };
        let sub = rng.pick(&["", "", "", "missing/", "ro/"]);
        let path = |who: &str| {
            let mut p = dir.as_os_str().as_bytes().to_vec();
            p.extend_from_slice(format!("/{sub}{who}-out").as_bytes());
            p
        };
        // "ro" is a directory in which a directory already has the target's
        // name, so the final rename fails.
        if sub == "ro/" {
            for who in ["c", "r"] {
                let target = PathBuf::from(OsStr::from_bytes(&path(who)));
                let _ = std::fs::create_dir_all(target.join("occupied"));
            }
        }
        let c_path = cstring(&path("c"));
        let want = c!(mp_save_to_file(
            c_path.as_ptr().cast(),
            data.as_ptr().cast(),
            data.len()
        ));
        let got = io_utils::mp_save_to_file(&path("r"), &data);
        assert_eq!(want, got, "{sub:?} {data:?}");
        if got {
            let c_read = std::fs::read(OsStr::from_bytes(&path("c"))).unwrap();
            let r_read = std::fs::read(OsStr::from_bytes(&path("r"))).unwrap();
            assert_eq!(c_read, r_read);
            assert_eq!(r_read, data);
        }
        // When the final rename fails the C leaves its temporary file
        // behind, and so does the port. Both sides must leave the same
        // number of files with the same lengths of name.
        let target_dir = dir.join(sub);
        if target_dir.is_dir() {
            let mut left: [Vec<usize>; 2] = [Vec::new(), Vec::new()];
            for entry in std::fs::read_dir(&target_dir).unwrap() {
                let path = entry.unwrap().path();
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                for (who, list) in ["c-out", "r-out"].iter().zip(&mut left) {
                    if name.starts_with(who) {
                        list.push(name.len());
                        if name.len() > who.len() {
                            std::fs::remove_file(&path).unwrap();
                        }
                    }
                }
            }
            left[0].sort_unstable();
            left[1].sort_unstable();
            assert_eq!(left[0], left[1], "{sub:?}");
        }
    });
}
