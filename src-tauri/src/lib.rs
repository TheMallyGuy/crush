use crate::swifttunnel_sdk::SwiftTunnel;
use commands::account_operations::{
    clear_cookies, decrypt_cookie_data, encrypt_cookie_data, export_all_cookies, get_auth_ticket,
    get_csrf_token, quick_sign_create, quick_sign_poll, validate_roblox_cookie,
};
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
use commands::swifttunnel::{
    connect, start_browser_login, swift_auth_logout, swift_cancel_auth, swift_fetch_servers,
    swift_get_servers, swift_is_logged_in, swift_servers_ping,
};
use commands::watcher::watch_logs;
use commands::window::{create_or_focus_window, kill_window};
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri_plugin_cli::CliExt;
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;
use tauri_plugin_updater::UpdaterExt;
mod commands;
use crate::rpc::kill_rpc;
use rpc::RpcState;
use simple_i18n::I18n;

pub mod collector;
pub mod interactive;
pub mod larp_focuser;
pub mod priorites;
pub mod rd;
pub mod rpc;
pub mod simple_i18n;
pub mod swifttunnel_sdk;
pub mod tray; // nice name choice buddy

use crate::tray::setup_tray;

pub struct SdkState(pub Mutex<SwiftTunnel>);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn register_plugins<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .plugin(tauri_plugin_deep_link::init())
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
    #[cfg(windows)]
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            use std::os::windows::ffi::OsStrExt;
            use windows::Win32::System::LibraryLoader::SetDllDirectoryW;
            let lib_dir = exe_dir.join("libraries");
            let mut wide: Vec<u16> = lib_dir.as_os_str().encode_wide().chain([0u16]).collect();
            let _ = unsafe { SetDllDirectoryW(windows::core::PCWSTR(wide.as_mut_ptr())) };
        }
    }

    let sdk = SwiftTunnel::init().expect("SwiftTunnel SDK init failed");

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_cache::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            let is_deep_link = args
                .iter()
                .any(|a| a.starts_with("roblox-player:") || a.starts_with("roblox:"));

            if is_deep_link {
                return;
            }

            let launch_id = args
                .windows(2)
                .find_map(|w| {
                    if w[0] == "-l" || w[0] == "--launch" {
                        Some(w[1].clone())
                    } else {
                        None
                    }
                })
                .or_else(|| {
                    args.iter().find_map(|a| {
                        a.strip_prefix("--launch=")
                            .or_else(|| a.strip_prefix("-l="))
                            .map(|s| s.to_string())
                    })
                });

            if let Some(game_id) = launch_id {
                if !game_id.is_empty() {
                    handle_received_url(
                        app,
                        format!("roblox://experiences/start?placeId={}", game_id),
                    );
                    return;
                }
            }

            if let Some(window) = app.get_webview_window("crushBoostrapChoiceWindow") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }));

    builder = register_plugins(builder);

    builder
        .manage(RpcState::new())
        .manage(SdkState(Mutex::new(sdk)))
        .setup(|app| {
            print_debug_info();

            let platform = tauri_plugin_os::platform();

            let locale = app
                .get_store("config.json")
                .and_then(|store| store.get("settings"))
                .and_then(|v| v.get("language").and_then(|l| l.as_str().map(|s| s.to_string())))
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

            let pool = collector::init(app.handle());
            app.manage(pool);

            // run auto update after startup
            // https://v2.tauri.app/plugin/updater/

            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = update(app_handle).await {
                    log::warn!("update check failed: {}", e);
                }
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
            get_latest_version_studio,
            set_process_priority,
            close_crash_handler,
            get_gbs,
            write_gbs,
            set_fullscreen_prop,
            read_fullscreen_prop,
            get_csrf_token,
            get_auth_ticket,
            export_all_cookies,
            clear_cookies,
            decrypt_cookie_data,
            encrypt_cookie_data,
            quick_sign_poll,
            quick_sign_create,
            validate_roblox_cookie,
            start_browser_login,
            swift_is_logged_in,
            swift_cancel_auth,
            swift_auth_logout,
            swift_get_servers,
            swift_fetch_servers,
            connect,
            swift_servers_ping
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
