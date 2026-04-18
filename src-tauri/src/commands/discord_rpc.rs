use crate::rpc::{apply_rpc_full, start_rpc,RpcState};
use tauri::Manager;

#[tauri::command]
pub async fn set_rpc(
    app: tauri::AppHandle,
    details: String,
    state_text: String,
) -> Result<(), String> {
    let rpc_state = app.state::<RpcState>();
    let _ = start_rpc(&rpc_state, "1484521125550620813").await;
    apply_rpc_full(
        &rpc_state,
        None,
        Some(&details),
        Some(&state_text),
        None,
        None,
        Some(vec![
            ("View repo".to_string(), "https://github.com/TheMallyGuy/crush".to_string())
        ]),
    ).await
}