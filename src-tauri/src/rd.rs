// roblox downloader i guess

use tauri_plugin_http::reqwest;
use std::time::Instant;
use futures::future::join_all;
use serde::Deserialize;

#[derive(Deserialize)]
struct LatestVersion {

    #[serde(rename = "clientVersionUpload")]
    client_version_upload: String,
}

const URLS: &[&str] = &[
    "https://setup-aws.rbxcdn.com", // fallback!! only use this if none of the urls are functioning
    "https://setup-ak.rbxcdn.com", // asia, eu, region (akamai best)
    "https://setup-cfly.rbxcdn.com" // us region
    ];

const FILES: &[&str] = &[
    "RobloxApp.zip",
    "Redist",
    "WebView2RuntimeInstaller.zip",
    "content-avatar.zip", 
    "shaders.zip",
    "ssl.zip",

    "WebView2.zip",
    "WebView2RuntimeInstaller.zip",

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
    "extracontent-places.zip"
    ];

pub async fn best_region() -> Option<&'static str> {
    let client = reqwest::Client::new();
    log::info!("[BACKEND] testing for best regions");

    let futures = URLS.iter().map(|&url| {
        let client = client.clone();

        async move {
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
    });

    let mut results = join_all(futures).await;
    results.sort_by_key(|&(_, time)| time);

    let fastest = results.first().map(|(url, _)| *url);

    log::info!("[BACKEND] best url: {:?}", fastest);

    fastest
}

async fn latest_version() -> Result<LatestVersion, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

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

    let base_version = versionhash
        .map(|v| v.to_string())
        .unwrap_or(version.client_version_upload);

    let base_version = if base_version.starts_with("version-") {
        base_version
    } else {
        format!("version-{}", base_version)
    };

    let base = match region_url {
        Some(r) => r.to_string(),
        None => {
            best_region()
                .await
                .unwrap_or("https://setup.rbxcdn.com")
                .to_string()
        }
    };

    let urls: Vec<String> = FILES
        .iter()
        .map(|file| format!("{}/{}-{}", base, base_version, file))
        .collect();

    Ok(urls)
}