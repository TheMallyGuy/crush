use super::api::{fetch_game_icon, fetch_game_thumbnail, fetch_place_info};
use super::config::integration_enabled;
use super::state::WatcherState;
use super::types::{EmitServerInfomation, IpInfo};
use crate::island::{show, show_with_image};
use crate::rd::get_client;
use crate::rpc::{apply_rpc_full, start_rpc, RpcState};
use crate::simple_i18n::I18n;
use crate::t;
use chrono::{Local, Timelike};
use serde_json::json;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

pub(super) fn emit_server_info(
    app: &AppHandle,
    instance_id: &str,
    game_id: u64,
    region_info: &str,
) {
    log::info!("{}", region_info);
    let payload = EmitServerInfomation {
        server_id: instance_id.to_string(),
        game_id,
        region_info: region_info.to_string(),
    };
    app.emit("serverInfomation", payload).unwrap();
}

pub(super) async fn fetch_and_store_location(
    ip: &str,
    state: &mut WatcherState,
    app: &AppHandle,
) -> Result<(), String> {
    if state.activity.place_id.is_none() {
        log::info!("UDMUX fired but no place_id, skipping");
        return Ok(());
    }
    log::info!("UDMUX IP: {}", ip);

    let res = tokio::time::timeout(
        Duration::from_secs(5),
        get_client()
            .get(format!("https://get.geojs.io/v1/ip/geo/{}.json", ip))
            .send(),
    )
    .await;

    let response = match res {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => return Err(e.to_string()),
        Err(_) => return Err("geojs.io request timed out".to_string()),
    };

    let info: IpInfo = tokio::time::timeout(Duration::from_secs(5), response.json())
        .await
        .map_err(|_| "geojs.io json parse timed out".to_string())?
        .map_err(|e| e.to_string())?;

    state.pending_server_ip = Some(ip.to_string());
    state.pending_server_location = Some(format!(
        "{}, {}",
        info.city.as_deref().unwrap_or("Unknown City"),
        info.region.as_deref().unwrap_or("Unknown Region")
    ));

    if state.activity.in_game {
        if let (Some(id), Some(loc)) = (
            state.activity.instance_id.as_deref(),
            state.pending_server_location.as_deref(),
        ) {
            if let Some(place_id) = state.activity.place_id {
                emit_server_info(app, id, place_id, loc);
            }
        }
    }
    Ok(())
}

pub(super) async fn sleep_schedule_inner(
    app: &AppHandle,
    in_game: bool,
    count: u64,
) -> Result<u64, String> {
    let hour = Local::now().hour();
    let is_late = !(7..23).contains(&hour); // 11 PM to 7 AM

    if !is_late {
        return Ok(count);
    }

    let notify = |title: &str, body: &str| {
        app.notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|e| e.to_string())
    };

    let store = app.store("config.json").map_err(|e| e.to_string())?;

    let sleep_enabled = store
        .get("integrations")
        .or_else(|| store.get("intergrations"))
        .and_then(|v| v.get("sleepSchedule").cloned())
        .and_then(|v| v.get("enabled").cloned())
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    if sleep_enabled {
        if count == 0 {
            notify(
                "About your sleep schedule...",
                "It's getting late. You should go to sleep.",
            )?;

            let mut integrations = store
                .get("integrations")
                .or_else(|| store.get("intergrations"))
                .unwrap_or_else(
                    || json!({ "sleepSchedule": { "visible": true, "enabled": true } }),
                );

            if let Some(integrations_obj) = integrations.as_object_mut() {
                if let Some(sleep_schedule) = integrations_obj.get_mut("sleepSchedule") {
                    if let Some(sleep_obj) = sleep_schedule.as_object_mut() {
                        sleep_obj.insert("visible".to_string(), json!(true));
                        sleep_obj.insert("enabled".to_string(), json!(true));
                    }
                } else {
                    integrations_obj.insert(
                        "sleepSchedule".to_string(),
                        json!({ "visible": true, "enabled": true }),
                    );
                }
            }

            store.set("integrations", integrations);
            let _ = store
                .save()
                .map_err(|e| log::error!("Failed to save store: {}", e));
        } else if count == 1 {
            notify("Hiya...", "Sleep now?")?;
        } else if count == 2 {
            notify("Last warning", "This is the last one, go to sleep or else")?;
        } else if count == 3 && !in_game {
            notify("Good night", "Now go to bed and have sweet dreams")?;
        }

        return Ok(count + 1);
    }

    Ok(count)
}

