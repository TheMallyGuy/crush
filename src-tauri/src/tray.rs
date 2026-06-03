use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconId;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_store::StoreExt;

pub struct TrayState {
    pub id: TrayIconId,
    pub dynamic_items: Mutex<Vec<(String, String)>>,
}

impl TrayState {
    pub fn new(id: TrayIconId) -> Self {
        Self {
            id,
            dynamic_items: Mutex::new(vec![]),
        }
    }

    pub fn get_tray(app: &AppHandle) -> Option<tauri::tray::TrayIcon> {
        let state = app.state::<TrayState>();
        app.tray_by_id(&state.id)
    }
}

fn build_menu(
    app: &AppHandle,
    extra_items: &[(&str, &str)],
) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let open_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let mut items: Vec<Box<dyn tauri::menu::IsMenuItem<tauri::Wry>>> = vec![Box::new(open_i)];

    for (id, label) in extra_items {
        let item = MenuItem::with_id(app, *id, *label, true, None::<&str>)?;
        items.push(Box::new(item));
    }

    items.push(Box::new(quit_i));

    let item_refs: Vec<&dyn tauri::menu::IsMenuItem<tauri::Wry>> =
        items.iter().map(|i| i.as_ref()).collect();
    let menu = Menu::with_items(app, &item_refs)?;

    Ok(menu)
}

pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_menu(app.handle(), &[])?;

    let mut tray_builder = TrayIconBuilder::new();

    if let Some(icon) = app.default_window_icon() {
        tray_builder = tray_builder.icon(icon.clone());
    }

    let tray = tray_builder
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "open" => {
                let window = app
                    .get_webview_window("CrushMainWindow")
                    .or_else(|| app.get_webview_window("crushBoostrapChoiceWindow"));

                if let Some(window) = window {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "serverinfo" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(window) = app.get_webview_window("CrushHello") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    } else {
                        let webview_url = WebviewUrl::App(
                            "mainWin/other/serverInfo"
                                .parse()
                                .expect("Failed to parse URL"),
                        );
                    }
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let window = app
                    .get_webview_window("CrushMainWindow")
                    .or_else(|| app.get_webview_window("crushBoostrapChoiceWindow"));

                if let Some(window) = window {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    app.manage(TrayState::new(tray.id().clone()));

    Ok(())
}

pub fn add_menu_item(
    app: &AppHandle,
    id: &str,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(tray) = TrayState::get_tray(app) else {
        return Ok(());
    };

    {
        let state = app.state::<TrayState>();
        let mut items = state.dynamic_items.lock().unwrap();
        // avoid duplicates
        if !items.iter().any(|(i, _)| i == id) {
            items.push((id.to_string(), label.to_string()));
        }
    }

    let state = app.state::<TrayState>();
    let items = state.dynamic_items.lock().unwrap();
    let as_refs: Vec<(&str, &str)> = items
        .iter()
        .map(|(i, l)| (i.as_str(), l.as_str()))
        .collect();

    let menu = build_menu(app, &as_refs)?;
    tray.set_menu(Some(menu))?;

    Ok(())
}

pub fn remove_menu_item(app: &AppHandle, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let Some(tray) = TrayState::get_tray(app) else {
        return Ok(());
    };

    {
        let state = app.state::<TrayState>();
        let mut items = state.dynamic_items.lock().unwrap();
        items.retain(|(i, _)| i != id);
    }

    let state = app.state::<TrayState>();
    let items = state.dynamic_items.lock().unwrap();
    let as_refs: Vec<(&str, &str)> = items
        .iter()
        .map(|(i, l)| (i.as_str(), l.as_str()))
        .collect();

    let menu = build_menu(app, &as_refs)?;
    tray.set_menu(Some(menu))?;

    Ok(())
}
