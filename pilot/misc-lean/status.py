#!/usr/bin/env python3
"""Prints the status of each of the 141 functions of mpv's misc/ folder.

Usage: pilot/misc-lean/status.py [DIFFTEST_OUTPUT]

DIFFTEST_OUTPUT is what `make difftest` printed; its `difftest <name>: <n>
inputs agree` lines give the test column. The Aeneas columns are read from
MiscLoops/RmpvMisc.lean and MiscRec/RmpvMisc.lean next to this script: a
function is `body` if the file has a `def` for it and `axiom` if it has an
`axiom`.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

THREADS = "threaded; see the report"
# (C file, C function, Rust item or None, note)
F = []


def add(cfile, names, module=None, note=""):
    for entry in names.split():
        c, _, rust = entry.partition("=")
        F.append((cfile, c, f"{module}::{rust or c}" if module else None, note))


add("bstr.c", "bstrcmp bstrcasecmp bstrchr bstrrchr bstrcspn bstrspn bstr_find bstr_lstrip bstr_strip "
    "bstr_split bstr_split_tok bstr_splice bstrtoll bstrtod bstr_splitchar bstr_strip_linebreaks "
    "bstr_eatstart bstr_eatend bstr_lower", "bstr")
add("bstr.c", "bstr_sscanf", None, "variadic over a scanf format; mpv's callers are outside misc/")
add("bstr.c", "bstr_parse_utf8_code_length bstr_decode_partial_utf8 bstr_decode_utf8 bstr_split_utf8 "
    "bstr_validate_utf8 bstr_sanitize_utf8_latin1 resize_append bstr_xappend", "bstr")
add("bstr.c", "bstr_xappend_asprintf bstr_xappend_vasprintf", None,
    "variadic over a printf format; the port formats at each call site (cfloat::format_d, format_f)")
add("bstr.c", "bstr_case_startswith bstr_case_endswith h_to_i bstr_decode_hex bstr_find_in_list0 bstr_in_list0",
    "bstr")
add("charset_conv.c", "mp_charset_is_utf8 mp_charset_is_utf16 ms_bom_guess mp_charset_guess mp_iconv_to_utf8",
    "charset_conv")
add("codepoint_width.c", "ucdLookup=ucd_lookup ucdGraphemeJoins=ucd_grapheme_joins ucdGraphemeDone=ucd_grapheme_done "
    "ucdToCharacterWidth=ucd_to_character_width is_valid_partial_sequence term_disp_width", "codepoint_width")
add("dispatch.c", "queue_dtor mp_dispatch_create mp_dispatch_set_wakeup_fn mp_dispatch_set_onlock_fn "
    "mp_dispatch_append mp_dispatch_enqueue mp_dispatch_enqueue_autofree mp_dispatch_enqueue_notify "
    "mp_dispatch_cancel_fn mp_dispatch_run mp_dispatch_queue_process mp_dispatch_interrupt "
    "mp_dispatch_adjust_timeout mp_dispatch_lock mp_dispatch_unlock", None, THREADS)
add("hash.c", "mp_hash_to_bstr", "hash")
add("io_utils.c", "mp_mkostemps mp_save_to_file", "io_utils")
add("json.c", "eat_c eat_ws json_skip_whitespace read_id read_str read_sub json_parse write_json_str add_indent "
    "json_append json_append_str json_write json_write_pretty", "json")
add("language.c", "lang_compare canonicalize mp_match_lang mp_guess_lang_from_filename", "language")
add("natural_sort.c", "mp_natural_sort_cmp", "natural_sort")
add("node.c", "node_init node_array_add node_map_add node_map_badd node_map_add_string node_map_add_bstr "
    "node_map_add_int64 node_map_add_double node_map_add_flag node_map_get node_map_bget equal_mpv_value "
    "equal_mpv_node", "node")
add("path_utils.c", "mp_basename_bstr mp_basename mp_dirname mp_path_strip_trailing_separator split_ext "
    "mp_strip_ext mp_get_ext mp_path_is_absolute mp_path_join_bstr mp_path_join mp_getcwd mp_normalize_path "
    "mp_path_exists mp_path_isdir mp_is_url mp_split_proto mp_mkdirp", "path_utils")
add("random.c", "rotl_u64 splitmix64 mp_rand_seed mp_rand_next mp_rand_next_double mp_rand_in_range32", "random")
add("rendezvous.c", "mp_rendezvous", "rendezvous")
add("thread_pool.c", "worker_thread thread_pool_dtor add_thread mp_thread_pool_create thread_pool_add "
    "mp_thread_pool_queue mp_thread_pool_run", None, THREADS)
add("thread_tools.c", "mp_waiter_wait mp_waiter_wakeup mp_waiter_poll", "thread_tools")
add("thread_tools.c", "cancel_destroy mp_cancel_new trigger_locked mp_cancel_trigger mp_cancel_reset "
    "mp_cancel_test mp_cancel_wait retrigger_locked mp_cancel_set_cb mp_cancel_set_parent mp_cancel_get_fd",
    None, THREADS)
assert len(F) == 141, len(F)

# Why Aeneas has no body for a ported function.
NO_BODY = {
    "bstr::bstrtod": "left out: `f64` in the signature",
    "random::mp_rand_next_double": "left out: `f64` in the signature",
    "rendezvous::mp_rendezvous": "rejected: `Mutex`, `Condvar`",
    "thread_tools::mp_waiter_wait": "rejected: `Mutex`, `Condvar`",
    "thread_tools::mp_waiter_wakeup": "rejected: `Mutex`, `Condvar`",
    "thread_tools::mp_waiter_poll": "rejected: `Mutex`, `Condvar`",
}
LOOP_AXIOM = "axiom: Lean rejects recursion through `loop`"
# Tests that cover several functions under one line, or a line under another name.
TEST_ALIAS = {
    "mp_waiter_wait": "mp_waiter_wait, mp_waiter_wakeup, mp_waiter_poll",
    "mp_waiter_wakeup": "mp_waiter_wait, mp_waiter_wakeup, mp_waiter_poll",
    "mp_waiter_poll": "mp_waiter_wait, mp_waiter_wakeup, mp_waiter_poll",
}
UNTESTED = {
    "resize_append": "not testable: only manages capacity",
}


def lean_items(lib):
    text = open(os.path.join(HERE, lib, "RmpvMisc.lean")).read()
    defs = set(re.findall(r"^def ([\w.]+)", text, re.M))
    axioms = set(re.findall(r"^axiom ([\w.]+)", text, re.M))
    return defs, axioms


def aeneas(rust, lib):
    if rust in NO_BODY:
        return NO_BODY[rust]
    name = rust.replace("::", ".")
    defs, axioms = lib
    if name in defs:
        return "body"
    if name in axioms:
        return LOOP_AXIOM
    return "MISSING"


def main():
    tests = {}
    if len(sys.argv) > 1:
        for line in open(sys.argv[1]):
            m = re.match(r"difftest (.*): (\d+) inputs agree", line)
            if m:
                tests[m.group(1)] = int(m.group(2))
    loops, rec = lean_items("MiscLoops"), lean_items("MiscRec")
    count = {"ported": 0, "tested": 0, "inputs": 0, "loops": 0, "rec": 0}
    last = None
    for cfile, c, rust, note in F:
        if cfile != last:
            print(f"\n### `misc/{cfile}`\n")
            print("| C function | Ported | Differential test | Aeneas, default | Aeneas, `-loops-to-rec` |")
            print("| --- | --- | --- | --- | --- |")
            last = cfile
        if rust is None:
            print(f"| `{c}` | no: {note} | not ported | not attempted | not attempted |")
            continue
        count["ported"] += 1
        key = TEST_ALIAS.get(c, c)
        if c in UNTESTED:
            test = UNTESTED[c]
        elif key in tests:
            test = f"{tests[key]:,} inputs, all agree"
            count["tested"] += 1
            count["inputs"] += tests[key]
        else:
            test = "MISSING"
        a_loops, a_rec = aeneas(rust, loops), aeneas(rust, rec)
        count["loops"] += a_loops == "body"
        count["rec"] += a_rec == "body"
        print(f"| `{c}` | `{rust}` | {test} | {a_loops} | {a_rec} |")
    print(f"\nTotals: {count['ported']} of 141 ported; {count['tested']} differentially tested, all agreeing; "
          f"{count['loops']} with a body in the default translation, {count['rec']} with -loops-to-rec.")


main()
