use crate::rpc::{apply_rpc, RpcState};
use tauri::State;

#[tauri::command]
pub async fn set_rpc(
    state: State<'_, RpcState>,
    details: String,
    state_text: String,
) -> Result<(), String> {
    apply_rpc(&state.client, &details, &state_text).await
}
