use super::HingeClient;
use super::render::{render_profile, summarize_connection_initiation};
use super::serde_helpers::{
    attachment_from_value, parse_json_value_with_path, parse_ts, sanitize_component,
};
use crate::errors::HingeError;
use crate::logging::log_request;
use crate::models::{
    ExportChatInput, ExportChatResult, ExportedMediaFile, SendbirdChannelHandle,
    SendbirdGroupChannel, SendbirdMessage,
};
use crate::storage::Storage;
use chrono::{DateTime, Local, Utc};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::cmp::min;
use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

pub(super) const SENDBIRD_ACCEPT_LANGUAGE: &str = "en-GB";
pub(super) const SENDBIRD_REST_USER_AGENT: &str = "Jios/4.26.0";
pub(super) const SENDBIRD_WS_ORIGIN: &str = "https://web-sb-kr-7-704.sendbird.com";

impl<S: Storage + Clone> HingeClient<S> {
    pub(super) fn sendbird_header_value(&self) -> String {
        format!(
            "iOS,{},{},{}",
            self.settings.os_version,
            self.settings.sendbird_sdk_version,
            self.settings.sendbird_app_id
        )
    }

    pub(super) fn sendbird_user_agent_value(&self) -> String {
        format!("iOS/c{}///", self.settings.sendbird_sdk_version)
    }

    pub(super) fn sendbird_sdk_user_agent_value(&self) -> String {
        format!(
            "main_sdk_info=chat/ios/{}&device_os_platform=ios&os_version={}",
            self.settings.sendbird_sdk_version, self.settings.os_version
        )
    }

    pub(super) fn sendbird_headers(&self) -> Result<reqwest::header::HeaderMap, HingeError> {
        use reqwest::header::{HeaderMap, HeaderValue};
        let mut h = HeaderMap::new();
        h.insert("accept", HeaderValue::from_static("*/*"));
        h.insert("connection", HeaderValue::from_static("keep-alive"));
        h.insert(
            "accept-language",
            HeaderValue::from_static(SENDBIRD_ACCEPT_LANGUAGE),
        );
        h.insert(
            "x-session-key",
            HeaderValue::from_str(&self.session_id)
                .map_err(|e| HingeError::Http(format!("Invalid session key header: {}", e)))?,
        );
        h.insert(
            "x-device-id",
            HeaderValue::from_str(&self.device_id)
                .map_err(|e| HingeError::Http(format!("Invalid device id header: {}", e)))?,
        );
        h.insert(
            "x-install-id",
            HeaderValue::from_str(&self.install_id)
                .map_err(|e| HingeError::Http(format!("Invalid install id header: {}", e)))?,
        );
        h.insert(
            "sb-user-id",
            HeaderValue::from_str(
                self.hinge_auth
                    .as_ref()
                    .map(|token| token.identity_id.as_str())
                    .unwrap_or_default(),
            )
            .map_err(|e| HingeError::Http(format!("Invalid sb-user-id header: {}", e)))?,
        );
        if let Some(sb_auth) = &self.sendbird_auth {
            h.insert(
                "sb-access-token",
                HeaderValue::from_str(&sb_auth.token)
                    .map_err(|e| HingeError::Http(format!("Invalid sb-access-token: {}", e)))?,
            );
        }
        if let Some(session_key) = &self.sendbird_session_key {
            h.insert(
                "Session-Key",
                HeaderValue::from_str(session_key)
                    .map_err(|e| HingeError::Http(format!("Invalid session key: {}", e)))?,
            );
        }
        // Timestamp header present in logs
        let ts = chrono::Utc::now().timestamp_millis();
        h.insert(
            "Request-Sent-Timestamp",
            HeaderValue::from_str(&ts.to_string())
                .map_err(|e| HingeError::Http(format!("Invalid timestamp: {}", e)))?,
        );
        // SDK-identifying headers observed in logs
        h.insert(
            "SendBird",
            HeaderValue::from_str(&self.sendbird_header_value())
                .map_err(|e| HingeError::Http(format!("Invalid SendBird header: {}", e)))?,
        );
        h.insert(
            "SB-User-Agent",
            HeaderValue::from_str(&self.sendbird_user_agent_value())
                .map_err(|e| HingeError::Http(format!("Invalid SB-User-Agent: {}", e)))?,
        );
        h.insert(
            "SB-SDK-User-Agent",
            HeaderValue::from_str(&self.sendbird_sdk_user_agent_value())
                .map_err(|e| HingeError::Http(format!("Invalid SB-SDK-User-Agent: {}", e)))?,
        );
        h.insert(
            "user-agent",
            HeaderValue::from_static(SENDBIRD_REST_USER_AGENT),
        );
        Ok(h)
    }

    async fn sendbird_get(&self, path_and_query: &str) -> Result<reqwest::Response, HingeError> {
        let url = format!("{}/v3{}", self.settings.sendbird_api_url, path_and_query);
        let headers = self.sendbird_headers()?;
        log_request("GET", &url, &headers, None);
        let res = self.http.get(url).headers(headers.clone()).send().await?;
        log::info!("[sendbird] GET {} -> {}", path_and_query, res.status());
        Ok(res)
    }

    async fn ensure_sendbird_session(&mut self) -> Result<(), HingeError> {
        // If a WS is already connected, we're good
        if self.sendbird_ws_connected {
            return Ok(());
        }

        // Ensure we have Sendbird JWT from Hinge
        if self.sendbird_auth.is_none() {
            self.authenticate_with_sendbird().await?;
        }

        // Start and hold a single WS connection; capture LOGI and broadcast frames
        let (cmd_tx, broadcast_tx) = self.start_sendbird_ws().await?;
        self.sendbird_ws_cmd_tx = Some(cmd_tx);
        self.sendbird_ws_broadcast_tx = Some(broadcast_tx);
        self.sendbird_ws_connected = true;
        Ok(())
    }

