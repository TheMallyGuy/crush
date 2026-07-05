use anyhow::Result;
use vergen::{Build, Emitter, Rustc};

fn main() -> Result<()> {
    let build = Build::all_build();
    let rustc = Rustc::all_rustc();

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&rustc)?
        .emit()?;

    tauri_build::build();

    Ok(())
}
