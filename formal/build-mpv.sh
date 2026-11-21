#!/usr/bin/env bash
# Build mpv at the pinned commit as a static library, so the coverage and
# fidelity tools can read real compile flags and link real objects.
#
# Outputs, all under formal/.mpv-build:
#   compile_commands.json  one entry per compiled mpv C file
#   libmpv.a               every mpv object
#   harness.rsp            -I, -D and language flags for code that includes mpv
#   harness-link.rsp       libmpv.a and the libraries it needs, in link order
#
# Usage: cc @formal/.mpv-build/harness.rsp x.c @formal/.mpv-build/harness-link.rsp -o x
#
# Every stage is skipped when its output is already up to date.
set -euo pipefail

formal=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
lock=$formal/mpv.lock.toml
mpv_src=$formal/.mpv
deps=$formal/.deps
out=$formal/.mpv-build

die() {
    echo "build-mpv: $*" >&2
    exit 1
}

# Read `key` from `[table]` in the lock file.
lock_value() {
    sed -n "/^\[$1\]/,/^\[/ s/^$2 = \"\(.*\)\"/\1/p" "$lock"
}

[[ -f $mpv_src/meson.build ]] || die "mpv source missing at $mpv_src; run 'make mpv' first"

# The build must be of unpatched mpv at the locked commit; `make mpv` keeps
# local edits when it checks out, so check here.
mpv_commit=$(sed -n '/^\[/q; s/^commit = "\(.*\)"/\1/p' "$lock")
mpv_head=$(git -C "$mpv_src" rev-parse HEAD)
[[ $mpv_head == "$mpv_commit" ]] ||
    die "$mpv_src is at $mpv_head but mpv.lock.toml pins $mpv_commit; run 'make mpv'"
[[ -z $(git -C "$mpv_src" status --porcelain) ]] ||
    die "$mpv_src has local changes; discard them with 'git -C $mpv_src checkout -- .' and run 'make mpv'"

# --- Stage 1: system packages ------------------------------------------------

packages=(
    git gcc g++ ninja-build pkg-config python3 meson
    libavcodec-dev libavfilter-dev libavformat-dev libavutil-dev
    libswresample-dev libswscale-dev libass-dev
)

missing=()
for p in "${packages[@]}"; do
    status=$(dpkg-query -W -f='${Status}' "$p" 2>/dev/null || true)
    [[ $status == "install ok installed" ]] || missing+=("$p")
