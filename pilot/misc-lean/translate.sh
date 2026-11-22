#!/usr/bin/env bash
# Translate crates/misc to Lean with Charon and Aeneas.
#
# Usage: pilot/misc-lean/translate.sh [OUT_DIR]
#   AENEAS_DIST  the unpacked Aeneas release holding `charon` and `aeneas`
#                (default: /home/user/spike-aeneas/dist)
#
# Writes two translations of the same crate:
#   OUT_DIR/loops/RmpvMisc.lean  Aeneas's default: a Rust loop is a call of
#                                the library's `loop` combinator.
#   OUT_DIR/rec/RmpvMisc.lean    With -loops-to-rec: a Rust loop is a
#                                recursive Lean function.
# and copies them to MiscLoops/ and MiscRec/ next to this script.
#
# Aeneas exits non-zero when it could not translate everything; the Lean
# file is still written, with axioms or `sorry` in place of what failed, so
# this script prints the exit code and the number of error lines and
# carries on. Read OUT_DIR/*/aeneas.log.
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
dist="${AENEAS_DIST:-/home/user/spike-aeneas/dist}"
out="$(mkdir -p "${1:-$here/out}" && cd "${1:-$here/out}" && pwd)"
crate="$here/../../crates/misc"

# Aeneas has no float type. The two functions whose signature has an `f64`
# are left out. The modules of C library stand-ins that need floats or
# formatting (cfloat, and cprintf, the printf engine) stay opaque: their
# functions become declarations without a body, which is what a C library
# call is on the C side too. So does the module of system calls.
exclude=(
  rmpv_misc::bstr::bstrtod
  rmpv_misc::random::mp_rand_next_double
  # Mutex and Condvar: Aeneas reports "Unimplemented" for each function
  # body, and the declarations it emits for the std types do not type-check.
  # dispatch, thread_pool and mp_cancel also hold boxed closures ("Dynamic
  # trait types are not supported yet"); with these modules in, Aeneas stops
  # with a fatal error and writes no file.
  rmpv_misc::thread_tools::_
  rmpv_misc::rendezvous::_
  rmpv_misc::threads::_
  rmpv_misc::dispatch::_
  rmpv_misc::thread_pool::_
  # The scanf engine stores through &mut f32 and &mut f64; there is no float
  # type in Aeneas's library.
  rmpv_misc::cscanf::_
  rmpv_misc::bstr::bstr_sscanf
)
opaque=(
  rmpv_misc::cfloat::_
  rmpv_misc::cprintf::_
  rmpv_misc::os::_
)
args=()
for name in "${exclude[@]}"; do args+=(--exclude "$name"); done
for name in "${opaque[@]}"; do args+=(--opaque "$name"); done

# Lean cannot show that a function is well defined when it calls itself
# from inside a `loop`, so in the default translation the three functions
# that walk a node tree in a loop stay opaque. -loops-to-rec needs no such
# exception.
tree=(
  --opaque rmpv_misc::json::read_sub
  --opaque rmpv_misc::json::json_append
  --opaque rmpv_misc::node::equal_mpv_value
)

translate() { # NAME, extra charon args..., then "--", then extra aeneas args
  local name="$1"; shift
  local charon_extra=()
  while [ "$1" != "--" ]; do charon_extra+=("$1"); shift; done
  shift
  mkdir -p "$out/$name"
  (cd "$crate" && "$dist/charon" cargo --preset=aeneas --rustc-arg=--cfg=aeneas "${args[@]}" \
    "${charon_extra[@]}" --dest-file "$out/$name/rmpv_misc.llbc") >"$out/$name/charon.log" 2>&1 \
    || { cat "$out/$name/charon.log"; exit 1; }
  # The lookup tables of codepoint_width.c have thousands of entries, and
  # the proof of their length needs more recursion depth than the default.
  "$dist/aeneas" -backend lean -no-progress-bar -max-recdepth 100000 "$@" \
    -dest "$out/$name" "$out/$name/rmpv_misc.llbc" >"$out/$name/aeneas.log" 2>&1
  echo "$name: aeneas exit code $?, error lines:" \
    "$(sed 's/\x1b\[[0-9;]*m//g' "$out/$name/aeneas.log" | grep -c '^\[Error\]')"
}

translate loops "${tree[@]}" --
translate rec -- -loops-to-rec
cp "$out/loops/RmpvMisc.lean" "$here/MiscLoops/RmpvMisc.lean"
cp "$out/rec/RmpvMisc.lean" "$here/MiscRec/RmpvMisc.lean"
