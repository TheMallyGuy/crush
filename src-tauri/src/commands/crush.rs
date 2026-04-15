#[derive(serde::Serialize)]
pub struct BuildInfo {
    hash: String,
    build_date: String,
    version: String,
}

#[tauri::command]
pub async fn crush() -> Result<BuildInfo, String> {
    Ok(BuildInfo {
        hash: env!("VERGEN_RUSTC_COMMIT_HASH").to_string(),
        build_date: env!("VERGEN_BUILD_DATE").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
