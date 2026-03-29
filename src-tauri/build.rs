use vergen::{BuildBuilder, RustcBuilder, Emitter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rustc = RustcBuilder::all_rustc()?;
    let build = BuildBuilder::all_build()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&rustc)?
        .emit()?;

    tauri_build::build();

    Ok(())
}