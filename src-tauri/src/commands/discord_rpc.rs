use crate::rpc::{apply_rpc_full, RpcState};
use tauri::Manager;

#[tauri::command]
pub async fn set_rpc(
    app: tauri::AppHandle,
    details: String,
    state_text: String,
) -> Result<(), String> {
    let rpc_state = app.state::<RpcState>();
    apply_rpc_full(
        &rpc_state,
        None,
        Some(&details),
        Some(&state_text),
        None,
        None,
        Some(vec![(
            "View repo".to_string(),
            "https://github.com/TheMallyGuy/crush".to_string(),
        )]),
    )
    .await
}
