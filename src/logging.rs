/// Library crates should not install a process-global logger.
///
/// Applications can install `tracing_subscriber`, `env_logger`, or any other
/// logger before constructing the client.
pub fn init_logger() {}

/// Format HTTP headers for logging (hiding sensitive data)
pub fn format_headers(headers: &reqwest::header::HeaderMap) -> String {
    let mut output = String::new();
    for (name, value) in headers.iter() {
        let value_str = if name == "authorization" {
            "Bearer ***REDACTED***".to_string()
        } else if name == "x-session-id" || name == "x-device-id" || name == "x-install-id" {
            format!(
                "***{}",
                &value
                    .to_str()
                    .unwrap_or("")
                    .chars()
                    .rev()
                    .take(4)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
            )
        } else {
            value.to_str().unwrap_or("").to_string()
        };
        output.push_str(&format!("  {}: {}\n", name.as_str(), value_str));
    }
    output
}

/// Format WebSocket headers for logging (hiding sensitive data)
pub fn format_ws_headers(headers: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    let mut output = String::new();
    for (name, value) in headers.iter() {
        let n = name.as_ref();
        let v = value.as_ref();
        let value_str = if n.eq_ignore_ascii_case("SENDBIRD-WS-AUTH")
            || n.eq_ignore_ascii_case("SENDBIRD-WS-TOKEN")
        {
            "***REDACTED***".to_string()
        } else if n.eq_ignore_ascii_case("Cookie") {
            "***".to_string()
        } else {
            v.to_string()
        };
        output.push_str(&format!("  {}: {}\n", n, value_str));
    }
    output
}

/// Format JSON for pretty logging
pub fn format_json(json: &serde_json::Value) -> String {
    serde_json::to_string_pretty(json).unwrap_or_else(|_| "Invalid JSON".to_string())
}

/// Log HTTP request details
pub fn log_request(
    method: &str,
    url: &str,
    headers: &reqwest::header::HeaderMap,
    body: Option<&serde_json::Value>,
) {
    log::info!("━━━━━━━━━━ HTTP REQUEST ━━━━━━━━━━");
    log::info!("{} {}", method, url);
    log::debug!("Headers:");
    log::debug!("{}", format_headers(headers));

    if let Some(body) = body {
        log::debug!("Body:");
        log::debug!("{}", format_json(body));
    }
    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

/// Log HTTP response details
pub fn log_response(
    status: reqwest::StatusCode,
    headers: &reqwest::header::HeaderMap,
    body: Option<&serde_json::Value>,
) {
    log::info!("━━━━━━━━━━ HTTP RESPONSE ━━━━━━━━━━");
    log::info!("Status: {}", status);
    log::debug!("Headers:");
    // Log each header on its own line so the logger prefix is consistent
    for (name, value) in headers.iter() {
        let printable = value.to_str().unwrap_or("");
        let value_str = if name == "authorization" {
            "Bearer ***REDACTED***".to_string()
        } else if name == "x-session-id" || name == "x-device-id" || name == "x-install-id" {
            format!(
                "***{}",
                printable
                    .chars()
                    .rev()
                    .take(4)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
            )
        } else {
            printable.to_string()
        };
        log::debug!("  {}: {}", name.as_str(), value_str);
    }

    if let Some(body) = body {
        log::debug!("Body:");
        let formatted = format_json(body);
        // Print full body (no truncation) for clarity during debugging
        log::debug!("{}", formatted);
    }
    log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