pub(super) async fn send_location_notification(
    app: &AppHandle,
    state: &mut WatcherState,
    store: &tauri_plugin_store::Store<tauri::Wry>,
) -> Result<(), String> {
    if !integration_enabled(store, &["serverLocationNotifier"]) {
        state.pending_server_ip = None;
        state.pending_server_location = None;
        return Ok(());
    }

    let i18n = app.state::<I18n>().inner().clone();

    if let (Some(ip), Some(location)) = (
        state.pending_server_ip.as_deref(),
        state.pending_server_location.as_deref(),
    ) {
        state.location_notified = true;
        let title_key = if state.activity.is_private_server {
            "rpc.rust.watcher.serverInfomation.titles.private"
        } else {
            "rpc.rust.watcher.serverInfomation.titles.public"
        };

        let title = i18n.t(title_key);
        let description = t!(
            i18n,
            "rpc.rust.watcher.serverInfomation.description",
            ip = ip,
            location = location
        );

        let image = match state.activity.place_id {
            Some(place_id) => {
                let thumbnail = fetch_game_thumbnail(place_id).await.unwrap_or_else(|e| {
                    log::warn!("failed to fetch game thumbnail: {}", e);
                    None
                });

                // fall back to the square game icon if there's no thumbnail
                match thumbnail {
                    Some(url) => Some(url),
                    None => fetch_game_icon(place_id).await.unwrap_or_else(|e| {
                        log::warn!("failed to fetch game icon: {}", e);
                        None
                    }),
                }
            }
            None => None,
        };

        match image {
            Some(url) => show_with_image(app, &title, Some(&description), url),
            None => show(app, &title, Some(&description)),
        }
    }
    Ok(())
}

pub(super) async fn update_discord_rpc(
    app: &AppHandle,
    state: &mut WatcherState,
    place_id: u64,
) -> Result<(), String> {
    let now = Instant::now();
    if state
        .last_rpc
        .is_some_and(|t| now.duration_since(t).as_secs() <= 2)
    {
        return Ok(());
    }

    state.last_rpc = Some(now);
    let instance_id = state.activity.instance_id.clone().unwrap_or_default();
    let app_c = app.clone();

    let is_private = state.activity.is_private_server;

    tauri::async_runtime::spawn(async move {
        let (name, image) = match fetch_place_info(place_id).await {
            Ok(Some(v)) => v,
            _ => ("Roblox".to_string(), String::new()),
        };

        let image_key = if image.is_empty() {
            None
        } else {
            Some(image.as_str())
        };

        let mut buttons: Vec<(String, String)> = vec![(
            "View Game".to_string(),
            format!("https://www.roblox.com/games/{}", place_id),
        )];

        if !is_private {
            buttons = vec![
                (
                    "Join Server".to_string(),
                    format!(
                        "https://deeplink.multicrew.dev?placeId={}&jobId={}",
                        place_id, instance_id
                    ),
                ),
                (
                    "View Game".to_string(),
                    format!("https://www.roblox.com/games/{}", place_id),
                ),
            ];
        }

        const CLIENT_ID: &str = "363445589247131668";
        let rpc = app_c.state::<RpcState>();

        if rpc.client.lock().await.is_none() {
            if let Err(e) = start_rpc(&rpc, CLIENT_ID).await {
                log::error!("RPC start failed: {}", e);
                return;
            }
        }

        let i18n = app_c.state::<I18n>().inner().clone();

        let res = tokio::time::timeout(
            Duration::from_secs(5),
            apply_rpc_full(
                &rpc,
                Some(&name),
                Some(&i18n.t("rpc.rust.watcher.general").to_string()),
                None,
                None,
                None,
                Some(buttons.clone()),
                image_key,
            ),
        )
        .await;

        if let Ok(Err(e)) = res {
            log::warn!("RPC failed ({}), reconnecting…", e);
            *rpc.client.lock().await = None;
            *rpc.runner.lock().await = None;

            if let Err(e) = start_rpc(&rpc, CLIENT_ID).await {
                log::error!("RPC reconnect failed: {}", e);
                return;
            }

            let i18n = app_c.state::<I18n>().inner().clone();

            let _ = tokio::time::timeout(
                Duration::from_secs(5),
                apply_rpc_full(
                    &rpc,
                    Some(&name),
                    Some(&i18n.t("rpc.rust.watcher.general").to_string()),
                    None,
                    None,
                    None,
                    Some(buttons.clone()),
                    image_key,
                ),
            )
            .await;
        } else if res.is_err() {
            log::error!("RPC apply timed out");
        }
    });

    Ok(())
}
