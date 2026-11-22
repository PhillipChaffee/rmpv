#!/usr/bin/env python3
"""Checks that the differential tests notice a wrong port.

Usage (from the repository root): crates/misc-difftest/tools/mutate.py

Each entry plants one bug in crates/misc, runs the test named with it, and
expects that test to fail. The source file is restored afterwards. Exits
non-zero if a bug went unnoticed or the text to replace is no longer there.
"""
import subprocess
import sys

# (file, text to replace, replacement, test filter)
MUTATIONS = [
    ("cfloat.rs", "let tiny = !(carried && top == -1023);", "let tiny = true;", "clib::strtod"),
    ("cfloat.rs", "let below = cmp_decimal_binary(digits, e10, (1 << 54) - 1, -1076) == Ordering::Less;",
     "let below = false;", "clib::strtod"),
    ("cfloat.rs", "return (bits, !exact);", "return (bits, true);", "clib::strtod"),
    ("cfloat.rs", "erange = overflow;", "erange = false;", "clib::strtod"),
    ("clib.rs", "scan_integer(s, base, true);", "scan_integer(s, base, false);", "bstrtoll"),
    ("clib.rs", "        if c1 == 0 {\n            return 0;\n        }\n        i += 1;\n    }\n    0\n}\n\n/// `strcasecmp`",
     "        i += 1;\n    }\n    0\n}\n\n/// `strcasecmp`", "bstrcasecmp"),
    ("bstr.rs", "end = if end as usize > s.len() {", "end = if end > s.len() as i32 {", "bstr_splice"),
    ("bstr.rs", "let end = bstr_cut(s, 0_usize.wrapping_sub(suffix.len()) as i32);",
     "let end = &s[s.len().saturating_sub(suffix.len())..];", "bstr_case_endswith"),
    ("bstr.rs", "if strchr(reject, s[i as usize]).is_some() {",
     "if s[i as usize] != 0 && strchr(reject, s[i as usize]).is_some() {", "bstrcspn"),
    ("bstr.rs", "    (res, s)\n}", "    (res, next)\n}", "bstr_decode_utf8"),
    ("common.rs", "if !num.is_empty() || c > 0x0010_ffff {", "if c > 0x0010_ffff {", "common_helpers"),
    ("json.rs", "if nsrci >= nsrcf {", "if nsrci > nsrcf {", "json_parse"),
    ("json.rs", "if indent == 0 {\n                bstr_xappend(b, bstr0(s));\n            } else {\n                write_json_str(b, s);\n            }",
     "write_json_str(b, s);", "json_write_pretty"),
    ("language.rs", "|| i <= 0\n", "|| i < 0\n", "mp_guess_lang_from_filename"),
    ("language.rs", "score = i32::MAX - idx;", "score = i32::MAX;", "mp_match_lang"),
    ("codepoint_width.rs", "        return (0, bstr_cut(s, 1), false);\n    }\n\n    *prev_pos = total - s.len();",
     "        *prev_pos = total - s.len();\n        return (0, bstr_cut(s, 1), false);\n    }\n\n    *prev_pos = total - s.len();",
     "term_disp_width"),
    ("io_utils.rs", "out[i] = digits[width - 1 - i];", "out[i] = digits[5 - i];", "mp_mkostemps"),
    ("node.rs", "if is_map && strcmp(&l_a.keys[n], &l_b.keys[n]) != 0 {",
     "if false && is_map && strcmp(&l_a.keys[n], &l_b.keys[n]) != 0 {", "equal_mpv_node"),
    ("path_utils.rs", "let skip = len == 0 || (len == 1 && component[0] == b'.');", "let skip = len == 0;",
     "cwd_and_normalize"),
    ("path_utils.rs", "!path.is_empty() && strchr(MP_PATH_SEPARATORS, path[0]).is_some()",
     "!path.is_empty() && path[0] == b'/'", "fs::views"),
    ("natural_sort.rs", "i1 = skip_zeros(name1, i1);", "", "mp_natural_sort_cmp"),
    ("hash.rs", "while msg.len() % 64 != 56 {", "while msg.len() % 64 != 55 {", "mp_hash_to_bstr"),
    ("cprintf.rs", "let octal_marker = prec <= number_length && word != 0 && s.alt && base == 8;",
     "let octal_marker = prec < number_length && word != 0 && s.alt && base == 8;", "vsnprintf"),
    ("cprintf.rs", "None if s.prec == -1 || s.prec >= 6 => b\"(null)\",",
     "None if s.prec == -1 || s.prec >= 5 => b\"(null)\",", "vsnprintf"),
    ("cprintf.rs", "round_away = next_value >= 8 && (last_value & 1 == 1 || more_bits);",
     "round_away = next_value >= 8;", "vsnprintf_floats"),
    ("cprintf.rs", "if s.alt && exp == p && exp_digits(magnitude, MAX_SIG_DIGITS).1 == p - 1 {",
     "if false {", "vsnprintf_floats"),
    ("cscanf.rs", "if c == b'-' || c == b'+' {\n        got_sign = true;", "if c == b'-' {\n        got_sign = true;",
     "vsscanf"),
    ("cscanf.rs", "} else if width != 0 && lower == Some(b'b') && (base == 2 || base == 0) {",
     "} else if width != 0 && lower == Some(b'b') && base == 2 {", "vsscanf"),
    ("cfloat.rs", "    if rem > half || (rem == half && (sticky || mant & 1 == 1)) {\n        mant += 1;\n    }\n    if top < -126 {",
     "    if rem > half || (rem == half && mant & 1 == 1) {\n        mant += 1;\n    }\n    if top < -126 {", "strtof"),
    ("dispatch.rs", "        if cur.mergeable && cur.key == item.key {", "        if cur.key == item.key {",
     "dispatch_sequences"),
    ("dispatch.rs", "    if queue.wakeup_fn.is_none() {\n        state.interrupted = true;\n    }",
     "    state.interrupted = true;", "dispatch_adjust_timeout"),
    ("thread_pool.rs", "        && num_threads < p.max_threads", "        && num_threads <= p.max_threads",
     "thread_pool_growth"),
    ("thread_tools.rs", "        ps.slaves.push(Arc::downgrade(slave));\n        retrigger_locked(new, &ps);",
     "        ps.slaves.push(Arc::downgrade(slave));", "mp_cancel_sequences"),
    ("thread_tools.rs", "    if let Some(pipe) = &state.wakeup_pipe {\n        // A full pipe",
     "    if let Some(pipe) = state.wakeup_pipe.as_ref().filter(|_| state.cb.is_none()) {\n        // A full pipe",
     "mp_cancel_sequences"),
]

ok = True
for name, old, new, test in MUTATIONS:
    path = "crates/misc/src/" + name
    source = open(path).read()
    if old not in source:
        print("MISSING   ", name, old[:50].replace("\n", " "))
        ok = False
        continue
    open(path, "w").write(source.replace(old, new, 1))
    try:
        run = subprocess.run(["cargo", "test", "-q", "-p", "rmpv-misc-difftest", "--release", "--", test],
                             capture_output=True, text=True)
    finally:
        open(path, "w").write(source)
    caught = run.returncode != 0 and "panicked" in run.stdout + run.stderr
    print("caught    " if caught else "NOT CAUGHT", name, test, "|", new[:50].replace("\n", " "))
    ok &= caught
sys.exit(0 if ok else 1)
