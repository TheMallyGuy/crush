use filthy_rich::{
    types::{Activity, ActivityType, StatusDisplayType},
    PresenceClient, PresenceRunner,
};
use tokio::sync::Mutex;

pub struct RpcState {
    pub runner: Mutex<Option<PresenceRunner>>,
    pub client: Mutex<Option<PresenceClient>>,
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
    let mut runner = PresenceRunner::new(client_id);
    runner.run(true).await.map_err(|e| e.to_string())?;
    let client = runner.clone_handle();
    *state.client.lock().await = Some(client);
    *state.runner.lock().await = Some(runner);
    Ok(())
}

pub async fn apply_rpc(state: &RpcState, details: &str, state_text: &str) -> Result<(), String> {
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

pub async fn apply_rpc_full(
    state: &RpcState,
    name: Option<&str>,
    details: Option<&str>,
    state_text: Option<&str>,
    activity_type: Option<ActivityType>,
    status_display_type: Option<StatusDisplayType>,
    buttons: Option<Vec<(String, String)>>,
) -> Result<(), String> {
    if let Some(ref btns) = buttons {
        if btns.len() > 2 {
            return Err("Discord allows a maximum of 2 buttons per activity".into());
        }
    }

    let lock = state.client.lock().await;
    if let Some(client) = lock.as_ref() {
        let mut builder = Activity::new();

        if let Some(n) = name {
            builder = builder.name(n);
        }
        if let Some(d) = details {
            builder = builder.details(d);
        }
        if let Some(s) = state_text {
            builder = builder.state(s);
        }
        if let Some(t) = activity_type {
            builder = builder.activity_type(t);
        }
        if let Some(dt) = status_display_type {
            builder = builder.status_display_type(dt);
        }
        if let Some(btns) = buttons {
            for (label, url) in btns {
                builder = builder.add_button(label, url);
            }
        }

        client
            .set_activity(builder.build())
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("RPC not initialized".into())
    }
}

pub async fn set_name(state: &RpcState, name: &str) -> Result<(), String> {
    let lock = state.client.lock().await;
    if let Some(client) = lock.as_ref() {
        let activity = Activity::new().name(name).build();
        client
            .set_activity(activity)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("RPC not initialized".into())
    }
}

pub async fn set_activity_type(
    state: &RpcState,
    activity_type: ActivityType,
) -> Result<(), String> {
    let lock = state.client.lock().await;
    if let Some(client) = lock.as_ref() {
        let activity = Activity::new().activity_type(activity_type).build();
        client
            .set_activity(activity)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("RPC not initialized".into())
    }
}

pub async fn set_status_display_type(
    state: &RpcState,
    display_type: StatusDisplayType,
) -> Result<(), String> {
    let lock = state.client.lock().await;
    if let Some(client) = lock.as_ref() {
        let activity = Activity::new().status_display_type(display_type).build();
        client
            .set_activity(activity)
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("RPC not initialized".into())
    }
}

pub async fn set_buttons(state: &RpcState, buttons: Vec<(&str, &str)>) -> Result<(), String> {
    if buttons.len() > 2 {
        return Err("Discord allows a maximum of 2 buttons per activity".into());
    }
    let lock = state.client.lock().await;
    if let Some(client) = lock.as_ref() {
        let mut builder = Activity::new();
        for (label, url) in buttons {
            builder = builder.add_button(label, url);
        }
        client
            .set_activity(builder.build())
            .await
            .map_err(|e| e.to_string())
    } else {
        Err("RPC not initialized".into())
    }
}

pub async fn clear_rpc(state: &RpcState) -> Result<(), String> {
    let lock = state.client.lock().await;
    if let Some(client) = lock.as_ref() {
        client.clear_activity().await.map_err(|e| e.to_string())
    } else {
        Err("RPC not initialized".into())
    }
}

pub async fn kill_rpc(state: &RpcState) -> Result<(), String> {
    if let Some(client) = state.client.lock().await.take() {
        client.close().await.map_err(|e| e.to_string())?;
    }
    if let Some(mut runner) = state.runner.lock().await.take() {
        runner.wait().await.map_err(|e| e.to_string())?;
    }
    Ok(())
}
