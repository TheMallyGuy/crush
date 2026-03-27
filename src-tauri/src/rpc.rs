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

pub async fn apply_rpc(
    client_lock: &Mutex<Option<DiscordIPC>>,
    details: &str,
    state_text: &str,
) -> Result<(), String> {
    let lock = client_lock.lock().await;
    if let Some(client) = lock.as_ref() {
        let activity = Activity::new()
            .details(details)
            .state(state_text)
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