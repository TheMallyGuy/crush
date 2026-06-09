use reqwest::Client;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

pub struct ConversationHistory(pub Mutex<Vec<serde_json::Value>>);

const STORE_PATH: &str = "settings.json";
const KEY_FIELD: &str = "ai_nvidia_key";
const NVIDIA_API_URL: &str = "https://integrate.api.nvidia.com/v1/chat/completions";
const NVIDIA_MODEL: &str = "mistralai/mistral-medium-3.5-128b";
const VISION_MODEL: &str = "meta/llama-3.2-11b-vision-instruct";
const SYSTEM_PROMPT: &str = "You are a helpful AI assistant embedded in Crush, a Roblox game launcher for Windows. \
You have tools to take a screenshot of the user's screen and search the web. \
Keep responses concise - you're displayed in a compact overlay panel. \
Answer in the same language the user speaks.";

fn get_tools() -> serde_json::Value {
    serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "take_screenshot",
                "description": "Take a screenshot of the user's current screen and return a detailed description of what is visible.",
                "parameters": { "type": "object", "properties": {}, "required": [] }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web for current information on a topic.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query string"
                        }
                    },
                    "required": ["query"]
                }
            }
        }
    ])
}

fn capture_screen_jpeg() -> Result<String, String> {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
        GetDeviceCaps, SelectObject, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        DIB_RGB_COLORS, HORZRES, SRCCOPY, VERTRES,
    };

    unsafe {
        let screen_dc = windows::Win32::Graphics::Gdi::GetDC(None);
        if screen_dc.is_invalid() {
            return Err("GetDC failed".to_string());
        }

        let src_w = GetDeviceCaps(Some(screen_dc), HORZRES);
        let src_h = GetDeviceCaps(Some(screen_dc), VERTRES);

        let scale = if src_w > 1280 {
            1280.0 / src_w as f32
        } else {
            1.0
        };
        let dst_w = (src_w as f32 * scale) as i32;
        let dst_h = (src_h as f32 * scale) as i32;

        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        let bitmap = CreateCompatibleBitmap(screen_dc, dst_w, dst_h);
        let old_obj = SelectObject(mem_dc, bitmap.into());

        let _ = StretchBlt(
            mem_dc,
            0,
            0,
            dst_w,
            dst_h,
            Some(screen_dc),
            0,
            0,
            src_w,
            src_h,
            SRCCOPY,
        );

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: dst_w,
                biHeight: -dst_h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut pixels = vec![0u8; (dst_w * dst_h * 4) as usize];
        GetDIBits(
            mem_dc,
            bitmap,
            0,
            dst_h as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(mem_dc, old_obj);
        let _ = DeleteObject(bitmap.into());
        DeleteDC(mem_dc);
        windows::Win32::Graphics::Gdi::ReleaseDC(None, screen_dc);

        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        let img = image::RgbaImage::from_raw(dst_w as u32, dst_h as u32, pixels)
            .ok_or_else(|| "Failed to create image buffer".to_string())?;
        let img = image::DynamicImage::ImageRgba8(img);

        let mut jpeg_bytes: Vec<u8> = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut jpeg_bytes),
            image::ImageFormat::Jpeg,
        )
        .map_err(|e| e.to_string())?;

        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes))
    }
}

