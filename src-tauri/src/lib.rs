use commands::archive::{extract_files_from_zip, extract_zip};
use commands::boostrapper_importer::export_boostrapconfig;
use commands::crush::crush;
use commands::discord_rpc::set_rpc;
use commands::fs::copy_file;
use commands::gbs_operations::{get_gbs, write_gbs};
use commands::launch_roblox::launch;
use commands::mods::apply_mod;
use commands::pre_processing::{close_crash_handler, set_process_priority};
use commands::properity::{read_fullscreen_prop, set_fullscreen_prop};
use commands::rename::rename;
use commands::roblox_deployment::{
    get_best_region, get_download_deployment_urls, get_latest_version_player,
    get_latest_version_studio,
};
use commands::shortcuts::new_shorcut;
use commands::watcher::watch_logs;
use commands::window::{
    apply_vibrancy_to_window, create_or_focus_window, kill_window, set_window_vibrancy,
};
use image::GenericImageView;
use tao::window::Icon;
use tauri::{Emitter, Manager};
use tauri_plugin_cli::CliExt;
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;
use tauri_plugin_updater::UpdaterExt;
mod commands;
use crate::rpc::kill_rpc;
use rpc::RpcState;
use simple_i18n::I18n;

pub mod interactive;
pub mod priorites;
pub mod rd;
pub mod rpc;
pub mod simple_i18n;
pub mod tray;

use crate::tray::setup_tray;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn load_icon(path: &std::path::Path) -> tauri::image::Image<'static> {
    let img = image::open(path).expect("Failed to open icon");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    tauri::image::Image::new_owned(rgba, width, height)
}

fn register_plugins<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let is_deep_link = args
                .iter()
                .any(|a| a.starts_with("roblox-player:") || a.starts_with("roblox:"));

            if is_deep_link {
                return;
            }

            app.dialog()
                .message("The app is already running! Look for it in your system tray.")
                .kind(MessageDialogKind::Info)
                .title("Already Running")
                .blocking_show();
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_fs_pro::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
}

fn handle_received_url(app_handle: &tauri::AppHandle, url: String) {
    app_handle.emit("deep-link-received", url.clone()).ok();

    if let Some(win) = app_handle.get_webview_window("crushBoostrapChoiceWindow") {
        let _ = win.show();
        let _ = win.set_focus();
    }
    log::info!("{}", url);
}

fn setup_deep_links(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.deep_link().register_all()?;

    let app_handle = app.handle().clone();
    app.deep_link().on_open_url(move |event| {
        if let Some(url) = event.urls().first() {
            handle_received_url(&app_handle, url.to_string());
        }
    });

    Ok(())
}

fn spawn_discord_rpc(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state: tauri::State<'_, RpcState> = app_handle.state::<RpcState>();

        if let Err(e) = crate::rpc::start_rpc(&state, "363445589247131668").await {
            log::error!("RPC error: {}", e);
        };
    });
}

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0u64;
        let mut last_notified_percent = 0u64;

        let app_download = app.clone(); // for download closure
        let app_finish = app.clone(); // for finish closure

        update
            .download_and_install(
                move |chunk_length, content_length| {
                    downloaded += chunk_length as u64;
                    println!("downloaded {downloaded} from {content_length:?}");

                    if let Some(total) = content_length {
                        let percent = (downloaded * 100) / total;
                        if percent >= last_notified_percent + 30 {
                            last_notified_percent = percent;
                            app_download
                                .notification()
                                .builder()
                                .title("Auto updater")
                                .body(format!("Downloading... {}%", percent))
                                .show()
                                .unwrap();
                        }
                    }
                },
                move || {
                    app_finish
                        .notification()
                        .builder()
                        .title("Auto updater")
                        .body("Download finished!")
                        .show()
                        .unwrap();
                },
            )
            .await?;

        app.notification()
            .builder()
            .title("Auto updater")
            .body("Update installed! The new update will take effect after restart.")
            .show()
            .unwrap();
        app.restart();
    }

    Ok(())
}

