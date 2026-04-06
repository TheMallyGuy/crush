use filthy_rich::{Activity, DiscordIPC};
use tokio::sync::Mutex;

// @pochita please add a new function that reconnect rpc and one for adding a custom button (if no button is added, its will not show buttons) 

pub struct RpcState {
    pub client: Mutex<Option<DiscordIPC>>,
}

impl RpcState {
    pub fn new() -> Self {
        Self {
            client: Mutex::new(None),
        }
    }
}

impl Default for RpcState {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn apply_rpc(
    client_lock: &Mutex<Option<DiscordIPC>>,
    details: &str,
    state_text: &str,
) -> Result<(), String> {
    let lock = client_lock.lock().await;
    if let Some(client) = lock.as_ref() {
        let activity = Activity::new().details(details).state(state_text).build();
        client
            .set_activity(activity)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("RPC not initialized".into())
    }
}

pub async fn kill_rpc(
    client_lock: &tokio::sync::Mutex<Option<filthy_rich::DiscordIPC>>,
) -> Result<(), String> {
    let mut lock = client_lock.lock().await;

    if let Some(mut client) = lock.take() {
        let _ = client.clear_activity().await;

        let _ = client.close().await;

        Ok(())
    } else {
        Err("RPC not running".into())
    }
}
