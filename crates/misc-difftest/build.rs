//! Compiles the C shims against mpv's headers and links the tests with the
//! real static build of mpv in `formal/.mpv-build`.
//!
//! Without that build the crate still compiles, but without `cfg(mpv_build)`,
//! so it holds no tests. Set `RMPV_REQUIRE_MPV_BUILD=1` to fail instead.

use std::path::{Path, PathBuf};
use std::process::Command;

const SHIMS: [&str; 9] = [
    "inline_shim",
    "interpose",
    "static_bstr",
    "static_charset_conv",
    "static_codepoint_width",
    "static_json",
    "static_language",
    "static_path_utils",
    "static_random",
];

fn run(cmd: &mut Command) {
    let status = cmd
        .status()
        .unwrap_or_else(|e| panic!("cannot run {cmd:?}: {e}"));
    assert!(status.success(), "{cmd:?} failed");
}

fn main() {
    println!("cargo::rustc-check-cfg=cfg(mpv_build)");
    println!("cargo::rerun-if-env-changed=RMPV_MPV_BUILD");
    println!("cargo::rerun-if-env-changed=RMPV_REQUIRE_MPV_BUILD");
    println!("cargo::rerun-if-changed=shim");

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let build = std::env::var("RMPV_MPV_BUILD")
        .map_or_else(|_| manifest.join("../../formal/.mpv-build"), PathBuf::from);
    let libmpv = build.join("libmpv.a");
    println!("cargo::rerun-if-changed={}", libmpv.display());
    if !libmpv.exists() {
        assert!(
            std::env::var_os("RMPV_REQUIRE_MPV_BUILD").is_none(),
            "{} is missing: run `make mpv-build`",
            libmpv.display()
        );
        println!(
            "cargo::warning=no mpv build at {}: the differential tests are not compiled",
            build.display()
        );
        return;
    }

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("set by cargo"));
    let shim = manifest.join("shim");
    let mut objects = Vec::new();
    for name in SHIMS {
        let object = out.join(format!("{name}.o"));
        run(Command::new("cc")
            .arg(format!("@{}", build.join("harness.rsp").display()))
            .arg("-w")
            .arg("-I")
            .arg(&shim)
            .arg("-c")
            .arg(shim.join(format!("{name}.c")))
            .arg("-o")
            .arg(&object));
        if name.starts_with("static_") {
            // The object holds a second copy of everything its mpv source
            // file defines. Only the dt_* entry points may be seen by the
            // linker, so that every other call still reaches libmpv.a.
            run(Command::new("objcopy")
                .args(["-w", "-G", "dt_*"])
                .arg(&object));
        }
        objects.push(object);
    }
    let archive = out.join("libdtshim.a");
    let _ = std::fs::remove_file(&archive);
    run(Command::new("ar").arg("crs").arg(&archive).args(&objects));

    let link_rsp = std::fs::read_to_string(build.join("harness-link.rsp"))
        .unwrap_or_else(|e| panic!("cannot read harness-link.rsp: {e}"));
    println!("cargo::rustc-link-arg-tests=-Wl,--start-group");
    println!("cargo::rustc-link-arg-tests={}", archive.display());
    for arg in link_rsp.split_whitespace() {
        if Path::new(arg).extension().is_some_and(|e| e == "a") {
            println!("cargo::rustc-link-arg-tests={arg}");
        }
    }
    println!("cargo::rustc-link-arg-tests=-Wl,--end-group");
    for arg in link_rsp.split_whitespace() {
        if Path::new(arg).extension().is_none_or(|e| e != "a") {
            println!("cargo::rustc-link-arg-tests={arg}");
        }
    }
    println!("cargo::rustc-cfg=mpv_build");
}
