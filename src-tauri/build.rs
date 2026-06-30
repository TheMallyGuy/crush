use anyhow::Result;
use vergen::{Build, Emitter, Rustc};

fn main() -> Result<()> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let libraries_dir = manifest_dir.join("libraries");
    let dll_src = libraries_dir.join("swifttunnel.dll");

    println!("cargo:rustc-link-search=native={}", libraries_dir.display());
    println!("cargo:rustc-link-lib=dylib=swifttunnel");
    println!("cargo:rustc-link-arg=/DELAYLOAD:swifttunnel.dll");
    println!("cargo:rustc-link-lib=delayimp");
    println!("cargo:rerun-if-changed={}", dll_src.display());

    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    let target_dir = std::path::PathBuf::from(&out_dir)
        .ancestors()
        .find(|p| p.ends_with("debug") || p.ends_with("release"))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("target/debug"));

    let _ = std::fs::copy(&dll_src, target_dir.join("swifttunnel.dll"));

    let build = Build::all_build();
    let rustc = Rustc::all_rustc();

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&rustc)?
        .emit()?;

    tauri_build::build();

    Ok(())
}
