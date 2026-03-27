use tauri::State;
use crate::rpc::{RpcState, apply_rpc};

#[tauri::command]
pub async fn set_rpc(
    state: State<'_, RpcState>,
    details: String,
    state_text: String,
) -> Result<(), String> {
    apply_rpc(&state.client, &details, &state_text).await
}