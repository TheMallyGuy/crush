use crate::rpc::{apply_rpc, RpcState};
use tauri::Manager;

#[tauri::command]
pub async fn set_rpc(
    app: tauri::AppHandle,
    details: String,
    state_text: String,
) -> Result<(), String> {
    let rpc_state = app.state::<RpcState>();
    apply_rpc(&rpc_state, &details, &state_text).await
}
