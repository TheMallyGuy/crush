use super::types::{GamesResponse, IconResponse, ThumbnailResponse, UniverseResponse};
use crate::rd::get_client;
use std::time::Duration;

pub(super) async fn fetch_universe_id(place_id: u64) -> Result<u64, String> {
    let res = tokio::time::timeout(
        Duration::from_secs(5),
        get_client()
            .get(format!(
                "https://apis.roblox.com/universes/v1/places/{}/universe",
                place_id
            ))
            .send(),
    )
    .await;

    let response = match res {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Err(e.to_string()),
        Err(_) => return Err("apis.roblox.com request timed out".to_string()),
    };

    let universe: UniverseResponse = tokio::time::timeout(Duration::from_secs(5), response.json())
        .await
        .map_err(|_| "apis.roblox.com json parse timed out".to_string())?
        .map_err(|e| e.to_string())?;

    Ok(universe.universe_id)
}

pub(super) async fn fetch_place_info(place_id: u64) -> Result<Option<(String, String)>, String> {
    let client = get_client();

    let universe_id = fetch_universe_id(place_id).await?;

    let (games_res, icon_res) = tokio::join!(
        tokio::time::timeout(
            Duration::from_secs(5),
            client
                .get(format!(
                    "https://games.roblox.com/v1/games?universeIds={}",
                    universe_id
                ))
                .send()
        ),
        tokio::time::timeout(
            Duration::from_secs(5),
            client
                .get(format!(
                    "https://thumbnails.roblox.com/v1/games/icons?universeIds={}&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false",
                    universe_id
                ))
                .send()
        ),
    );

    let games_response = games_res
        .map_err(|_| "games.roblox.com request timed out".to_string())?
        .map_err(|e| e.to_string())?;

    let icon_response = icon_res
        .map_err(|_| "thumbnails.roblox.com request timed out".to_string())?
        .map_err(|e| e.to_string())?;

    let (games_data, icon_data) = tokio::join!(
        tokio::time::timeout(
            Duration::from_secs(5),
            games_response.json::<GamesResponse>()
        ),
        tokio::time::timeout(Duration::from_secs(5), icon_response.json::<IconResponse>()),
    );

    let name = games_data
        .map_err(|_| "games.roblox.com json parse timed out".to_string())?
        .map_err(|e| e.to_string())?
        .data
        .into_iter()
        .next()
        .map(|g| g.name)
        .unwrap_or_else(|| "Unknown Game".to_string());

    let image_url = icon_data
        .map_err(|_| "thumbnails.roblox.com json parse timed out".to_string())?
        .map_err(|e| e.to_string())?
        .data
        .into_iter()
        .next()
        .map(|i| i.image_url)
        .unwrap_or_default();

    Ok(Some((name, image_url)))
}

/// Fetch the wide promotional game thumbnail (not the square icon) for a place.
/// Returns `None` if the universe has no completed thumbnail.
pub(super) async fn fetch_game_thumbnail(place_id: u64) -> Result<Option<String>, String> {
    let universe_id = fetch_universe_id(place_id).await?;

    let res = tokio::time::timeout(
        Duration::from_secs(5),
        get_client()
            .get(format!(
                "https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds={}&countPerUniverse=1&defaults=true&size=768x432&format=Png&isCircular=false",
                universe_id
            ))
            .send(),
    )
    .await
    .map_err(|_| "thumbnails.roblox.com request timed out".to_string())?
    .map_err(|e| e.to_string())?;

    let thumbnails: ThumbnailResponse = tokio::time::timeout(Duration::from_secs(5), res.json())
        .await
        .map_err(|_| "thumbnails.roblox.com json parse timed out".to_string())?
        .map_err(|e| e.to_string())?;

    let image_url = thumbnails
        .data
        .into_iter()
        .next()
        .and_then(|entry| entry.thumbnails.into_iter().next())
        .and_then(|thumb| thumb.image_url);

    Ok(image_url)
}

pub(super) async fn fetch_game_icon(place_id: u64) -> Result<Option<String>, String> {
    let universe_id = fetch_universe_id(place_id).await?;

    let res = tokio::time::timeout(
        Duration::from_secs(5),
        get_client()
            .get(format!(
                "https://thumbnails.roblox.com/v1/games/icons?universeIds={}&returnPolicy=PlaceHolder&size=512x512&format=Png&isCircular=false",
                universe_id
            ))
            .send(),
    )
    .await
    .map_err(|_| "thumbnails.roblox.com request timed out".to_string())?
    .map_err(|e| e.to_string())?;

    let icons: IconResponse = tokio::time::timeout(Duration::from_secs(5), res.json())
        .await
        .map_err(|_| "thumbnails.roblox.com json parse timed out".to_string())?
        .map_err(|e| e.to_string())?;

    let image_url = icons
        .data
        .into_iter()
        .next()
        .map(|icon| icon.image_url)
        .filter(|url| !url.is_empty());

    Ok(image_url)
}