done
if ((${#missing[@]})); then
    if [[ $(id -u) == 0 ]] && command -v apt-get >/dev/null; then
        echo "build-mpv: installing ${missing[*]}"
        mkdir -p "$deps"
        {
            apt-get update -qq &&
                DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends "${missing[@]}"
        } >"$deps/apt.log" 2>&1 || die "apt-get failed, see $deps/apt.log"
    else
        die "missing packages: ${missing[*]}
install them with: sudo apt-get install ${missing[*]}"
    fi
fi

# --- Stage 2: libplacebo -----------------------------------------------------
# Ubuntu 24.04 ships libplacebo 6.338; mpv needs >=7.387.0.

lp_repo=$(lock_value libplacebo repo)
lp_commit=$(lock_value libplacebo commit)
[[ -n $lp_repo && -n $lp_commit ]] || die "no [libplacebo] repo and commit in $lock"

lp_src=$deps/libplacebo
lp_build=$deps/libplacebo-build
lp_prefix=$deps/prefix
lp_stamp=$deps/libplacebo.stamp
# GPU backends and every optional external library are off: mpv only needs
# libplacebo's CPU-side code once its own GPU outputs are disabled.
lp_options=(
    --prefix="$lp_prefix" --libdir=lib --buildtype=release
    -Ddefault_library=static -Dauto_features=disabled
    -Ddemos=false -Dtests=false -Dbench=false -Dfuzz=false
)
lp_key="$lp_commit ${lp_options[*]}"

if [[ ! -f $lp_stamp || $(cat "$lp_stamp") != "$lp_key" ]]; then
    echo "build-mpv: building libplacebo $lp_commit"
    rm -rf "$lp_src" "$lp_build" "$lp_prefix" "$lp_stamp"
    git init -q "$lp_src"
    git -C "$lp_src" fetch -q --depth 1 "$lp_repo" "$lp_commit"
    git -C "$lp_src" checkout -q --detach FETCH_HEAD
    # Without GPU backends only the code generators' Python libraries,
    # fast_float and the Vulkan headers are used; the Vulkan stub functions
    # are compiled even when the Vulkan backend is off.
    git -C "$lp_src" submodule -q update --init --depth 1 \
        3rdparty/jinja 3rdparty/markupsafe 3rdparty/fast_float 3rdparty/Vulkan-Headers
    meson setup "$lp_build" "$lp_src" "${lp_options[@]}" >"$deps/libplacebo-setup.log" ||
        die "libplacebo configure failed, see $deps/libplacebo-setup.log"
    ninja -C "$lp_build" install >"$deps/libplacebo-build.log" ||
        die "libplacebo build failed, see $deps/libplacebo-build.log"
    echo "$lp_key" >"$lp_stamp"
fi

export PKG_CONFIG_PATH=$lp_prefix/lib/pkgconfig

# --- Stage 3: mpv ------------------------------------------------------------

# Optional features are off so the build depends only on what meson.build
# requires unconditionally. gl defaults to enabled rather than auto, and the
# build date would make the objects differ between runs.
mpv_options=(
    -Dlibmpv=true -Ddefault_library=static -Dcplayer=false
    -Dauto_features=disabled -Dgl=disabled -Dbuild-date=false
)
mpv_stamp=$out/.setup-options
# A new libplacebo means a new configure: meson caches what it found.
mpv_key="$lp_key ${mpv_options[*]}"

if [[ ! -f $out/build.ninja || ! -f $mpv_stamp || $(cat "$mpv_stamp") != "$mpv_key" ]]; then
    echo "build-mpv: configuring mpv"
    rm -rf "$out"
    meson setup "$out" "$mpv_src" "${mpv_options[@]}" >"$deps/mpv-setup.log" ||
        die "mpv configure failed, see $deps/mpv-setup.log"
    echo "$mpv_key" >"$mpv_stamp"
fi

ninja -C "$out" --quiet libmpv.a || die "mpv build failed"

# --- Stage 4: response files -------------------------------------------------

python3 -I - "$out" "$lp_prefix/lib/pkgconfig" <<'EOF'
import json, os, re, shlex, subprocess, sys

out, lp_pc = sys.argv[1], sys.argv[2]

def quote(arg):
    return '"' + arg.replace("\\", "\\\\").replace('"', '\\"') + '"' if re.search(r'[\s"\\]', arg) else arg

entries = json.load(open(os.path.join(out, "compile_commands.json")))
entry = next(e for e in entries if e["file"].endswith("common/playlist.c"))
# Meson gives every mpv C file the same -I and -D flags, so one entry speaks
# for all of them. Its include paths are relative to the build directory.
flags = []
for a in shlex.split(entry["command"]):
    if a.startswith("-I"):
        flags.append("-I" + os.path.normpath(os.path.join(entry["directory"], a[2:])))
    elif a.startswith(("-D", "-U", "-std=")) or a == "-pthread":
        flags.append(a)

with open(os.path.join(out, "harness.rsp"), "w") as f:
    f.write("\n".join(quote(a) for a in flags) + "\n")

# libmpv's own pkg-config file lists its libraries in link order. FFmpeg and
# libass are shared, so their private dependencies are not needed.
env = dict(os.environ, PKG_CONFIG_PATH=os.pathsep.join([os.path.join(out, "meson-uninstalled"), lp_pc]))
libs = subprocess.run(["pkg-config", "--libs", "mpv-uninstalled"],
                      env=env, check=True, capture_output=True, text=True).stdout.split()
libs = [a for a in libs if a != "-lmpv" and not a.startswith("-L" + out)]
# libplacebo.pc does not record that its static archive has C++ objects.
libs.append("-lstdc++")
with open(os.path.join(out, "harness-link.rsp"), "w") as f:
    f.write("\n".join(quote(a) for a in [os.path.join(out, "libmpv.a")] + libs) + "\n")
EOF

# --- Acceptance: a harness built from the response files runs mpv code -------

accept=$out/acceptance
cat >"$accept.c" <<'EOF'
#include "common/playlist.c"

#include <stdio.h>

int main(void)
{
    struct playlist *pl = talloc_zero(NULL, struct playlist);
    playlist_append_file(pl, "a.mkv");
    int n = pl->num_entries;
    talloc_free(pl);
    printf("acceptance: playlist_append_file gives num_entries=%d\n", n);
    return n == 1 ? 0 : 1;
}
EOF
cc @"$out/harness.rsp" "$accept.c" @"$out/harness-link.rsp" -o "$accept" ||
    die "acceptance harness failed to compile or link"
"$accept" || die "acceptance harness failed"