    async fn start_sendbird_ws(
        &mut self,
    ) -> Result<
        (
            tokio::sync::mpsc::UnboundedSender<String>,
            tokio::sync::broadcast::Sender<String>,
        ),
        HingeError,
    > {
        let sb = self
            .sendbird_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("sendbird token missing".into()))?;
        let user_id = self
            .hinge_auth
            .as_ref()
            .map(|t| t.identity_id.clone())
            .unwrap_or_default();
        let ws_url = format!(
            "{}/?p=iOS&sv={}&pv={}&uikit_config=0&use_local_cache=0&include_extra_data=premium_feature_list,file_upload_size_limit,emoji_hash,application_attributes,notifications,message_template,ai_agent&include_poll_details=1&user_id={}&ai={}&pmce=1&expiring_session=0&config_ts=0",
            self.settings.sendbird_ws_url,
            self.settings.sendbird_sdk_version,
            self.settings.os_version,
            user_id,
            self.settings.sendbird_app_id
        );
        let ws_ts = chrono::Utc::now().timestamp_millis().to_string();
        let host = ws_url
            .trim_start_matches("wss://")
            .trim_start_matches("ws://")
            .split('/')
            .next()
            .unwrap_or("");
        let ws_key = tokio_tungstenite::tungstenite::handshake::client::generate_key();
        let mut builder = tokio_tungstenite::tungstenite::http::Request::builder().uri(&ws_url);
        if let Some(sk) = &self.sendbird_session_key {
            builder = builder.header("SENDBIRD-WS-AUTH", sk);
        } else {
            builder = builder.header("SENDBIRD-WS-TOKEN", sb.token.clone());
        }
        builder = builder
            .header("Accept", "*/*")
            .header("Accept-Encoding", "gzip, deflate")
            .header("Sec-WebSocket-Extensions", "permessage-deflate")
            .header("Sec-WebSocket-Key", &ws_key)
            .header("Sec-WebSocket-Version", "13")
            .header("Request-Sent-Timestamp", &ws_ts)
            .header("Host", host)
            .header("Origin", SENDBIRD_WS_ORIGIN)
            .header("Accept-Language", SENDBIRD_ACCEPT_LANGUAGE)
            .header("x-session-key", &self.session_id)
            .header("x-device-id", &self.device_id)
            .header("x-install-id", &self.install_id)
            .header("sb-user-id", &user_id)
            .header("sb-access-token", &sb.token)
            .header("SendBird", self.sendbird_header_value())
            .header("SB-User-Agent", self.sendbird_user_agent_value())
            .header("SB-SDK-User-Agent", self.sendbird_sdk_user_agent_value())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("User-Agent", &self.hinge_user_agent());
        // Log WS request (redacted)
        {
            let mut pairs: Vec<(String, String)> = Vec::new();
            if let Some(sk) = &self.sendbird_session_key {
                pairs.push(("SENDBIRD-WS-AUTH".into(), sk.clone()));
            } else {
                pairs.push(("SENDBIRD-WS-TOKEN".into(), sb.token.clone()));
            }
            pairs.push(("Accept".into(), "*/*".into()));
            pairs.push(("Accept-Encoding".into(), "gzip, deflate".into()));
            pairs.push((
                "Sec-WebSocket-Extensions".into(),
                "permessage-deflate".into(),
            ));
            pairs.push(("Accept-Language".into(), SENDBIRD_ACCEPT_LANGUAGE.into()));
            pairs.push(("Host".into(), host.to_string()));
            pairs.push(("Origin".into(), SENDBIRD_WS_ORIGIN.into()));
            pairs.push(("Sec-WebSocket-Key".into(), ws_key.clone()));
            pairs.push(("Sec-WebSocket-Version".into(), "13".into()));
            pairs.push(("Request-Sent-Timestamp".into(), ws_ts.clone()));
            pairs.push(("x-session-key".into(), self.session_id.clone()));
            pairs.push(("x-device-id".into(), self.device_id.clone()));
            pairs.push(("x-install-id".into(), self.install_id.clone()));
            pairs.push(("sb-user-id".into(), user_id.clone()));
            pairs.push(("sb-access-token".into(), sb.token.clone()));
            pairs.push(("SendBird".into(), self.sendbird_header_value()));
            pairs.push(("SB-User-Agent".into(), self.sendbird_user_agent_value()));
            pairs.push((
                "SB-SDK-User-Agent".into(),
                self.sendbird_sdk_user_agent_value(),
            ));
            pairs.push(("Connection".into(), "Upgrade".into()));
            pairs.push(("Upgrade".into(), "websocket".into()));
            pairs.push(("User-Agent".into(), self.hinge_user_agent()));
            log::info!("━━━━━━━━━━ WS REQUEST ━━━━━━━━━━");
            log::info!("GET {}", ws_url);
            log::debug!("Headers:\n{}", crate::logging::format_ws_headers(&pairs));
            log::info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }

        let req: tokio_tungstenite::tungstenite::http::Request<()> = builder
            .body(())
            .map_err(|e| HingeError::Http(e.to_string()))?;
        let (ws, _resp) = tokio_tungstenite::connect_async(req)
            .await
            .map_err(|e| HingeError::Http(e.to_string()))?;
        let (write_half, mut read_half) = ws.split();
        let write_half = std::sync::Arc::new(tokio::sync::Mutex::new(write_half));

