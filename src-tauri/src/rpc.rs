use filthy_rich::{Activity, DiscordIPCClient, DiscordIPCRunner};
use tokio::sync::Mutex;

pub struct RpcState {
    pub runner: Mutex<Option<DiscordIPCRunner>>,
    pub client: Mutex<Option<DiscordIPCClient>>,
}

impl RpcState {
    pub fn new() -> Self {
        Self {
            runner: Mutex::new(None),
            client: Mutex::new(None),
        }
    }
}

impl Default for RpcState {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn start_rpc(state: &RpcState, client_id: &str) -> Result<(), String> {
    let mut runner = DiscordIPCRunner::new(client_id);

    let client = runner
        .run(true) // wait_for_ready = true
        .await
        .map_err(|e| e.to_string())?
        .clone();

    *state.client.lock().await = Some(client);
    *state.runner.lock().await = Some(runner);

    Ok(())
}

pub async fn apply_rpc(
    state: &RpcState,
    details: &str,
    state_text: &str,
) -> Result<(), String> {
    let lock = state.client.lock().await;
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

pub async fn kill_rpc(state: &RpcState) -> Result<(), String> {
    let client = state.client.lock().await.take();

    if let Some(client) = client {
        // Send close command through the client
        client.close().await.map_err(|e| e.to_string())?;
    }

    if let Some(mut runner) = state.runner.lock().await.take() {
        runner.wait().await.map_err(|e| e.to_string())?;
    }

    Ok(())
}