async fn execute_tool(
    name: &str,
    args: &serde_json::Value,
    client: &Client,
    api_key: &str,
) -> String {
    match name {
        "take_screenshot" => {
            let b64 = match tokio::task::spawn_blocking(capture_screen_jpeg).await {
                Ok(Ok(b)) => b,
                Ok(Err(e)) => return format!("Screenshot capture failed: {e}"),
                Err(e) => return format!("Screenshot task panicked: {e}"),
            };

            match client
                .post(NVIDIA_API_URL)
                .bearer_auth(api_key)
                .json(&serde_json::json!({
                    "model": VISION_MODEL,
                    "messages": [{
                        "role": "user",
                        "content": [
                            {
                                "type": "text",
                                "text": "Describe this screenshot in detail. What applications, windows, text, or content is visible?"
                            },
                            {
                                "type": "image_url",
                                "image_url": { "url": format!("data:image/jpeg;base64,{}", b64) }
                            }
                        ]
                    }],
                    "max_tokens": 512,
                    "temperature": 0.2
                }))
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => r
                    .json::<serde_json::Value>()
                    .await
                    .ok()
                    .and_then(|j| j["choices"][0]["message"]["content"].as_str().map(str::to_owned))
                    .unwrap_or_else(|| "Vision model returned no content.".to_string()),
                Ok(r) => {
                    let status = r.status();
                    let body = r.text().await.unwrap_or_default();
                    format!("Vision model error {status}: {body}")
                }
                Err(e) => format!("Vision request failed: {e}"),
            }
        }

        "web_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            if query.is_empty() {
                return "No search query provided.".to_string();
            }

            let mut url = reqwest::Url::parse("https://api.duckduckgo.com/").unwrap();
            url.query_pairs_mut()
                .append_pair("q", query)
                .append_pair("format", "json")
                .append_pair("no_html", "1")
                .append_pair("skip_disambig", "1");

            let resp = client
                .get(url)
                .header("User-Agent", "Mozilla/5.0 crush-app")
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => match r.json::<serde_json::Value>().await {
                    Ok(json) => {
                        let mut out = String::new();

                        if let Some(a) = json["Answer"].as_str() {
                            if !a.is_empty() {
                                out.push_str(&format!("Answer: {a}\n\n"));
                            }
                        }
                        if let Some(t) = json["AbstractText"].as_str() {
                            if !t.is_empty() {
                                let url = json["AbstractURL"].as_str().unwrap_or("");
                                out.push_str(&format!("Summary: {t}\nSource: {url}\n\n"));
                            }
                        }
                        if let Some(topics) = json["RelatedTopics"].as_array() {
                            let items: Vec<&str> = topics
                                .iter()
                                .take(5)
                                .filter_map(|t| t["Text"].as_str())
                                .filter(|t| !t.is_empty())
                                .collect();
                            if !items.is_empty() {
                                out.push_str("Related:\n");
                                for item in items {
                                    out.push_str(&format!("- {item}\n"));
                                }
                            }
                        }

                        if out.is_empty() {
                            format!("No results found for \"{query}\".")
                        } else {
                            out
                        }
                    }
                    Err(e) => format!("Failed to parse search response: {e}"),
                },
                Ok(r) => format!("Search failed: HTTP {}", r.status()),
                Err(e) => format!("Search request failed: {e}"),
            }
        }

        unknown => format!("Unknown tool: {unknown}"),
    }
}

#[derive(Default)]
struct ToolCallAccum {
    id: String,
    name: String,
    arguments: String,
}

async fn stream_request(
    client: &Client,
    api_key: &str,
    messages: &serde_json::Value,
    with_tools: bool,
    window: &tauri::WebviewWindow,
) -> (Option<Vec<ToolCallAccum>>, String) {
    let mut body = serde_json::json!({
        "model": NVIDIA_MODEL,
        "messages": messages,
        "stream": true,
        "max_tokens": 1024,
        "temperature": 0.7,
    });
    if with_tools {
        body["tools"] = get_tools();
        body["tool_choice"] = serde_json::json!("auto");
    }

    let mut response = match client
        .post(NVIDIA_API_URL)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            let _ = window.emit("ai:token", format!("API error {status}: {text}"));
            return (None, String::new());
        }
        Err(e) => {
            let _ = window.emit("ai:token", format!("Request failed: {e}"));
            return (None, String::new());
        }
    };

    let mut buf = String::new();
    let mut in_think = false;
    let mut tool_map: HashMap<usize, ToolCallAccum> = HashMap::new();
    let mut finish_reason = String::new();
    let mut response_text = String::new();

    loop {
        match response.chunk().await {
            Ok(Some(bytes)) => {
                buf.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(end) = buf.find("\n\n") {
                    let event = buf[..end].to_string();
                    buf = buf[end + 2..].to_string();

                    for line in event.lines() {
                        let Some(data) = line.strip_prefix("data: ") else {
                            continue;
                        };
                        if data == "[DONE]" {
                            break;
                        }
                        let Ok(json) = serde_json::from_str::<serde_json::Value>(data) else {
                            continue;
                        };

                        let choice = &json["choices"][0];
                        if let Some(fr) = choice["finish_reason"].as_str() {
                            finish_reason = fr.to_string();
                        }
                        let delta = &choice["delta"];

                        if let Some(tc_arr) = delta["tool_calls"].as_array() {
                            for tc in tc_arr {
                                let idx = tc["index"].as_u64().unwrap_or(0) as usize;
                                let e = tool_map.entry(idx).or_default();
                                if let Some(id) = tc["id"].as_str() {
                                    e.id = id.to_string();
                                }
                                if let Some(n) = tc["function"]["name"].as_str() {
                                    e.name = n.to_string();
                                }
                                if let Some(a) = tc["function"]["arguments"].as_str() {
                                    e.arguments.push_str(a);
                                }
                            }
                        }

                        if let Some(content) = delta["content"].as_str() {
                            let mut out = String::new();
                            let mut rest = content;
                            loop {
                                if in_think {
                                    if let Some(p) = rest.find("</think>") {
                                        in_think = false;
                                        rest = &rest[p + 8..];
                                    } else {
                                        break;
                                    }
                                } else if let Some(p) = rest.find("<think>") {
                                    out.push_str(&rest[..p]);
                                    in_think = true;
                                    rest = &rest[p + 7..];
                                } else {
                                    out.push_str(rest);
                                    break;
                                }
                            }
                            if !out.is_empty() {
                                response_text.push_str(&out);
                                let _ = window.emit("ai:token", out);
                            }
                        }
                    }
                }
            }
            Ok(None) => break,
            Err(e) => {
                let _ = window.emit("ai:token", format!(" [stream error: {e}]"));
                break;
            }
        }
    }

    if finish_reason == "tool_calls" && !tool_map.is_empty() {
        let mut sorted: Vec<_> = tool_map.into_iter().collect();
        sorted.sort_by_key(|(idx, _)| *idx);
        (Some(sorted.into_iter().map(|(_, tc)| tc).collect()), String::new())
    } else {
        (None, response_text)
    }
}