        let (tx_cmd, mut rx_cmd) = tokio::sync::mpsc::unbounded_channel::<String>();
        let (tx_broadcast, _rx_broadcast) = tokio::sync::broadcast::channel::<String>(1024);
        let (sk_tx, sk_rx) = tokio::sync::oneshot::channel::<String>();

        // Reader: capture LOGI to set Session-Key; forward frames; respond to Ping
        {
            let write_for_pong = write_half.clone();
            let tx_broadcast_c = tx_broadcast.clone();
            let pending_requests = self.sendbird_ws_pending_requests.clone();
            tokio::spawn(async move {
                let mut sk_tx_opt = Some(sk_tx);
                while let Some(msg) = read_half.next().await {
                    match msg {
                        Ok(Message::Ping(_)) => {
                            let mut w = write_for_pong.lock().await;
                            let _ = w.send(Message::Pong(Vec::new().into())).await;
                        }
                        Ok(Message::Text(t)) => {
                            let t = t.to_string();
                            // Handle LOGI frame - extract session key
                            if t.starts_with("LOGI")
                                && let Some(start) = t.find('{')
                                && let Ok(val) =
                                    serde_json::from_str::<serde_json::Value>(&t[start..])
                            {
                                if let Some(k) = crate::ws::sendbird_logi_session_key(&val) {
                                    let _ = tx_broadcast_c.send(format!("__SESSION_KEY__:{}", k));
                                    if let Some(tx) = sk_tx_opt.take() {
                                        let _ = tx.send(k.to_string());
                                    }
                                }
                                // Log important LOGI fields
                                log::info!(
                                    "[sendbird ws] LOGI received - user_id: {}, ping_interval: {}, pong_timeout: {}",
                                    val.get("user_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown"),
                                    val.get("ping_interval")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0),
                                    val.get("pong_timeout")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0)
                                );
                            }
                            // Handle PING frame - respond with PONG
                            else if t.starts_with("PING") {
                                log::debug!("[sendbird ws] Received PING, sending PONG");
                                if let Some(start) = t.find('{')
                                    && let Ok(_val) =
                                        serde_json::from_str::<serde_json::Value>(&t[start..])
                                {
                                    let pong_response = json!({
                                        "sts": chrono::Utc::now().timestamp_millis(),
                                        "ts": chrono::Utc::now().timestamp_millis()
                                    });
                                    let pong_msg = format!("PONG{}", pong_response);
                                    let mut w = write_for_pong.lock().await;
                                    let _ = w.send(Message::Text(pong_msg.into())).await;
                                }
                            }
                            // Handle READ acknowledgments
                            else if t.starts_with("READ") && t.contains("channel_id") {
                                log::debug!("[sendbird ws] Received READ acknowledgment");
                                // Check if this is a response to a pending request
                                if let Some(start) = t.find('{')
                                    && let Ok(val) =
                                        serde_json::from_str::<serde_json::Value>(&t[start..])
                                    && let Some(req_id) = val.get("req_id").and_then(|v| v.as_str())
                                {
                                    let mut pending = pending_requests.lock().await;
                                    if let Some(tx) = pending.remove(req_id) {
                                        let _ = tx.send(val.clone());
                                        log::debug!(
                                            "[sendbird ws] Matched READ response for req_id: {}",
                                            req_id
                                        );
                                    }
                                }
                            }
                            // Broadcast all messages to subscribers
                            // Parse SYEV (system event) frames for structured typing events
                            if t.starts_with("SYEV")
                                && let Some(start) = t.find('{')
                                && let Ok(val) =
                                    serde_json::from_str::<serde_json::Value>(&t[start..])
                            {
                                // Broadcast raw
                                let _ = tx_broadcast_c.send(t.clone());
                                // Try to parse into structured model
                                if let Ok(evt) = serde_json::from_value::<
                                    crate::models::SendbirdSyevEvent,
                                >(val.clone())
                                {
                                    // Typing start/end logging
                                    if evt.cat
                                        == crate::models::SendbirdSyevEvent::CATEGORY_TYPING_START
                                    {
                                        log::debug!(
                                            "[sendbird ws] SYEV typing start user={} channel={}",
                                            evt.data
                                                .as_ref()
                                                .map(|u| u.user_id.as_str())
                                                .unwrap_or("unknown"),
                                            evt.channel_url
                                        );
                                    } else if evt.cat
                                        == crate::models::SendbirdSyevEvent::CATEGORY_TYPING_END
                                    {
                                        log::debug!(
                                            "[sendbird ws] SYEV typing end user={} channel={}",
                                            evt.data
                                                .as_ref()
                                                .map(|u| u.user_id.as_str())
                                                .unwrap_or("unknown"),
                                            evt.channel_url
                                        );
                                    }
                                    // Broadcast structured event for consumers
                                    if let Ok(json_evt) = serde_json::to_string(&evt) {
                                        let _ =
                                            tx_broadcast_c.send(format!("__SYEV__:{}", json_evt));
                                    }
                                    continue;
                                }
                            }
                            let _ = tx_broadcast_c.send(t);
                        }
                        Ok(Message::Binary(b)) => {
                            let _ = tx_broadcast_c.send(String::from_utf8_lossy(&b).into_owned());
                        }
                        Ok(Message::Pong(_)) => {}
                        Ok(Message::Close(frame)) => {
                            if let Some(cf) = frame {
                                let code_u16: u16 = cf.code.into();

                                // Analyze if this could be time-based
                                // Your observation: LOGI at 8:03:11.826, Close at 8:03:12.526, code 55409
                                // Theory: The code might be derived from timestamp
                                let now = chrono::Utc::now();
                                let ms_timestamp = now.timestamp_millis();

                                // Various time-based calculations that might match
                                let last_5_of_ms = (ms_timestamp % 100000) as u16;
                                let last_5_of_seconds = ((ms_timestamp / 1000) % 100000) as u16;
                                let seconds_today = (now.timestamp() % 86400) as u16;
                                let ms_today = ((now.timestamp() % 86400) * 1000
                                    + now.timestamp_subsec_millis() as i64)
                                    as u32;
                                let ms_today_mod = (ms_today % 65536) as u16; // Fit in u16

                                log::debug!(
                                    "[sendbird ws] Time analysis - code: {}, last5_ms: {}, last5_sec: {}, sec_today: {}, ms_today_mod: {}",
                                    code_u16,
                                    last_5_of_ms,
                                    last_5_of_seconds,
                                    seconds_today,
                                    ms_today_mod
                                );

                                let code_desc = match code_u16 {
                                    // Standard WebSocket close codes (1000-4999)
                                    1000 => "Normal closure",
                                    1001 => "Going away",
                                    1002 => "Protocol error",
                                    1003 => "Unsupported data",
                                    1006 => "Abnormal closure",
                                    1008 => "Policy violation",
                                    1009 => "Message too big",
                                    1010 => "Mandatory extension",
                                    1011 => "Internal server error",
                                    1015 => "TLS handshake failure",
                                    // Sendbird appears to use dynamic codes possibly derived from timestamps
                                    _ if code_u16 >= 10000 => {
                                        "Sendbird dynamic code (possibly time-derived)"
                                    }
                                    _ => "Non-standard close code",
                                };
                                log::warn!(
                                    "[sendbird ws] Connection closed - code: {} ({}), reason: {}",
                                    code_u16,
                                    code_desc,
                                    cf.reason
                                );
                                let _ = tx_broadcast_c
                                    .send(format!("__CLOSE__:{}:{}", code_u16, cf.reason));

                                // Since Sendbird uses dynamic codes, we can't determine reconnection strategy from the code
                                // The reason string might be more informative than the code itself
                                if !cf.reason.is_empty() {
                                    log::info!(
                                        "[sendbird ws] Close reason provided: {}",
                                        cf.reason
                                    );
                                }
                            } else {
                                log::warn!("[sendbird ws] Connection closed without frame");
                                let _ = tx_broadcast_c.send("__CLOSE__".into());
                            }
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("[sendbird ws] WebSocket error: {}", e);
                            let _ = tx_broadcast_c.send(format!("__ERROR__:{}", e));
                            break;
                        }
                    }
                }
            });
        }

        // Writer: forward commands to WS
        {
            let write_for_cmds = write_half.clone();
            tokio::spawn(async move {
                while let Some(cmd) = rx_cmd.recv().await {
                    let mut w = write_for_cmds.lock().await;

                    // Check if this is a special close command
                    if cmd.starts_with("__CLOSE__:") {
                        // Parse the close code and reason
                        let parts: Vec<&str> = cmd
                            .strip_prefix("__CLOSE__:")
                            .unwrap_or("")
                            .split(':')
                            .collect();
                        let code = parts
                            .first()
                            .and_then(|s| s.parse::<u16>().ok())
                            .unwrap_or(1000);
                        let reason = parts.get(1).unwrap_or(&"").to_string();

                        // Send WebSocket Close frame
                        let close_frame = tokio_tungstenite::tungstenite::protocol::CloseFrame {
                            code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::from(code),
                            reason: reason.into(),
                        };
                        let _ = w.send(Message::Close(Some(close_frame))).await;
                        break; // Stop the writer task after sending close
                    } else {
                        // Regular text command
                        let _ = w.send(Message::Text(cmd.into())).await;
                    }
                }
            });
        }

        // Wait for LOGI to deliver session key before returning so REST can use it
        if let Ok(k) = sk_rx.await {
            self.sendbird_session_key = Some(k);
            log::info!("[sendbird] Session-Key captured");
            if let Some(path) = &self.session_path {
                let _ = self.save_session(path);
            }
        } else {
            log::warn!("Sendbird LOGI not received before startup return");
        }
        Ok((tx_cmd, tx_broadcast))
    }

    pub async fn sendbird_list_my_group_channels(
        &mut self,
        user_id: &str,
        limit: usize,
    ) -> Result<serde_json::Value, HingeError> {
        self.ensure_sendbird_session().await?;
        let q = format!(
            "/users/{}/my_group_channels?&include_left_channel=false&member_state_filter=all&super_mode=all&show_latest_message=false&show_pinned_messages=false&unread_filter=all&show_delivery_receipt=true&show_conversation=false&show_member=true&show_empty=true&limit={}&user_id={}&is_feed_channel=false&order=latest_last_message&hidden_mode=unhidden_only&distinct_mode=all&show_read_receipt=true&show_metadata=true&is_explicit_request=true&show_frozen=true&public_mode=all&include_chat_notification=false",
            user_id, limit, user_id
        );
        let res = self.sendbird_get(&q).await?;
        self.parse_response(res).await
    }

    pub async fn sendbird_list_channels_typed(
        &mut self,
        limit: usize,
    ) -> Result<crate::models::SendbirdChannelsResponse, HingeError> {
        let user_id = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?
            .identity_id
            .clone();
        let limit = limit.clamp(1, 200);
        let raw = self
            .sendbird_list_my_group_channels(&user_id, limit)
            .await?;
        parse_json_value_with_path(raw).map_err(|e| {
            HingeError::Serde(format!("Failed to parse Sendbird channels response: {}", e))
        })
    }

    pub async fn sendbird_get_channel(
        &mut self,
        channel_url: &str,
    ) -> Result<serde_json::Value, HingeError> {
        self.ensure_sendbird_session().await?;
        let q = format!(
            "/sdk/group_channels/{}?&is_feed_channel=false&show_latest_message=false&show_metadata=false&show_empty=false&show_member=true&show_frozen=false&show_read_receipt=true&show_pinned_messages=false&include_chat_notification=false&show_delivery_receipt=true&show_conversation=true",
            channel_url
        );
        let res = self.sendbird_get(&q).await?;
        self.parse_response(res).await
    }

    pub async fn sendbird_get_channel_typed(
        &mut self,
        channel_url: &str,
    ) -> Result<SendbirdGroupChannel, HingeError> {
        let value = self.sendbird_get_channel(channel_url).await?;
        parse_json_value_with_path(value)
            .map_err(|e| HingeError::Serde(format!("Failed to parse channel: {}", e)))
    }

    pub async fn sendbird_get_messages(
        &mut self,
        channel_url: &str,
        message_ts: i64,
        prev_limit: usize,
    ) -> Result<crate::models::SendbirdMessagesResponse, HingeError> {
        self.ensure_sendbird_session().await?;
        let q = format!(
            "/group_channels/{}/messages?&include_reply_type=all&sdk_source=external_legacy&with_sorted_meta_array=true&message_ts={}&is_sdk=true&include_reactions_summary=true&include_parent_message_info=false&reverse=true&prev_limit={}&custom_types=%2A&include=false&next_limit=0&include_poll_details=true&show_subchannel_messages_only=false&include_thread_info=false",
            channel_url, message_ts, prev_limit
        );
        let res = self.sendbird_get(&q).await?;
        self.parse_response(res).await
    }

    pub async fn sendbird_get_full_messages(
        &mut self,
        channel_url: &str,
    ) -> Result<Vec<SendbirdMessage>, HingeError> {
        self.ensure_sendbird_session().await?;
        const PAGE_SIZE: usize = 120;
        let mut anchor = chrono::Utc::now().timestamp_millis();
        let mut seen: HashSet<String> = HashSet::new();
        let mut collected: Vec<(i64, SendbirdMessage)> = Vec::new();

        loop {
            let batch = self
                .sendbird_get_messages(channel_url, anchor, PAGE_SIZE)
                .await?;
            if batch.messages.is_empty() {
                break;
            }
            let mut earliest = anchor;
            let mut added = 0usize;
            for message in batch.messages {
                if seen.insert(message.message_id.clone()) {
                    let ts = parse_ts(&message.created_at).unwrap_or(anchor);
                    earliest = min(earliest, ts.saturating_sub(1));
                    collected.push((ts, message));
                    added += 1;
                }
            }
            if added == 0 {
                break;
            }
            if earliest >= anchor || earliest <= 0 {
                break;
            }
            anchor = earliest;
            if collected.len() >= 4000 {
                log::warn!(
                    "[sendbird] Stopping history fetch after {} messages to avoid huge exports",
                    collected.len()
                );
                break;
            }
        }

        collected.sort_by_key(|(ts, _)| *ts);
        Ok(collected.into_iter().map(|(_, msg)| msg).collect())
    }

    pub async fn export_chat(
        &mut self,
        input: ExportChatInput,
    ) -> Result<ExportChatResult, HingeError> {
        self.ensure_sendbird_session().await?;
        let auth = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?
            .clone();
        let self_user_id = auth.identity_id.clone();

        let prompts_manager = match self.fetch_prompts_manager().await {
            Ok(mgr) => Some(mgr),
            Err(err) => {
                log::warn!("Failed to prefetch prompts for export: {}", err);
                None
            }
        };

        let channel = self.sendbird_get_channel_typed(&input.channel_url).await?;
        let partner = channel
            .members
            .iter()
            .find(|member| !member.user_id.is_empty() && member.user_id != self_user_id)
            .cloned()
            .ok_or_else(|| HingeError::Http("unable to determine conversation partner".into()))?;

        let peer_id = partner.user_id.clone();
        let profile = self
            .get_profiles(vec![peer_id.clone()])
            .await?
            .into_iter()
            .next();
        let profile_content = self
            .get_profile_content(vec![peer_id.clone()])
            .await?
            .into_iter()
            .next();

        let display_name = profile
            .as_ref()
            .map(|p| p.profile.first_name.clone())
            .filter(|name| !name.trim().is_empty())
            .or_else(|| {
                if !partner.nickname.trim().is_empty() {
                    Some(partner.nickname.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| peer_id.clone());

        let age_label = profile
            .as_ref()
            .and_then(|p| p.profile.age)
            .map(|age| age.to_string())
            .unwrap_or_else(|| "Unknown age".to_string());

        let initiation_summary_lines = if let Some(lines) = input.initiation_summary_lines.clone() {
            if lines.is_empty() { None } else { Some(lines) }
        } else {
            match self.get_connections_v2().await {
                Ok(resp) => resp
                    .connections
                    .into_iter()
                    .find(|conn| {
                        let initiator = conn.initiator_id.trim();
                        let subject = conn.subject_id.trim();
                        (!initiator.is_empty() && initiator == self_user_id && subject == peer_id)
                            || (!subject.is_empty()
                                && subject == self_user_id
                                && initiator == peer_id)
                    })
                    .and_then(|conn| {
                        summarize_connection_initiation(
                            &conn,
                            &self_user_id,
                            &peer_id,
                            &display_name,
                        )
                    }),
                Err(err) => {
                    log::warn!(
                        "Failed to fetch connections for initiation summary: {}",
                        err
                    );
                    None
                }
            }
        };

        let base_dir = Path::new(&input.output_dir);
        let export_dir = base_dir.to_path_buf();
        fs::create_dir_all(&export_dir).map_err(|e| HingeError::Storage(e.to_string()))?;

        let messages = self.sendbird_get_full_messages(&input.channel_url).await?;

        let mut transcript = String::new();
        writeln!(transcript, "Chat with {} ({})", display_name, age_label).ok();
        writeln!(transcript, "Channel: {}", input.channel_url).ok();
        writeln!(transcript, "Exported at {}", Utc::now().to_rfc3339()).ok();
        if let Some(lines) = &initiation_summary_lines {
            for line in lines {
                writeln!(transcript, "{line}").ok();
            }
        }
        transcript.push('\n');

        let mut media_files: Vec<ExportedMediaFile> = Vec::new();

        if input.include_media
            && let Some(ref content) = profile_content
        {
            for (idx, photo) in content.content.photos.iter().enumerate() {
                let mut file_name = format!("profile_photo_{}", idx + 1);
                if let Some(ext) = photo
                    .url
                    .split('.')
                    .next_back()
                    .filter(|part| part.len() <= 5)
                {
                    file_name.push('.');
                    file_name.push_str(ext);
                }
                let sanitized = sanitize_component(&file_name);
                let target_path = export_dir.join(&sanitized);
                let bytes = self.http_get_bytes(&photo.url).await?;
                fs::write(&target_path, &bytes).map_err(|e| HingeError::Storage(e.to_string()))?;
                media_files.push(ExportedMediaFile {
                    message_id: format!("profile_photo_{}", idx + 1),
                    file_name: sanitized.clone(),
                    file_path: target_path.to_string_lossy().to_string(),
                });
            }
        }

        for message in &messages {
            let timestamp = parse_ts(&message.created_at).unwrap_or(0);
            let local_time: DateTime<Local> = DateTime::<Utc>::from_timestamp_millis(timestamp)
                .map(|dt| dt.with_timezone(&Local))
                .unwrap_or_else(Local::now);
            let sender = if message.user.user_id == self_user_id {
                "You".to_string()
            } else if !message.user.nickname.is_empty() {
                message.user.nickname.clone()
            } else {
                display_name.clone()
            };
            let body = if !message.message.trim().is_empty() {
                message.message.clone()
            } else if !message.data.trim().is_empty() {
                message.data.clone()
            } else if !message.custom_type.trim().is_empty() {
                format!("[{} message]", message.custom_type)
            } else {
                "[non-text message]".into()
            };

            writeln!(
                transcript,
                "{} - {}: {}",
                local_time.format("%Y-%m-%d %H:%M:%S"),
                sender,
                body
            )
            .ok();

            if input.include_media
                && let Some((url, name)) = attachment_from_value(&message.file)
            {
                let sanitized = sanitize_component(&name);
                let target_path = export_dir.join(&sanitized);
                let bytes = self.http_get_bytes(&url).await?;
                fs::write(&target_path, &bytes).map_err(|e| HingeError::Storage(e.to_string()))?;
                writeln!(transcript, "    [Saved attachment: {}]", sanitized).ok();
                media_files.push(ExportedMediaFile {
                    message_id: message.message_id.clone(),
                    file_name: sanitized.clone(),
                    file_path: target_path.to_string_lossy().to_string(),
                });
            }
        }

        let transcript_path = export_dir.join("chat.txt");
        fs::write(&transcript_path, transcript).map_err(|e| HingeError::Storage(e.to_string()))?;

        let profile_text = render_profile(
            profile.as_ref(),
            profile_content.as_ref(),
            prompts_manager.as_ref(),
        );
        let profile_path = if !profile_text.trim().is_empty() {
            let path = export_dir.join("profile.txt");
            fs::write(&path, profile_text).map_err(|e| HingeError::Storage(e.to_string()))?;
            Some(path)
        } else {
            None
        };

        Ok(ExportChatResult {
            folder_path: export_dir.to_string_lossy().to_string(),
            transcript_path: transcript_path.to_string_lossy().to_string(),
            profile_path: profile_path.map(|p| p.to_string_lossy().to_string()),
            message_count: messages.len().min(i32::MAX as usize) as i32,
            media_files,
        })
    }

    pub async fn sendbird_create_distinct_dm(
        &mut self,
        self_user_id: &str,
        peer_user_id: &str,
        data_mm: i32,
    ) -> Result<serde_json::Value, HingeError> {
        self.ensure_sendbird_session().await?;
        let payload = json!({
            "is_ephemeral": false,
            "is_exclusive": false,
            "data": format!("{{\n  \"mm\" : {}\n}}", data_mm),
            "user_ids": [peer_user_id, self_user_id],
            "is_super": false,
            "is_distinct": true,
            "strict": false,
            "is_broadcast": false,
            "message_survival_seconds": -1,
            "is_public": false
        });
        let url = format!(
            "{}/v3{}",
            self.settings.sendbird_api_url, "/group_channels?"
        );
        let mut headers = self.sendbird_headers()?;
        use reqwest::header::HeaderValue;
        headers.insert(
            "content-type",
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        log_request("POST", &url, &headers, Some(&payload));
        let res = self
            .http
            .post(url)
            .headers(headers)
            .body(serde_json::to_string(&payload).unwrap_or_default())
            .send()
            .await?;
        log::info!("[sendbird] POST /group_channels -> {}", res.status());
        self.parse_response(res).await
    }

    pub async fn sendbird_get_or_create_dm_channel(
        &mut self,
        self_user_id: &str,
        peer_user_id: &str,
    ) -> Result<String, HingeError> {
        // Try find existing channel containing exactly the two members
        let q = format!(
            "/users/{}/my_group_channels?&members_exactly_in={}&show_latest_message=false&distinct_mode=all&hidden_mode=unhidden_only&show_pinned_messages=false&show_metadata=true&member_state_filter=all&user_id={}&is_explicit_request=true&public_mode=all&include_left_channel=false&show_conversation=false&show_frozen=true&is_feed_channel=false&show_delivery_receipt=true&unread_filter=all&super_mode=all&show_member=true&show_read_receipt=true&order=chronological&show_empty=true&include_chat_notification=false&limit=1",
            self_user_id, peer_user_id, self_user_id
        );
        self.ensure_sendbird_session().await?;
        let res = self.sendbird_get(&q).await?;
        let v: serde_json::Value = self.parse_response(res).await?;
        if let Some(url) = v
            .get("channels")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("channel_url"))
            .and_then(|s| s.as_str())
        {
            return Ok(url.to_string());
        }
        let created = self
            .sendbird_create_distinct_dm(self_user_id, peer_user_id, 1)
            .await?;
        let url = created
            .get("channel_url")
            .and_then(|s| s.as_str())
            .ok_or_else(|| HingeError::Http("missing channel_url in create response".into()))?;
        Ok(url.to_string())
    }

    pub async fn ensure_sendbird_channel_with(
        &mut self,
        peer_user_id: &str,
    ) -> Result<SendbirdChannelHandle, HingeError> {
        let self_user_id = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?
            .identity_id
            .clone();
        let channel_url = self
            .sendbird_get_or_create_dm_channel(&self_user_id, peer_user_id)
            .await?;
        Ok(SendbirdChannelHandle { channel_url })
    }

    pub async fn sendbird_init_flow(&mut self) -> Result<serde_json::Value, HingeError> {
        // Health probe, user update is done by Hinge; we just list channels for the current user
        self.ensure_sendbird_session().await?;
        let user_id = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?
            .identity_id
            .clone();
        let res = self.sendbird_list_my_group_channels(&user_id, 20).await?;
        Ok(res)
    }

    /// Return Sendbird credentials for the JS client (appId and token), ensuring auth
    pub async fn sendbird_creds(&mut self) -> Result<serde_json::Value, HingeError> {
        // Ensure we have Sendbird JWT from Hinge but do not start WS
        if self.sendbird_auth.is_none() {
            self.authenticate_with_sendbird().await?;
        }
        let app_id = self.settings.sendbird_app_id.clone();
        let token = self
            .sendbird_auth
            .as_ref()
            .map(|t| t.token.clone())
            .unwrap_or_default();
        Ok(serde_json::json!({
            "appId": app_id,
            "token": token
        }))
    }

    // Open Sendbird WS and yield frames to a channel; also auto-respond to pings and allow READ commands
    pub async fn sendbird_ws_subscribe(
        &mut self,
    ) -> Result<
        (
            tokio::sync::mpsc::UnboundedSender<String>,
            tokio::sync::broadcast::Receiver<String>,
        ),
        HingeError,
    > {
        self.ensure_sendbird_session().await?;
        let cmd = self
            .sendbird_ws_cmd_tx
            .as_ref()
            .cloned()
            .ok_or_else(|| HingeError::Http("sendbird ws not started".into()))?;
        let tx = self
            .sendbird_ws_broadcast_tx
            .as_ref()
            .cloned()
            .ok_or_else(|| HingeError::Http("sendbird ws broadcast not available".into()))?;
        let rx = tx.subscribe();
        Ok((cmd, rx))
    }

    /// Send a raw command to the Sendbird WebSocket
    pub async fn sendbird_ws_send_command(&mut self, command: String) -> Result<(), HingeError> {
        self.ensure_sendbird_session().await?;
        let tx = self
            .sendbird_ws_cmd_tx
            .as_ref()
            .cloned()
            .ok_or_else(|| HingeError::Http("sendbird ws not started".into()))?;
        tx.send(command)
            .map_err(|e| HingeError::Http(format!("Failed to send WS command: {}", e)))?;
        Ok(())
    }

    /// Send a READ acknowledgment for a Sendbird channel (fire and forget)
    pub async fn sendbird_ws_send_read(&mut self, channel_url: &str) -> Result<(), HingeError> {
        let req_id = Uuid::new_v4().to_string().to_uppercase();
        let read_command = format!(
            r#"READ{{"req_id":"{}","channel_url":"{}"}}"#,
            req_id, channel_url
        );
        self.sendbird_ws_send_command(read_command).await
    }

    /// Send a READ acknowledgment and wait for the response
    pub async fn sendbird_ws_send_read_and_wait(
        &mut self,
        channel_url: &str,
    ) -> Result<crate::models::SendbirdReadResponse, HingeError> {
        self.ensure_sendbird_session().await?;

        // Generate request ID
        let req_id = Uuid::new_v4().to_string().to_uppercase();

        // Create oneshot channel for response
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Register the pending request
        {
            let mut pending = self.sendbird_ws_pending_requests.lock().await;
            pending.insert(req_id.clone(), tx);
        }

        // Send the READ command
        let read_command = format!(
            r#"READ{{"req_id":"{}","channel_url":"{}"}}"#,
            req_id, channel_url
        );
        self.sendbird_ws_send_command(read_command).await?;

        // Wait for response with timeout
        match tokio::time::timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(response)) => {
                // Parse the JSON response into our typed model
                parse_json_value_with_path(response)
                    .map_err(|e| HingeError::Http(format!("Failed to parse READ response: {}", e)))
            }
            Ok(Err(_)) => {
                // Channel was dropped, clean up
                let mut pending = self.sendbird_ws_pending_requests.lock().await;
                pending.remove(&req_id);
                Err(HingeError::Http("READ response channel dropped".into()))
            }
            Err(_) => {
                // Timeout, clean up
                let mut pending = self.sendbird_ws_pending_requests.lock().await;
                pending.remove(&req_id);
                Err(HingeError::Http("READ response timeout".into()))
            }
        }
    }

    /// Send a PING to keep the WebSocket alive
    pub async fn sendbird_ws_send_ping(&mut self) -> Result<(), HingeError> {
        let req_id = Uuid::new_v4().to_string().to_uppercase();
        let ping_command = format!(r#"PING{{"req_id":"{}"}}"#, req_id);
        self.sendbird_ws_send_command(ping_command).await
    }

    /// Send a TPST (Typing Start) command - fire and forget
    pub async fn sendbird_ws_send_typing_start(
        &mut self,
        channel_url: &str,
    ) -> Result<(), HingeError> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let tpst_command = format!(
            r#"TPST{{"req_id":null,"channel_url":"{}","time":{}}}"#,
            channel_url, timestamp
        );
        self.sendbird_ws_send_command(tpst_command).await
    }

    /// Send a TPEN (Typing End) command - fire and forget
    pub async fn sendbird_ws_send_typing_end(
        &mut self,
        channel_url: &str,
    ) -> Result<(), HingeError> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let tpen_command = format!(
            r#"TPEN{{"req_id":null,"channel_url":"{}","time":{}}}"#,
            channel_url, timestamp
        );
        self.sendbird_ws_send_command(tpen_command).await
    }

    /// Send an ENTR (Enter Channel) command - fire and forget
    pub async fn sendbird_ws_send_enter_channel(
        &mut self,
        channel_url: &str,
    ) -> Result<(), HingeError> {
        let entr_command = format!(r#"ENTR{{"req_id":null,"channel_url":"{}"}}"#, channel_url);
        self.sendbird_ws_send_command(entr_command).await
    }

    /// Send an EXIT (Exit Channel) command - fire and forget
    pub async fn sendbird_ws_send_exit_channel(
        &mut self,
        channel_url: &str,
    ) -> Result<(), HingeError> {
        let exit_command = format!(r#"EXIT{{"req_id":null,"channel_url":"{}"}}"#, channel_url);
        self.sendbird_ws_send_command(exit_command).await
    }

    /// Send a MACK (Message Acknowledgment) command - fire and forget
    pub async fn sendbird_ws_send_message_ack(
        &mut self,
        channel_url: &str,
        message_id: &str,
    ) -> Result<(), HingeError> {
        let mack_command = format!(
            r#"MACK{{"req_id":null,"channel_url":"{}","msg_id":"{}"}}"#,
            channel_url, message_id
        );
        self.sendbird_ws_send_command(mack_command).await
    }

    /// Close the WebSocket connection with a specific code
    pub async fn sendbird_ws_close(
        &mut self,
        code: Option<u16>,
        reason: Option<String>,
    ) -> Result<(), HingeError> {
        // Send close frame if we have a command channel
        if let Some(ref tx) = self.sendbird_ws_cmd_tx {
            // Sendbird uses custom close codes like 40909
            let close_code = code.unwrap_or(1000); // 1000 = Normal Closure
            let close_reason = reason.unwrap_or_else(|| "Client initiated close".to_string());

            // Send a close command through the channel
            // The writer task will handle converting this to a proper WebSocket Close frame
            let close_command = format!("__CLOSE__:{}:{}", close_code, close_reason);
            let _ = tx.send(close_command);

            log::info!(
                "[sendbird ws] Closing connection with code {} reason: {}",
                close_code,
                close_reason
            );
        }

        // Clear our state
        self.sendbird_ws_cmd_tx = None;
        self.sendbird_ws_broadcast_tx = None;
        self.sendbird_ws_connected = false;

        // Clear any pending requests
        let mut pending = self.sendbird_ws_pending_requests.lock().await;
        pending.clear();

        Ok(())
    }

    /// Check if WebSocket is connected and reconnect if needed
    pub async fn sendbird_ws_ensure_connected(&mut self) -> Result<bool, HingeError> {
        // Check if we have an active WebSocket connection
        if self.sendbird_ws_cmd_tx.is_some() {
            // Try to send a ping to verify connection is alive
            if self.sendbird_ws_send_ping().await.is_ok() {
                return Ok(true);
            }
        }

        // Connection is not active, clear the old state
        self.sendbird_ws_cmd_tx = None;
        self.sendbird_ws_broadcast_tx = None;

        // Try to reconnect
        log::info!("[sendbird ws] Reconnecting WebSocket...");
        self.start_sendbird_ws().await?;
        Ok(true)
    }
}
