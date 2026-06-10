use windows_dpapi::{decrypt_data, encrypt_data, Scope};

#[tauri::command]
pub async fn clear_cookies(webview: tauri::Webview) -> Result<(), String> {
    webview.clear_all_browsing_data().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn export_all_cookies(webview: tauri::Webview) -> Result<String, String> {
    let cookies = webview.cookies().map_err(|e| e.to_string())?;

    // check if user is logged in, return error if not found
    cookies
        .iter()
        .find(|c| c.name() == ".ROBLOSECURITY")
        .map(|c| c.value().to_string())
        .ok_or_else(|| ".ROBLOSECURITY cookie not found".to_string())?;

    let mut lines = Vec::new();

    for cookie in cookies {
        let domain = cookie.domain().unwrap_or("");
        let http_only = if cookie.http_only().unwrap_or(false) {
            "#HttpOnly_"
        } else {
            ""
        };
        let flag = "TRUE";
        let path = cookie.path().unwrap_or("/");
        let secure = if cookie.secure().unwrap_or(false) {
            "TRUE"
        } else {
            "FALSE"
        };

        let expires = match cookie.expires() {
            Some(cookie::Expiration::Session) => 0,
            Some(cookie::Expiration::DateTime(dt)) => dt.unix_timestamp(),
            None => 0,
        };

        let line = format!(
            "{}{}\t{}\t{}\t{}\t{}\t{}\t{}",
            http_only,
            domain,
            flag,
            path,
            secure,
            expires,
            cookie.name(),
            cookie.value()
        );
        lines.push(line);
    }

    Ok(lines.join("\n"))
}

#[tauri::command]
pub async fn decrypt_cookie_data(encrypted: String) -> Result<String, String> {
    let encrypted_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted)
            .map_err(|e| e.to_string())?;
    let decrypted_data = decrypt_data(&encrypted_bytes, Scope::User, None)
        .map_err(|e| format!("Failed to decrypt data: {}", e))?;

    Ok(String::from_utf8(decrypted_data).map_err(|e| e.to_string())?)
}

#[tauri::command]
pub async fn encrypt_cookie_data(decrypted: String) -> Result<String, String> {
    let encrypted_data = encrypt_data(decrypted.as_bytes(), Scope::User, None)
        .map_err(|e| format!("Failed to encrypt data: {}", e))?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        encrypted_data,
    ))
}

#[tauri::command]
pub async fn validate_roblox_cookie(cookie: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://users.roblox.com/v1/users/authenticated")
        .header("Cookie", &cookie)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.status() == 200 {
        let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(data)
    } else {
        Err(format!("HTTP {}", response.status()))
    }
}

#[tauri::command]
pub async fn get_csrf_token(cookie: String) -> Result<String, String> {
    let client = reqwest::Client::new();

    let resp = client
        .post("https://auth.roblox.com/v1/authentication-ticket/")
        .header("Cookie", &cookie)
        .header("Content-Length", "0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    resp.headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or("No CSRF token found".to_string())
}

#[tauri::command]
pub async fn get_auth_ticket(
    cookie: String,
    csrf: String,
    place_id: u64,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post("https://auth.roblox.com/v1/authentication-ticket/")
        .header("Cookie", &cookie)
        .header("X-CSRF-TOKEN", &csrf)
        .header("Origin", "https://www.roblox.com")
        .header(
            "Referer",
            format!("https://www.roblox.com/games/{}/", place_id),
        )
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    resp.headers()
        .get("rbx-authentication-ticket")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or("No auth ticket found".to_string())
}
#[tauri::command]
pub async fn quick_sign_create() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://apis.roblox.com/auth-token-service/v1/login/create")
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data)
}

#[tauri::command]
pub async fn quick_sign_poll(
    code: String,
    private_key: String,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);

    let mut csrf_token: Option<String> = None;

    while std::time::Instant::now() < deadline {
        let mut req = client
            .post("https://apis.roblox.com/auth-token-service/v1/login/status")
            .header("Origin", "https://www.roblox.com")
            .header("Referer", "https://www.roblox.com/")
            .json(&serde_json::json!({ "code": code, "privateKey": private_key }));

        if let Some(ref csrf) = csrf_token {
            req = req.header("X-CSRF-TOKEN", csrf);
        }

        let resp = req.send().await.map_err(|e| e.to_string())?;

        if resp.status() == 403 {
            if let Some(csrf) = resp.headers().get("x-csrf-token") {
                csrf_token = Some(csrf.to_str().unwrap_or("").to_string());
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let status = body["status"].as_str().unwrap_or("").to_string();

        if status == "Validated" {
            // exchange for cookie
            let cookie = perform_quick_sign_login(&code, &private_key).await?;
            return Ok(serde_json::json!({ "status": "Validated", "cookie": cookie }));
        }

        if status == "Cancelled" {
            return Ok(serde_json::json!({ "status": "Cancelled", "cookie": null }));
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }

    Ok(serde_json::json!({ "status": "TimedOut", "cookie": null }))
}

async fn perform_quick_sign_login(code: &str, private_key: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;

    let mut csrf_token: Option<String> = None;

    for _ in 0..3 {
        let mut req = client
            .post("https://auth.roblox.com/v2/login")
            .header("Origin", "https://www.roblox.com")
            .header("Referer", "https://www.roblox.com/")
            .json(&serde_json::json!({
                "ctype": "AuthToken",
                "cvalue": code,
                "password": private_key
            }));

        if let Some(ref csrf) = csrf_token {
            req = req.header("X-CSRF-TOKEN", csrf);
        }

        let resp = req.send().await.map_err(|e| e.to_string())?;

        if resp.status() == 403 {
            if let Some(csrf) = resp.headers().get("x-csrf-token") {
                csrf_token = Some(csrf.to_str().unwrap_or("").to_string());
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }

        if resp.status() == 200 {
            for header in resp.headers().get_all("set-cookie") {
                let val = header.to_str().unwrap_or("");
                if val.contains(".ROBLOSECURITY=") {
                    let start = val.find(".ROBLOSECURITY=").unwrap() + ".ROBLOSECURITY=".len();
                    let end = val[start..]
                        .find(';')
                        .map(|i| i + start)
                        .unwrap_or(val.len());
                    return Ok(val[start..end].to_string());
                }
            }
        }

        break;
    }

    Err("Could not get .ROBLOSECURITY cookie".to_string())
}
