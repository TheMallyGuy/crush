use tauri::State;
use crate::rpc::RpcState;
use filthy_rich::Activity;

#[tauri::command]
pub async fn set_rpc(
    state: State<'_, RpcState>,
    details: String,
    state_text: String,
) -> Result<(), String> {
    let lock = state.client.lock().await;

    if let Some(client) = lock.as_ref() {
        let activity = Activity::new()
            .details(&details)
            .state(&state_text)
            .build();

        client
            .set_activity(activity)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    } else {
        Err("RPC not initialized".into())
    }
}