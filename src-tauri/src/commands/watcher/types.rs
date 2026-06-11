use serde::{Deserialize, Serialize};
use serde_json::Value;

// API types

#[derive(Deserialize)]
pub(super) struct UniverseResponse {
    #[serde(alias = "universeId")]
    pub(super) universe_id: u64,
}

#[derive(Deserialize)]
pub(super) struct GameData {
    pub(super) name: String,
}

#[derive(Deserialize)]
pub(super) struct GamesResponse {
    pub(super) data: Vec<GameData>,
}

#[derive(Deserialize)]
pub(super) struct IpInfo {
    pub(super) city: Option<String>,
    pub(super) region: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct IconEntry {
    #[serde(rename = "imageUrl")]
    pub(super) image_url: String,
}

#[derive(Deserialize)]
pub(super) struct IconResponse {
    pub(super) data: Vec<IconEntry>,
}

#[derive(Deserialize)]
pub(super) struct ThumbnailImage {
    #[serde(rename = "imageUrl")]
    pub(super) image_url: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct ThumbnailEntry {
    pub(super) thumbnails: Vec<ThumbnailImage>,
}

#[derive(Deserialize)]
pub(super) struct ThumbnailResponse {
    pub(super) data: Vec<ThumbnailEntry>,
}

// bloxstrap rpc types

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RichPresence {
    pub(super) details: Option<String>,
    pub(super) state: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct BloxstrapRpcMessage {
    pub(super) command: String,
    #[serde(default)]
    pub(super) data: Value,
}

// emit data type

#[derive(Serialize, Deserialize, Clone)]
pub(super) struct EmitServerInfomation {
    pub(super) server_id: String,
    pub(super) game_id: u64,
    pub(super) region_info: String,
}
