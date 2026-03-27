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