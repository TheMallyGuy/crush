// roblox downloader i guess

use futures::future::join_all;
use reqwest;
use serde::Deserialize;
use std::sync::OnceLock;
use std::time::Instant;

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

    let Ok(_) = client.head(url).send().await else {
        log::error!("[BACKEND] {} failed to respond", url);
        return (url, u128::MAX);
    };

    let duration = start.elapsed().as_millis();
    log::info!("[BACKEND] {} returned in {}ms", url, duration);

    (url, duration)
}

pub async fn best_region() -> Option<&'static str> {
    let client = get_client();
    log::info!("[BACKEND] testing for best regions");

    let results = join_all(URLS.iter().map(|&url| ping_url(client, url))).await;

    let fastest = results
        .into_iter()
        .filter(|&(_, time)| time != u128::MAX)
        .min_by_key(|&(_, time)| time)
        .map(|(url, _)| url);

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
    version_hash: Option<&str>,
    region_url: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let latest = latest_version().await?;
    let raw_hash = version_hash.unwrap_or(&latest.client_version_upload);

    let base_version = format!(
        "version-{}",
        raw_hash.strip_prefix("version-").unwrap_or(raw_hash)
    );

    let base_url = match region_url {
        Some(url) => url.to_owned(),
        None => best_region()
            .await
            .unwrap_or("https://setup.rbxcdn.com")
            .to_owned(),
    };

    let urls = FILES
        .iter()
        .map(|file| format!("{base_url}/{base_version}-{file}"))
        .collect();

    Ok(urls)
}
