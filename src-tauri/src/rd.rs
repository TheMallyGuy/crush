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

// https://github.com/latte-soft/rdd/blob/master/src/js/rdd.js

const PLAYER_FILES: &[&str] = &[
    "RobloxApp.zip",
    "WebView2RuntimeInstaller.zip",
    "content-avatar.zip",
    "shaders.zip",
    "ssl.zip",
    "WebView2.zip",
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

const STUDIO_FILES: &[&str] = &[
    "RobloxStudio.zip",
    "redist.zip",
    "Libraries.zip",
    "LibrariesQt5.zip",
    "WebView2.zip",
    "WebView2RuntimeInstaller.zip",
    "shaders.zip",
    "ssl.zip",
    "Plugins.zip",
    "StudioFonts.zip",
    "BuiltInPlugins.zip",
    "ApplicationConfig.zip",
    "BuiltInStandalonePlugins.zip",
    "content-qt_translations.zip",
    "content-sky.zip",
    "content-fonts.zip",
    "content-avatar.zip",
    "content-models.zip",
    "content-sounds.zip",
    "content-configs.zip",
    "content-api-docs.zip",
    "content-textures2.zip",
    "content-studio_svg_textures.zip",
    "content-platform-fonts.zip",
    "content-platform-dictionaries.zip",
    "content-terrain.zip",
    "content-textures3.zip",
    "extracontent-translations.zip",
    "extracontent-luapackages.zip",
    "extracontent-textures.zip",
    "extracontent-scripts.zip",
    "extracontent-models.zip",
    "studiocontent-models.zip",
    "studiocontent-textures.zip",
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

    let results = join_all(URLS.iter().map(|&url| ping_url(client, url))).await;

    let fastest = results
        .into_iter()
        .filter(|&(_, time)| time != u128::MAX)
        .min_by_key(|&(_, time)| time)
        .map(|(url, _)| url);

    log::info!("[BACKEND] best url: {:?}", fastest);

    fastest
}

pub async fn latest_version_player() -> Result<LatestVersion, Box<dyn std::error::Error>> {
    let latest: LatestVersion = get_client()
        .get("https://clientsettings.roblox.com/v2/client-version/WindowsPlayer")
        .send()
        .await?
        .json()
        .await?;

    Ok(latest)
}

pub async fn latest_version_studio() -> Result<LatestVersion, Box<dyn std::error::Error>> {
    let latest: LatestVersion = get_client()
        .get("https://clientsettings.roblox.com/v2/client-version/WindowsStudio64")
        .send()
        .await?
        .json()
        .await?;

    Ok(latest)
}

pub async fn get_download_urls(
    is_player: bool,
    version_hash: Option<&str>,
    region_url: Option<&str>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let raw_hash: String = match version_hash {
        Some(hash) => hash.to_string(),
        None => {
            if is_player {
                latest_version_player().await?.client_version_upload
            } else {
                latest_version_studio().await?.client_version_upload
            }
        }
    };

    let base_version = format!(
        "version-{}",
        raw_hash.strip_prefix("version-").unwrap_or(&raw_hash)
    );

    let base_url = match region_url {
        Some(url) => url.to_string(),
        None => best_region()
            .await
            .unwrap_or("https://setup.rbxcdn.com")
            .to_string(),
    };

    let urls: Vec<String> = if is_player {
        PLAYER_FILES
            .iter()
            .map(|file| format!("{base_url}/{base_version}-{file}"))
            .collect()
    } else {
        STUDIO_FILES
            .iter()
            .map(|file| format!("{base_url}/{base_version}-{file}"))
            .collect()
    };

    Ok(urls)
}