use filthy_rich::{Activity, DiscordIPC};
use tokio::sync::Mutex;

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
    let Some(client) = lock.as_ref() else {
        return Err("RPC not initialized".into());
    };

    let activity = Activity::new().details(details).state(state_text).build();
    client
        .set_activity(activity)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn kill_rpc(client_lock: &Mutex<Option<DiscordIPC>>) -> Result<(), String> {
    let mut lock = client_lock.lock().await;

    let Some(mut client) = lock.take() else {
        return Err("RPC not running".into());
    };

    let _ = client.clear_activity().await;

    // DiscordIPC::close() is async and returns a Result.
    if let Err(e) = client.close().await {
        log::error!("Failed to close Discord RPC: {:?}", e);
    }

    Ok(())
}
