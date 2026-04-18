use crate::rpc::{apply_rpc_full, start_rpc, RpcState};
use tauri::Manager;

#[tauri::command]
pub async fn set_rpc(
    app: tauri::AppHandle,
    details: String,
    state_text: String,
) -> Result<(), String> {
    let rpc_state = app.state::<RpcState>();

    let needs_start = rpc_state.client.lock().await.is_none();

    if needs_start {
        start_rpc(&rpc_state, "1484521125550620813")
            .await
            .map_err(|e| format!("Failed to connect to Discord RPC: {e}"))?;
    }

    let result = apply_rpc_full(
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
    .await;

    if result.is_err() {
        *rpc_state.client.lock().await = None;
        *rpc_state.runner.lock().await = None;

        start_rpc(&rpc_state, "1484521125550620813")
            .await
            .map_err(|e| format!("Failed to reconnect to Discord RPC: {e}"))?;

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
    } else {
        Ok(())
    }
}