use crate::models::{HingeAuthToken, RecommendationSubject, SendbirdAuthToken};
use crate::settings::Settings;
use crate::storage::{SecretStore, Storage};
use reqwest::Client as Http;
use std::path::PathBuf;
use std::time::Instant;
use uuid::Uuid;

mod auth;
mod chat;
mod connections;
mod likes;
mod payload;
mod persistence;
mod profiles;
mod prompts;
mod ratings;
mod raw;
mod recommendations;
mod render;
mod sendbird;
mod serde_helpers;
mod settings;
mod transport;

pub const DEFAULT_PUBLIC_IDS_BATCH_SIZE: usize = 75;

#[derive(Clone, Debug)]
pub struct RecsFetchConfig {
    pub multi_fetch_count: usize,
    pub request_delay_ms: u64,
    pub rate_limit_retries: usize,
    pub rate_limit_backoff_ms: u64,
}

impl Default for RecsFetchConfig {
    fn default() -> Self {
        Self {
            multi_fetch_count: 3,
            request_delay_ms: 1_500,
            rate_limit_retries: 3,
            rate_limit_backoff_ms: 4_000,
        }
    }
}

#[derive(Clone)]
pub struct HingeClient<S: Storage + Clone> {
    http: Http,
    pub settings: Settings,
    pub storage: S,
    secret_store: Option<std::sync::Arc<dyn SecretStore>>,
    pub phone_number: String,
    pub device_id: String,
    pub install_id: String,
    pub session_id: String,
    pub installed: bool,
    pub hinge_auth: Option<HingeAuthToken>,
    pub sendbird_auth: Option<SendbirdAuthToken>,
    pub sendbird_session_key: Option<String>,
    // Sendbird WS state (single connection)
    sendbird_ws_cmd_tx: Option<tokio::sync::mpsc::UnboundedSender<String>>, // READ, etc.
    sendbird_ws_broadcast_tx: Option<tokio::sync::broadcast::Sender<String>>, // emits incoming frames
    sendbird_ws_connected: bool,
    sendbird_ws_pending_requests: std::sync::Arc<
        tokio::sync::Mutex<
            std::collections::HashMap<String, tokio::sync::oneshot::Sender<serde_json::Value>>,
        >,
    >,
    pub recommendations: std::collections::HashMap<String, RecommendationSubject>,
    // Persistence config
    session_path: Option<String>,
    cache_dir: Option<PathBuf>,
    auto_persist: bool,
    recs_fetch_config: RecsFetchConfig,
    public_ids_batch_size: usize,
    last_recs_v2_call: Option<Instant>,
}

impl<S: Storage + Clone> HingeClient<S> {
    pub fn set_recs_fetch_config(&mut self, config: RecsFetchConfig) {
        self.recs_fetch_config = config;
    }

    pub fn set_public_ids_batch_size(&mut self, batch_size: usize) {
        self.public_ids_batch_size = batch_size.max(1);
    }