#[tauri::command]
pub async fn ask_ai(app: tauri::AppHandle, prompt: String) {
    let window = app.get_webview_window("crushOverlay").unwrap();
    let _ = window.emit("ai:thinking", ());

    let api_key = app
        .store(STORE_PATH)
        .ok()
        .and_then(|s| s.get(KEY_FIELD))
        .and_then(|v| v.as_str().map(str::to_owned))
        .or_else(|| std::env::var("NVIDIA_API_KEY").ok())
        .unwrap_or_default();

    if api_key.is_empty() {
        let _ = window.emit("ai:token", "No API key. Set \"ai_nvidia_key\" in the settings store or the NVIDIA_API_KEY env var.");
        let _ = window.emit("ai:done", ());
        return;
    }

    // Build messages: system + last 3 exchanges (6 msgs) + new user turn
    let history_snapshot: Vec<serde_json::Value> = {
        let state = app.state::<ConversationHistory>();
        let guard = state.0.lock().unwrap();
        let all = guard.as_slice();
        // keep at most the last 6 messages (3 user+assistant pairs)
        let start = all.len().saturating_sub(6);
        all[start..].to_vec()
    };

    let mut messages_arr = vec![
        serde_json::json!({ "role": "system", "content": SYSTEM_PROMPT }),
    ];
    messages_arr.extend(history_snapshot);
    messages_arr.push(serde_json::json!({ "role": "user", "content": prompt }));
    let mut messages = serde_json::json!(messages_arr);

    let client = Client::new();

    let (tool_calls, first_text) =
        stream_request(&client, &api_key, &messages, true, &window).await;

    let final_text = if let Some(calls) = tool_calls {
        let tc_json: Vec<serde_json::Value> = calls
            .iter()
            .map(|tc| {
                serde_json::json!({
                    "id": tc.id,
                    "type": "function",
                    "function": { "name": tc.name, "arguments": tc.arguments }
                })
            })
            .collect();

        let msgs = messages.as_array_mut().unwrap();
        msgs.push(serde_json::json!({
            "role": "assistant",
            "content": null,
            "tool_calls": tc_json
        }));

        for tc in &calls {
            let args: serde_json::Value =
                serde_json::from_str(&tc.arguments).unwrap_or(serde_json::json!({}));
            let result = execute_tool(&tc.name, &args, &client, &api_key).await;
            msgs.push(serde_json::json!({
                "role": "tool",
                "tool_call_id": tc.id,
                "content": result
            }));
        }

        let (_, text) = stream_request(&client, &api_key, &messages, false, &window).await;
        text
    } else {
        first_text
    };

    // Persist the exchange to history
    if !final_text.is_empty() {
        let hist = app.state::<ConversationHistory>();
        let mut hist = hist.0.lock().unwrap();
        hist.push(serde_json::json!({ "role": "user", "content": prompt }));
        hist.push(serde_json::json!({ "role": "assistant", "content": final_text }));
    }

    let _ = window.emit("ai:done", ());
}

#[tauri::command]
pub async fn clear_ai_history(app: tauri::AppHandle) {
    let hist = app.state::<ConversationHistory>();
    hist.0.lock().unwrap().clear();
}

#[tauri::command]
pub async fn set_overlay_interactive(app: tauri::AppHandle, interactive: bool) {
    if let Some(window) = app.get_webview_window("crushOverlay") {
        let _ = window.set_ignore_cursor_events(!interactive);
    }
}

#[tauri::command]
pub async fn resize_overlay(app: tauri::AppHandle, w: u32, h: u32) {
    if let Some(window) = app.get_webview_window("crushOverlay") {
        if w == 0 || h == 0 {
            let _ = window.set_ignore_cursor_events(true);
            return;
        }
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let sw = monitor.size().width as i32;
            let x = (sw - w as i32) / 2;
            let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: w,
                height: h,
            }));
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x,
                y: 0,
            }));
            let _ = window.set_ignore_cursor_events(false);
        }
    }
}
