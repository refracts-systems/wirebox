use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=WIREBOX_ZIG");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let source = root.join("kernel/src/abi.zig");
    println!(
        "cargo:rerun-if-changed={}",
        root.join("kernel/src").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        root.join("kernel/include").display()
    );
    let target = env::var("TARGET").unwrap();
    assert_eq!(
        target, "x86_64-pc-windows-msvc",
        "Phase 1 supports Windows x64 MSVC only"
    );

    let zig = env::var_os("WIREBOX_ZIG").unwrap_or_else(|| {
        let local = PathBuf::from(env::var_os("LOCALAPPDATA").unwrap_or_default())
            .join("Programs/Zig/zig-x86_64-windows-0.17.0/zig.exe");
        if local.is_file() {
            local.into_os_string()
        } else {
            "zig".into()
        }
    });
    let version = Command::new(&zig)
        .arg("version")
        .output()
        .expect("Install Zig 0.17.0 or set WIREBOX_ZIG");
    assert!(version.status.success(), "zig version failed");
    let version = String::from_utf8(version.stdout).unwrap();
    // The project minimum is 0.17; this build baseline has been validated on 0.17.0.
    assert_eq!(
        version.trim(),
        "0.17.0",
        "Baseline build requires Zig 0.17.0; validate a newer compiler before changing the pin"
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let status = Command::new(zig)
        .arg("build-lib")
        .arg(&source)
        .args(["-target", "x86_64-windows-msvc", "-O", "Debug", "-static"])
        .arg(format!(
            "-femit-bin={}",
            out.join("wirebox_kernel.lib").display()
        ))
        .arg("--cache-dir")
        .arg(out.join("zig-cache"))
        .status()
        .expect("failed to start Zig");
    assert!(status.success(), "Zig static library build failed");
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=wirebox_kernel");
    println!("cargo:rustc-env=WIREBOX_ZIG_VERSION={}", version.trim());
}