    pub fn new(phone_number: impl Into<String>, storage: S, settings: Option<Settings>) -> Self {
        let settings = settings.unwrap_or_default();
        Self {
            http: Http::new(),
            settings,
            storage,
            secret_store: None,
            phone_number: phone_number.into(),
            device_id: Uuid::new_v4().to_string().to_uppercase(),
            install_id: Uuid::new_v4().to_string().to_uppercase(),
            session_id: Uuid::new_v4().to_string().to_uppercase(),
            installed: false,
            hinge_auth: None,
            sendbird_auth: None,
            sendbird_session_key: None,
            sendbird_ws_cmd_tx: None,
            sendbird_ws_broadcast_tx: None,
            sendbird_ws_connected: false,
            sendbird_ws_pending_requests: std::sync::Arc::new(tokio::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            recommendations: std::collections::HashMap::new(),
            session_path: None,
            cache_dir: None,
            auto_persist: false,
            recs_fetch_config: RecsFetchConfig::default(),
            public_ids_batch_size: DEFAULT_PUBLIC_IDS_BATCH_SIZE,
            last_recs_v2_call: None,
        }
    }

    pub fn with_secret_store(mut self, store: std::sync::Arc<dyn SecretStore>) -> Self {
        self.secret_store = Some(store);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::serde_helpers::parse_json_with_path;
    use crate::models::{
        MessageData, SendMessagePayload, SendbirdChannelsResponse, SendbirdMessagesResponse,
    };
    use crate::storage::FsStorage;
    use chrono::Utc;
    use serde::Deserialize;

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct PathAwareOuter {
        items: Vec<PathAwareInner>,
    }

    #[allow(dead_code)]
    #[derive(Debug, Deserialize)]
    struct PathAwareInner {
        count: u32,
    }

    #[test]
    fn response_deserialization_reports_json_path() {
        let err = parse_json_with_path::<PathAwareOuter>(r#"{"items":[{"count":"not-a-number"}]}"#)
            .expect_err("invalid nested field should fail");
        let message = err.to_string();
        assert!(message.contains("items[0].count"), "{message}");
    }

    #[test]
    fn send_message_payload_serializes_camel_case() {
        let payload = SendMessagePayload {
            dedup_id: Some("dedup-1".to_string()),
            ays: false,
            match_message: true,
            message_type: "text".to_string(),
            message_data: MessageData {
                message: "hello".to_string(),
            },
            subject_id: "subject-1".to_string(),
            origin: "connection".to_string(),
        };

        let value = serde_json::to_value(payload).expect("payload should serialize");
        assert_eq!(value["dedupId"], "dedup-1");
        assert_eq!(value["messageData"]["message"], "hello");
        assert_eq!(value["subjectId"], "subject-1");
    }

    #[test]
    fn sendbird_channel_and_message_fixtures_deserialize() {
        let channels = parse_json_with_path::<SendbirdChannelsResponse>(
            r#"{
                "channels": [{
                    "channel_url": "sendbird_group_channel_1",
                    "members": [{"user_id": "user-1", "nickname": "A"}],
                    "created_at": 1710000000000,
                    "updated_at": 1710000000100,
                    "last_message": {
                        "type": "MESG",
                        "message_id": 42,
                        "message": "hello",
                        "created_at": 1710000000000,
                        "user": {"user_id": "user-1"},
                        "channel_url": "sendbird_group_channel_1"
                    }
                }]
            }"#,
        )
        .expect("channels fixture should deserialize");
        assert_eq!(channels.channels[0].channel_url, "sendbird_group_channel_1");
        assert_eq!(
            channels.channels[0]
                .last_message
                .as_ref()
                .expect("last message")
                .message_id,
            "42"
        );

        let messages = parse_json_with_path::<SendbirdMessagesResponse>(
            r#"{
                "messages": [{
                    "type": "MESG",
                    "message_id": 43,
                    "message": "reply",
                    "created_at": 1710000000200,
                    "user": {"user_id": "user-2", "nickname": "B"},
                    "channel_url": "sendbird_group_channel_1"
                }]
            }"#,
        )
        .expect("messages fixture should deserialize");
        assert_eq!(messages.messages[0].message_id, "43");
        assert_eq!(messages.messages[0].user.user_id, "user-2");
    }

    #[test]
    fn sendbird_headers_include_hinge_and_sendbird_identity() {
        let mut client = HingeClient::new("+15555550123", FsStorage, None);
        client.session_id = "session-id".to_string();
        client.device_id = "device-id".to_string();
        client.install_id = "install-id".to_string();
        client.hinge_auth = Some(HingeAuthToken {
            identity_id: "user-id".to_string(),
            token: "hinge-token".to_string(),
            expires: Utc::now(),
        });
        client.sendbird_auth = Some(SendbirdAuthToken {
            token: "sendbird-token".to_string(),
            expires: Utc::now(),
        });
        client.sendbird_session_key = Some("sendbird-session-key".to_string());

        let headers = client
            .sendbird_headers()
            .expect("sendbird headers should build");

        assert_eq!(headers["accept"], "*/*");
        assert_eq!(headers["connection"], "keep-alive");
        assert_eq!(headers["accept-language"], "en-GB");
        assert_eq!(headers["x-session-key"], "session-id");
        assert_eq!(headers["x-device-id"], "device-id");
        assert_eq!(headers["x-install-id"], "install-id");
        assert_eq!(headers["sb-user-id"], "user-id");
        assert_eq!(headers["sb-access-token"], "sendbird-token");
        assert_eq!(headers["session-key"], "sendbird-session-key");
    }

    #[test]
    fn sendbird_sensitive_headers_are_redacted_in_logs() {
        let mut client = HingeClient::new("+15555550123", FsStorage, None);
        client.session_id = "session-id-secret".to_string();
        client.device_id = "device-id-secret".to_string();
        client.install_id = "install-id-secret".to_string();
        client.hinge_auth = Some(HingeAuthToken {
            identity_id: "user-id".to_string(),
            token: "hinge-token-secret".to_string(),
            expires: Utc::now(),
        });
        client.sendbird_auth = Some(SendbirdAuthToken {
            token: "sendbird-token-secret".to_string(),
            expires: Utc::now(),
        });
        client.sendbird_session_key = Some("sendbird-session-secret".to_string());

        let headers = client
            .sendbird_headers()
            .expect("sendbird headers should build");
        let rendered = crate::logging::format_headers(&headers);

        assert!(rendered.contains("sb-access-token: ***REDACTED***"));
        assert!(rendered.contains("session-key: ***REDACTED***"));
        assert!(rendered.contains("x-session-key: ***REDACTED***"));
        assert!(rendered.contains("x-device-id: ***cret"));
        assert!(rendered.contains("x-install-id: ***cret"));
        assert!(!rendered.contains("sendbird-token-secret"));
        assert!(!rendered.contains("sendbird-session-secret"));
        assert!(!rendered.contains("session-id-secret"));
    }
}