fn print_debug_info() {
    log::info!("Debug Info:");
    log::info!("OS: {}", tauri_plugin_os::platform());
    log::info!("Git hash: {}", env!("VERGEN_RUSTC_COMMIT_HASH"));
    log::info!("Build date: {}", env!("VERGEN_BUILD_DATE"));
    log::info!("Build timestamp: {}", env!("VERGEN_BUILD_TIMESTAMP"));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_clipboard_manager::init());

    builder = register_plugins(builder);

    builder
        .manage(RpcState::new())
        .setup(|app| {
            print_debug_info();

            let platform = tauri_plugin_os::platform();

            let locale = app
                .get_store("config.json")
                .and_then(|store| store.get("language"))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "en-US".to_string());

            let path = app
                .path()
                .resolve("resources/locales", tauri::path::BaseDirectory::Resource)?;

            let i18n = I18n::new(path, &locale).unwrap();

            app.manage(i18n);

            if platform != "windows" {
                app.dialog()
                    .message(format!(
                        "This app can't work on {}. However, we will have plans for {}.",
                        platform, platform
                    ))
                    .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                    .title("Error")
                    .blocking_show();
                std::process::exit(1);
            }

            let Some(window) = app.get_webview_window("crushBoostrapChoiceWindow") else {
                return Err("Failed to find main bootstrap choice window".into());
            };

            let effect = app
                .get_store("config.json")
                .and_then(|store| store.get("vibrancy"))
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "auto".to_string());

            apply_vibrancy_to_window(&window, &effect);

            let icon_resource = app.path().resolve("resources/icon.ico", tauri::path::BaseDirectory::Resource)?;
            let icon = load_icon(&icon_resource);
            window.set_icon(icon).ok(); // does this fucking work?????

            match app.cli().matches() {
                // https://v2.tauri.app/plugin/cli/

                // `matches` here is a Struct with { args, subcommand }.
                // `args` is `HashMap<String, ArgData>` where `ArgData` is a struct with { value, occurrences }.
                // `subcommand` is `Option<Box<SubcommandMatches>>` where `SubcommandMatches` is a struct with { name, matches }.
                Ok(matches) => {
                    log::info!("app launched with args: {:?}", matches.args);
                    if matches.args.contains_key("launch") {
                        let game_id = matches
                            .args
                            .get("launch")
                            .and_then(|arg| arg.value.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_default();

                        let app_handle = app.handle().clone();

                        if game_id.is_empty() {
                            log::info!(
                                "maybe didnt input game id, but cli still picking up arg, ingoring"
                            ); // tauri be weird sometimes
                        } else {
                            // add a guard here
                            tauri::async_runtime::spawn(async move {
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                                handle_received_url(
                                    &app_handle,
                                    format!("roblox://experiences/start?placeId={}", game_id), // not sure
                                );
                            });
                        }
                    }
                }
                Err(_) => {}
            }

            setup_deep_links(app)?;
            spawn_discord_rpc(app.handle().clone());
            setup_tray(app)?;

            // run auto update after startup
            // https://v2.tauri.app/plugin/updater/

            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                update(app_handle).await.unwrap();
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "CrushBoostrap" || window.label() == "crushBoostrap" {
                    return;
                }

                api.prevent_close();
                let _ = window.hide();

                let app_handle = window.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    let state = app_handle.state::<RpcState>();
                    let _ = kill_rpc(&state).await;
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            kill_window,
            get_download_deployment_urls,
            get_best_region,
            create_or_focus_window,
            new_shorcut,
            extract_zip,
            extract_files_from_zip,
            launch,
            get_latest_version_player,
            rename,
            apply_mod,
            crush,
            watch_logs,
            set_rpc,
            copy_file,
            export_boostrapconfig,
            set_window_vibrancy,
            get_latest_version_studio,
            set_process_priority,
            close_crash_handler,
            get_gbs,
            write_gbs,
            set_fullscreen_prop,
            read_fullscreen_prop
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
