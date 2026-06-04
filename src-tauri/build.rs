use vergen::{BuildBuilder, Emitter, RustcBuilder};
// i shaw manifest my crush!
const APP_MANIFEST: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rustc-link-search=native=libraries");
    println!("cargo:rustc-link-lib=dylib=swifttunnel");
    println!("cargo:rerun-if-changed=libraries/swifttunnel.dll");

    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    let target_dir = std::path::PathBuf::from(&out_dir)
        .ancestors()
        .find(|p| p.ends_with("debug") || p.ends_with("release"))
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("target/debug"));

    let _ = std::fs::copy("libraries/swifttunnel.dll", target_dir.join("swifttunnel.dll"));

    let rustc = RustcBuilder::all_rustc()?;
    let build = BuildBuilder::all_build()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&rustc)?
        .emit()?;

    let attrs = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new().app_manifest(APP_MANIFEST));
    tauri_build::try_build(attrs).expect("Failed to run tauri-build");

    Ok(())
}