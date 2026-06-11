use serde_json::Value;

pub fn integration_enabled(store: &tauri_plugin_store::Store<tauri::Wry>, path: &[&str]) -> bool {
    let v = store
        .get("integrations")
        .or_else(|| store.get("intergrations"));
    let Some(mut cur) = v else { return false };
    for key in path {
        cur = cur.get(key).cloned().unwrap_or(Value::Null);
    }
    cur.as_bool().unwrap_or(false)
}

pub fn config_get(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    root: &str,
    path: &[&str],
) -> Option<Value> {
    let mut cur = store.get(root)?;
    for key in path {
        cur = cur.get(key).cloned()?;
    }
    Some(cur)
}

pub fn config_bool(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    root: &str,
    path: &[&str],
) -> bool {
    config_get(store, root, path)
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

pub(super) fn get_transparency_bound(
    store: &tauri_plugin_store::Store<tauri::Wry>,
    key: &str,
    default: u8,
) -> u8 {
    let v = store
        .get("integrations")
        .or_else(|| store.get("intergrations"));
    let Some(root) = v else { return default };
    root.get("interactive")
        .and_then(|v| v.get("scopes"))
        .and_then(|v| v.get("transparencyScopes"))
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_u64())
        .map(|v| v.clamp(0, 255) as u8)
        .unwrap_or(default)
}
