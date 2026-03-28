// roblox downloader i guess

use futures::future::join_all;
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::Instant;
use tauri_plugin_http::reqwest;

pub fn get_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(reqwest::Client::new)
}

#[derive(Deserialize)]
pub struct LatestVersion {
    #[serde(rename = "clientVersionUpload")]
    pub client_version_upload: String,
}

const URLS: &[&str] = &[
    "https://setup-aws.rbxcdn.com", // fallback!! only use this if none of the urls are functioning
    "https://setup-ak.rbxcdn.com",  // asia, eu, region (akamai best)
    "https://setup-cfly.rbxcdn.com", // us region
];

const FILES: &[&str] = &[
    "RobloxApp.zip",
    "Redist",
    "WebView2RuntimeInstaller.zip",
    "content-avatar.zip",
    "shaders.zip",
    "ssl.zip",
    "WebView2.zip",
    "content-avatar.zip",
    "content-configs.zip",
    "content-fonts.zip",
    "content-sky.zip",
    "content-sounds.zip",
    "content-textures2.zip",
    "content-models.zip",
    "content-platform-fonts.zip",
    "content-platform-dictionaries.zip",
    "content-terrain.zip",
    "content-textures3.zip",
    "extracontent-luapackages.zip",
    "extracontent-translations.zip",
    "extracontent-models.zip",
    "extracontent-textures.zip",
    "extracontent-places.zip",
];

async fn ping_url(client: &reqwest::Client, url: &'static str) -> (&'static str, u128) {
    log::info!("[BACKEND] testing : {}", url);
    let start = Instant::now();

    let res = client.head(url).send().await;
    let duration = start.elapsed().as_millis();

    log::info!("[BACKEND] {} returned in {}ms", url, duration);

    match res {
        Ok(_) => (url, duration),
        Err(_) => (url, u128::MAX),
    }
}

pub async fn best_region() -> Option<&'static str> {
    let client = get_client();
    log::info!("[BACKEND] testing for best regions");

    let futures = URLS.iter().map(|&url| ping_url(client, url));

    let mut results = join_all(futures).await;
    results.sort_by_key(|&(_, time)| time);

    let fastest = results.first().map(|(url, _)| *url);

    log::info!("[BACKEND] best url: {:?}", fastest);

    fastest
}

pub async fn latest_version() -> Result<LatestVersion, Box<dyn std::error::Error>> {
    let client = get_client();

    let text = client
        .get("https://clientsettings.roblox.com/v2/client-version/WindowsPlayer")
        .send()
        .await?
        .text()
        .await?;

    let parsed: LatestVersion = serde_json::from_str(&text)?;

    Ok(parsed)
}

pub async fn get_download_urls(
    versionhash: Option<&str>,
    region_url: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let version = latest_version().await?;

    let raw_version = versionhash.unwrap_or(&version.client_version_upload);
    let base_version = if raw_version.starts_with("version-") {
        raw_version.to_string()
    } else {
        format!("version-{}", raw_version)
    };

    let base = match region_url {
        Some(r) => r.to_string(),
        None => best_region()
            .await
            .unwrap_or("https://setup.rbxcdn.com")
            .to_string(),
    };

    let urls = FILES
        .iter()
        .map(|file| format!("{}/{}-{}", base, base_version, file))
        .collect();

    Ok(urls)
}
