use crate::enums::EducationAttainedProfile;
use crate::errors::HingeError;
use crate::logging::{log_request, log_response};
use crate::models::{
    AccountInfo, AnswerContentPayload, AnswerEvaluateRequest, AuthSettings, ConnectionContentItem,
    ConnectionDetailApi, ConnectionItem, ConnectionsResponse, CreatePromptPollRequest,
    CreatePromptPollResponse, CreateRate, CreateRateContent, CreateRateContentPrompt,
    CreateVideoPromptRequest, CreateVideoPromptResponse, ExportChatInput, ExportChatResult,
    ExportStatus, ExportedMediaFile, HingeAuthToken, LikeLimit, LikeResponse, LikesV2Response,
    LoginTokens, MatchNoteResponse, NotificationSettings, PhotoAsset, PhotoAssetInput, Preferences,
    PreferencesResponse, ProfileContentFull, ProfileUpdate, Prompt, PromptsResponse,
    PublicUserProfile, RateInput, RateRespondRequest, RateRespondResponse, RecommendationSubject,
    RecommendationsResponse, SelfContentResponse, SelfProfileResponse, SendbirdAuthToken,
    SendbirdChannelHandle, SendbirdGroupChannel, SendbirdMessage, SkipInput, StandoutsResponse,
    UserSettings, UserTrait,
};
use crate::prompts_manager::HingePromptsManager;
use crate::settings::Settings;
use crate::storage::{SecretStore, Storage};
use chrono::{DateTime, Local, Utc};
use futures_util::{SinkExt, StreamExt};
use reqwest::{Client as Http, StatusCode};
use serde_json::json;
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

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

/// Convert ProfileUpdate to API format with numeric enum values
fn profile_update_to_api_json(update: &ProfileUpdate) -> serde_json::Value {
    use crate::enums::ApiEnum;

    let mut obj = serde_json::Map::new();

    // Convert each field, using the enum's to_api_value() when needed
    if let Some(ref children) = update.children {
        obj.insert(
            "children".to_string(),
            json!({
                "value": children.value.to_api_value(),
                "visible": children.visible
            }),
        );
    }

    if let Some(ref dating) = update.dating_intention {
        obj.insert(
            "datingIntention".to_string(),
            json!({
                "value": dating.value.to_api_value(),
                "visible": dating.visible
            }),
        );
    }

    if let Some(ref drinking) = update.drinking {
        obj.insert(
            "drinking".to_string(),
            json!({
                "value": drinking.value.to_api_value(),
                "visible": drinking.visible
            }),
        );
    }

    if let Some(ref drugs) = update.drugs {
        obj.insert(
            "drugs".to_string(),
            json!({
                "value": drugs.value.to_api_value(),
                "visible": drugs.visible
            }),
        );
    }

    if let Some(ref marijuana) = update.marijuana {
        obj.insert(
            "marijuana".to_string(),
            json!({
                "value": marijuana.value.to_api_value(),
                "visible": marijuana.visible
            }),
        );
    }

    if let Some(ref smoking) = update.smoking {
        obj.insert(
            "smoking".to_string(),
            json!({
                "value": smoking.value.to_api_value(),
                "visible": smoking.visible
            }),
        );
    }

    if let Some(ref politics) = update.politics {
        obj.insert(
            "politics".to_string(),
            json!({
                "value": politics.value.to_api_value(),
                "visible": politics.visible
            }),
        );
    }

    if let Some(ref religions) = update.religions {
        let values: Vec<i8> = religions.value.iter().map(|e| e.to_api_value()).collect();
        obj.insert(
            "religions".to_string(),
            json!({
                "value": values,
                "visible": religions.visible
            }),
        );
    }

    if let Some(ref ethnicities) = update.ethnicities {
        let values: Vec<i8> = ethnicities.value.iter().map(|e| e.to_api_value()).collect();
        obj.insert(
            "ethnicities".to_string(),
            json!({
                "value": values,
                "visible": ethnicities.visible
            }),
        );
    }

    if let Some(ref education) = update.education_attained {
        obj.insert(
            "educationAttained".to_string(),
            json!(education.to_api_value()),
        );
    }

    if let Some(ref relationships) = update.relationship_type_ids {
        let values: Vec<i8> = relationships
            .value
            .iter()
            .map(|e| e.to_api_value())
            .collect();
        obj.insert(
            "relationshipTypeIds".to_string(),
            json!({
                "value": values,
                "visible": relationships.visible
            }),
        );
    }

    if let Some(height) = update.height {
        obj.insert("height".to_string(), json!(height));
    }

    if let Some(ref gender) = update.gender_id {
        obj.insert("genderId".to_string(), json!(gender.to_api_value()));
    }

    if let Some(ref hometown) = update.hometown {
        obj.insert(
            "hometown".to_string(),
            json!({
                "value": hometown.value,
                "visible": hometown.visible
            }),
        );
    }

    if let Some(ref languages) = update.languages_spoken {
        obj.insert(
            "languagesSpoken".to_string(),
            json!({
                "value": languages.value,
                "visible": languages.visible
            }),
        );
    }

    if let Some(ref zodiac) = update.zodiac {
        obj.insert(
            "zodiac".to_string(),
            json!({
                "value": zodiac.value,
                "visible": zodiac.visible
            }),
        );
    }

    serde_json::Value::Object(obj)
}

/// Convert Preferences to API format with numeric enum values
fn preferences_to_api_json(prefs: &Preferences) -> serde_json::Value {
    use crate::enums::ApiEnum;

    json!({
        "genderedAgeRanges": prefs.gendered_age_ranges,
        "dealbreakers": prefs.dealbreakers,
        "religions": prefs.religions.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "drinking": prefs.drinking.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "genderedHeightRanges": prefs.gendered_height_ranges,
        "marijuana": prefs.marijuana.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "relationshipTypes": prefs.relationship_types.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "drugs": prefs.drugs.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "maxDistance": prefs.max_distance,
        "children": prefs.children.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "ethnicities": prefs.ethnicities.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "smoking": prefs.smoking.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "educationAttained": prefs.education_attained.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "familyPlans": prefs.family_plans,
        "datingIntentions": prefs.dating_intentions.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "politics": prefs.politics.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "genderPreferences": prefs.gender_preferences.iter().map(|e| e.to_api_value()).collect::<Vec<_>>()
    })
}

const CHILDREN_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't have children"),
    (2, "Have children"),
];

const DATING_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Unknown"),
    (1, "Life partner"),
    (2, "Long-term relationship"),
    (3, "Long-term, open to short"),
    (4, "Short-term, open to long"),
    (5, "Short-term relationship"),
    (6, "Figuring out their dating goals"),
];

const DRINKING_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't drink"),
    (2, "Drink"),
    (3, "Sometimes"),
];

const SMOKING_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't smoke"),
    (2, "Smoke"),
    (3, "Sometimes"),
];

const MARIJUANA_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't use marijuana"),
    (2, "Use marijuana"),
    (3, "Sometimes"),
    (4, "No preference"),
];

const DRUG_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't use drugs"),
    (2, "Use drugs"),
    (3, "Sometimes"),
];

const RELATIONSHIP_TYPE_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (1, "Monogamy"),
    (2, "Ethical non-monogamy"),
    (3, "Open relationship"),
    (4, "Polyamory"),
    (5, "Open to exploring"),
];

fn label_from_map(map: &'static [(i32, &'static str)], code: Option<i32>) -> Option<&'static str> {
    let key = code?;
    map.iter().find(|(c, _)| *c == key).map(|(_, label)| *label)
}

fn labels_from_map(
    map: &'static [(i32, &'static str)],
    codes: &Option<Vec<i32>>,
) -> Vec<&'static str> {
    match codes {
        Some(values) => values
            .iter()
            .filter_map(|code| map.iter().find(|(c, _)| c == code).map(|(_, label)| *label))
            .collect(),
        None => Vec::new(),
    }
}

fn sanitize_component(input: &str) -> String {
    let trimmed = input.trim();
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    if out.is_empty() { "export".into() } else { out }
}

fn parse_ts(value: &str) -> Option<i64> {
    value.parse::<i64>().ok()
}

fn parse_json_with_path<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, HingeError> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|err| {
        let path = err.path().to_string();
        if path == "." {
            HingeError::Serde(err.inner().to_string())
        } else {
            HingeError::Serde(format!("{} at {}", err.inner(), path))
        }
    })
}

fn parse_json_value_with_path<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<T, HingeError> {
    parse_json_with_path(&value.to_string())
}

fn attachment_from_value(value: &serde_json::Value) -> Option<(String, String)> {
    if !value.is_object() {
        return None;
    }
    let url = value
        .get("url")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("secure_url").and_then(|v| v.as_str()))?;
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            url.split('/')
                .next_back()
                .unwrap_or("attachment")
                .to_string()
        });
    Some((url.to_string(), name))
}

fn education_attained_label(value: &EducationAttainedProfile) -> &'static str {
    use EducationAttainedProfile::*;
    match value {
        PreferNotToSay => "Prefer not to say",
        HighSchool => "High school",
        TradeSchool => "Trade school",
        InCollege => "In college",
        Undergraduate => "Undergraduate degree",
        InGradSchool => "In grad school",
        Graduate => "Graduate degree",
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

    pub async fn rendered_profile_text_for_user(
        &mut self,
        user_id: &str,
    ) -> Result<String, HingeError> {
        let uid = user_id.trim();
        if uid.is_empty() {
            return Ok(String::new());
        }

        let prompts_manager = match self.fetch_prompts_manager().await {
            Ok(mgr) => Some(mgr),
            Err(err) => {
                log::warn!("Failed to prefetch prompts for rendered profile: {}", err);
                None
            }
        };

        let profile = self
            .get_profiles(vec![uid.to_string()])
            .await?
            .into_iter()
            .next();

        let profile_content = self
            .get_profile_content(vec![uid.to_string()])
            .await?
            .into_iter()
            .next();

        let text = render_profile(
            profile.as_ref(),
            profile_content.as_ref(),
            prompts_manager.as_ref(),
        );
        Ok(text)
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

    // Helper method for making GET requests with logging
    async fn http_get(&self, url: &str) -> Result<reqwest::Response, HingeError> {
        let headers = self.default_headers()?;
        log_request("GET", url, &headers, None);

        let res = self.http.get(url).headers(headers.clone()).send().await?;

        log::info!("GET {} -> {}", url, res.status());
        Ok(res)
    }

    async fn http_get_bytes(&self, url: &str) -> Result<Vec<u8>, HingeError> {
        log::info!("GET (bytes) {}", url);
        let res = self.http.get(url).send().await?;
        let status = res.status();
        if !status.is_success() {
            let text = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".into());
            return Err(HingeError::Http(format!("status {}: {}", status, text)));
        }
        res.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| HingeError::Http(format!("Failed to download media: {}", e)))
    }

    // Helper method for making POST requests with logging
    async fn http_post(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, HingeError> {
        let headers = self.default_headers()?;
        log_request("POST", url, &headers, Some(body));

        let res = self
            .http
            .post(url)
            .headers(headers.clone())
            .json(body)
            .send()
            .await?;

        log::info!("POST {} -> {}", url, res.status());
        Ok(res)
    }

    // Helper method for making PATCH requests with logging
    async fn http_patch(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, HingeError> {
        let headers = self.default_headers()?;
        log_request("PATCH", url, &headers, Some(body));

        let res = self
            .http
            .patch(url)
            .headers(headers.clone())
            .json(body)
            .send()
            .await?;

        log::info!("PATCH {} -> {}", url, res.status());
        Ok(res)
    }

    // Helper to parse response with logging
    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        res: reqwest::Response,
    ) -> Result<T, HingeError> {
        let status = res.status();
        let headers = res.headers().clone();

        if !status.is_success() {
            let text = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            log::error!("HTTP Error {}: {}", status, text);
            return Err(HingeError::Http(format!("status {}: {}", status, text)));
        }

        let text = res.text().await?;
        match parse_json_with_path::<T>(&text) {
            Ok(data) => {
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&text) {
                    log_response(status, &headers, Some(&json_val));
                }
                Ok(data)
            }
            Err(e) => {
                log::error!("Failed to parse response: {}", e);
                log::error!("Response text: {}", text);
                Err(e)
            }
        }
    }

    fn default_headers(&self) -> Result<reqwest::header::HeaderMap, HingeError> {
        use reqwest::header::{HeaderMap, HeaderValue};
        let mut h = HeaderMap::new();
        h.insert("content-type", HeaderValue::from_static("application/json"));
        h.insert("accept", HeaderValue::from_static("*/*"));
        h.insert("accept-language", HeaderValue::from_static("en-GB"));
        h.insert("connection", HeaderValue::from_static("keep-alive"));
        h.insert(
            "accept-encoding",
            HeaderValue::from_static("gzip, deflate, br"),
        );
        h.insert(
            "x-device-model-code",
            HeaderValue::from_static("iPhone15,2"),
        );
        h.insert("x-device-model", HeaderValue::from_static("unknown"));
        h.insert("x-device-region", HeaderValue::from_static("IN"));
        // Required Hinge headers
        h.insert(
            "x-session-id",
            HeaderValue::from_str(&self.session_id)
                .map_err(|e| HingeError::Http(format!("Invalid session id header: {}", e)))?,
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
        h.insert("x-device-platform", HeaderValue::from_static("iOS"));
        h.insert(
            "x-app-version",
            HeaderValue::from_str(&self.settings.hinge_app_version)
                .map_err(|e| HingeError::Http(format!("Invalid app version header: {}", e)))?,
        );
        h.insert(
            "x-build-number",
            HeaderValue::from_str(&self.settings.hinge_build_number)
                .map_err(|e| HingeError::Http(format!("Invalid build number header: {}", e)))?,
        );
        h.insert(
            "x-os-version",
            HeaderValue::from_str(&self.settings.os_version)
                .map_err(|e| HingeError::Http(format!("Invalid OS version header: {}", e)))?,
        );
        // Hardcoded Darwin kernel version for iOS 26.0 (iPhone 15,2)
        let ua = format!(
            "Hinge/{} CFNetwork/3859.100.1 Darwin/25.0.0",
            self.settings.hinge_build_number
        );
        h.insert(
            "user-agent",
            HeaderValue::from_str(&ua)
                .map_err(|e| HingeError::Http(format!("Invalid user agent header: {}", e)))?,
        );
        if let Some(token) = &self.hinge_auth {
            h.insert(
                "authorization",
                HeaderValue::from_str(&format!("Bearer {}", token.token))
                    .map_err(|e| HingeError::Http(format!("Invalid auth token header: {}", e)))?,
            );
        }
        Ok(h)
    }

    fn sendbird_headers(&self) -> Result<reqwest::header::HeaderMap, HingeError> {
        use reqwest::header::{HeaderMap, HeaderValue};
        let mut h = HeaderMap::new();
        h.insert("accept", HeaderValue::from_static("application/json"));
        h.insert(
            "accept-encoding",
            HeaderValue::from_static("gzip, deflate, br"),
        );
        h.insert("connection", HeaderValue::from_static("Keep-Alive"));
        h.insert(
            "accept-language",
            HeaderValue::from_static("en-IN,en;q=0.9"),
        );
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
        let sendbird_hdr = format!(
            "iOS,{},{},{}",
            self.settings.os_version,
            self.settings.sendbird_sdk_version,
            self.settings.sendbird_app_id
        );
        h.insert(
            "SendBird",
            HeaderValue::from_str(&sendbird_hdr)
                .map_err(|e| HingeError::Http(format!("Invalid SendBird header: {}", e)))?,
        );
        h.insert(
            "SB-User-Agent",
            HeaderValue::from_str(&format!("iOS/c{}///", self.settings.sendbird_sdk_version))
                .map_err(|e| HingeError::Http(format!("Invalid SB-User-Agent: {}", e)))?,
        );
        h.insert(
            "SB-SDK-User-Agent",
            HeaderValue::from_str(&format!(
                "main_sdk_info=chat/ios/{}&device_os_platform=ios&os_version={}",
                self.settings.sendbird_sdk_version, self.settings.os_version
            ))
            .map_err(|e| HingeError::Http(format!("Invalid SB-SDK-User-Agent: {}", e)))?,
        );
        h.insert("user-agent", HeaderValue::from_static("Jios/4.26.0"));
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

    #[allow(dead_code)]
    async fn sendbird_post_json(
        &self,
        path_and_query: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, HingeError> {
        let url = format!("{}/v3{}", self.settings.sendbird_api_url, path_and_query);
        let mut headers = self.sendbird_headers()?;
        use reqwest::header::HeaderValue;
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        log_request("POST", &url, &headers, Some(body));
        let res = self
            .http
            .post(url)
            .headers(headers.clone())
            .json(body)
            .send()
            .await?;
        log::info!("[sendbird] POST {} -> {}", path_and_query, res.status());
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
        let ws_url = format!(
            "{}/?p=iOS&sv={}&pv={}&uikit_config=0&use_local_cache=0&include_extra_data=premium_feature_list,file_upload_size_limit,emoji_hash,application_attributes,notifications,message_template,ai_agent&include_poll_details=1&user_id={}&ai={}&pmce=1&expiring_session=0&config_ts=0",
            self.settings.sendbird_ws_url,
            self.settings.sendbird_sdk_version,
            self.settings.os_version,
            self.hinge_auth
                .as_ref()
                .map(|t| t.identity_id.clone())
                .unwrap_or_default(),
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
            .header("Origin", "")
            .header("Accept-Language", "en-IN,en;q=0.9")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header(
                "User-Agent",
                &format!(
                    "Hinge/{} CFNetwork/3859.100.1 Darwin/25.0.0",
                    self.settings.hinge_build_number
                ),
            );
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
            pairs.push(("Accept-Language".into(), "en-IN,en;q=0.9".into()));
            pairs.push(("Host".into(), host.to_string()));
            pairs.push(("Origin".into(), "".into()));
            pairs.push(("Sec-WebSocket-Key".into(), ws_key.clone()));
            pairs.push(("Sec-WebSocket-Version".into(), "13".into()));
            pairs.push(("Request-Sent-Timestamp".into(), ws_ts.clone()));
            pairs.push(("Connection".into(), "Upgrade".into()));
            pairs.push(("Upgrade".into(), "websocket".into()));
            pairs.push((
                "User-Agent".into(),
                format!(
                    "Hinge/{} CFNetwork/3859.100.1 Darwin/25.0.0",
                    self.settings.hinge_build_number
                ),
            ));
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
                                if let Some(k) = val.get("key").and_then(|v| v.as_str()) {
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

    async fn ensure_device_registered(&mut self) -> Result<(), HingeError> {
        if self.installed {
            return Ok(());
        }
        let url = format!("{}/identity/install", self.settings.base_url);
        let body = json!({"installId": self.install_id});
        let res = self
            .http_post(&url, &body)
            .await
            .map_err(|e| HingeError::Http(format!("Failed to register device: {}", e)))?;

        if !res.status().is_success() {
            return Err(HingeError::Http(format!(
                "Device registration failed with status {}",
                res.status()
            )));
        }
        self.installed = true;
        Ok(())
    }

    pub async fn initiate_login(&mut self) -> Result<(), HingeError> {
        self.ensure_device_registered().await?;
        let url = format!("{}/auth/sms/v2/initiate", self.settings.base_url);
        let body = json!({"deviceId": self.device_id, "phoneNumber": self.phone_number});
        let res = self
            .http_post(&url, &body)
            .await
            .map_err(|e| HingeError::Http(format!("Failed to initiate SMS login: {}", e)))?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!(
                "SMS initiation failed with status {}",
                res.status()
            )));
        }
        Ok(())
    }

    pub async fn submit_otp(&mut self, otp: &str) -> Result<LoginTokens, HingeError> {
        let url = format!("{}/auth/sms/v2", self.settings.base_url);
        let body = json!({
            "installId": self.install_id,
            "deviceId": self.device_id,
            "phoneNumber": self.phone_number,
            "otp": otp,
        });
        let res = self.http_post(&url, &body).await?;
        if res.status() == reqwest::StatusCode::PRECONDITION_FAILED {
            let v: serde_json::Value = res.json().await?;
            let case_id = v
                .get("caseId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let email = v
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            return Err(HingeError::Email2FA { case_id, email });
        }
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let v = self.parse_response::<LoginTokens>(res).await?;
        if let Some(t) = v.hinge_auth_token.clone() {
            self.hinge_auth = Some(t);
        }
        if let Some(t) = v.sendbird_auth_token.clone() {
            self.sendbird_auth = Some(t);
        }
        Ok(v)
    }

    pub async fn submit_email_code(
        &mut self,
        case_id: &str,
        email_code: &str,
    ) -> Result<LoginTokens, HingeError> {
        let url = format!("{}/auth/device/validate", self.settings.base_url);
        let body = json!({
            "installId": self.install_id,
            "code": email_code,
            "caseId": case_id,
            "deviceId": self.device_id,
        });
        let res = self.http_post(&url, &body).await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let t = self.parse_response::<HingeAuthToken>(res).await?;
        self.hinge_auth = Some(t);
        let _ = self.authenticate_with_sendbird().await; // best-effort
        Ok(LoginTokens {
            hinge_auth_token: self.hinge_auth.clone(),
            sendbird_auth_token: self.sendbird_auth.clone(),
        })
    }

    pub fn save_session(&self, path: &str) -> Result<(), HingeError> {
        let session = json!({
          "phoneNumber": self.phone_number,
          "deviceId": self.device_id,
          "installId": self.install_id,
          "sessionId": self.session_id,
          "installed": self.installed,
          "hingeAuth": self.hinge_auth,
          "sendbirdAuth": self.sendbird_auth,
          "sendbirdSessionKey": self.sendbird_session_key,
        });
        let data =
            serde_json::to_string_pretty(&session).map_err(|e| HingeError::Serde(e.to_string()))?;
        self.storage
            .write_string(path, &data)
            .map_err(|e| HingeError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn load_session(&mut self, path: &str) -> Result<(), HingeError> {
        if !self.storage.exists(path) {
            return Ok(());
        }
        let data = self
            .storage
            .read_to_string(path)
            .map_err(|e| HingeError::Storage(e.to_string()))?;
        let v: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| HingeError::Serde(e.to_string()))?;
        if let Some(s) = v.get("phoneNumber").and_then(|v| v.as_str()) {
            self.phone_number = s.to_string();
        }
        if let Some(s) = v.get("deviceId").and_then(|v| v.as_str()) {
            self.device_id = s.to_string();
        }
        if let Some(s) = v.get("installId").and_then(|v| v.as_str()) {
            self.install_id = s.to_string();
        }
        if let Some(s) = v.get("sessionId").and_then(|v| v.as_str()) {
            self.session_id = s.to_string();
        }
        if let Some(b) = v.get("installed").and_then(|v| v.as_bool()) {
            self.installed = b;
        }
        if let Some(t) = v.get("hingeAuth").cloned() {
            self.hinge_auth = serde_json::from_value(t).ok();
        }
        if let Some(t) = v.get("sendbirdAuth").cloned() {
            self.sendbird_auth = serde_json::from_value(t).ok();
        }
        if let Some(k) = v.get("sendbirdSessionKey").and_then(|v| v.as_str()) {
            self.sendbird_session_key = Some(k.to_string());
        }
        Ok(())
    }

    pub fn load_tokens_secure(&mut self) -> Result<(), HingeError> {
        if let Some(store) = &self.secret_store {
            if let Some(v) = store
                .get_secret("hinge_auth")
                .map_err(|e| HingeError::Storage(e.to_string()))?
            {
                self.hinge_auth = serde_json::from_str(&v).ok();
            }
            if let Some(v) = store
                .get_secret("sendbird_auth")
                .map_err(|e| HingeError::Storage(e.to_string()))?
            {
                self.sendbird_auth = serde_json::from_str(&v).ok();
            }
        }
        Ok(())
    }

    pub fn with_persistence(
        mut self,
        session_path: Option<String>,
        cache_dir: Option<PathBuf>,
        auto_persist: bool,
    ) -> Self {
        self.session_path = session_path;
        self.cache_dir = cache_dir;
        self.auto_persist = auto_persist;
        if let Some(path) = self.session_path.clone() {
            let _ = self.load_session(&path);
        }
        if let Some(dir) = &self.cache_dir {
            let rec_path = dir.join(format!("recommendations_{}.json", self.session_id));
            let _ = self.load_recommendations(rec_path.to_string_lossy().as_ref());
        }
        self
    }

    fn recs_cache_path(&self) -> Option<String> {
        self.cache_dir.as_ref().map(|d| {
            d.join(format!("recommendations_{}.json", self.session_id))
                .to_string_lossy()
                .into_owned()
        })
    }

    fn prompts_cache_path(&self) -> Option<String> {
        self.cache_dir
            .as_ref()
            .map(|d| d.join("prompts_cache.json").to_string_lossy().into_owned())
    }

    pub async fn fetch_prompts(&mut self) -> Result<PromptsResponse, HingeError> {
        if self.auto_persist
            && let Some(path) = self.prompts_cache_path()
            && Path::new(&path).exists()
            && let Ok(text) = std::fs::read_to_string(&path)
            && let Ok(val) = serde_json::from_str::<PromptsResponse>(&text)
        {
            return Ok(val);
        }
        let url = format!("{}/prompts", self.settings.base_url);
        let payload = self.prompt_payload().await;
        let res = self.http_post(&url, &payload).await?;
        let body = self.parse_response::<PromptsResponse>(res).await?;
        if self.auto_persist
            && let Some(path) = self.prompts_cache_path()
        {
            let _ = std::fs::write(
                &path,
                serde_json::to_string_pretty(&body).unwrap_or("{}".into()),
            );
        }
        Ok(body)
    }

    pub async fn fetch_prompts_manager(&mut self) -> Result<HingePromptsManager, HingeError> {
        let resp = self.fetch_prompts().await?;
        Ok(HingePromptsManager::new(resp))
    }

    pub async fn get_prompt_text(&mut self, prompt_id: &str) -> Result<String, HingeError> {
        let mgr = self.fetch_prompts_manager().await?;
        Ok(mgr.get_prompt_display_text(prompt_id))
    }

    pub async fn search_prompts(&mut self, query: &str) -> Result<Vec<Prompt>, HingeError> {
        let mgr = self.fetch_prompts_manager().await?;
        let items = mgr.search_prompts(query);
        Ok(items.into_iter().cloned().collect())
    }

    pub async fn get_prompts_by_category(
        &mut self,
        category_slug: &str,
    ) -> Result<Vec<Prompt>, HingeError> {
        let mgr = self.fetch_prompts_manager().await?;
        let items = mgr.get_prompts_by_category(category_slug);
        Ok(items.into_iter().cloned().collect())
    }

    pub async fn get_recommendations(&mut self) -> Result<RecommendationsResponse, HingeError> {
        self.get_recommendations_v2_params(crate::models::RecsV2Params {
            new_here: false,
            active_today: false,
        })
        .await
    }

    pub async fn get_recommendations_v2_params(
        &mut self,
        params: crate::models::RecsV2Params,
    ) -> Result<RecommendationsResponse, HingeError> {
        let url = format!("{}/rec/v2", self.settings.base_url);
        let identity_id = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?
            .identity_id
            .clone();

        use serde::Serialize;
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            player_id: String,
            new_here: bool,
            active_today: bool,
        }

        let body = Body {
            player_id: identity_id,
            new_here: params.new_here,
            active_today: params.active_today,
        };

        let body_json =
            serde_json::to_value(&body).map_err(|e| HingeError::Serde(e.to_string()))?;

        let fetch_count = self.recs_fetch_config.multi_fetch_count.max(1);
        let min_delay = Duration::from_millis(self.recs_fetch_config.request_delay_ms);
        let mut aggregated: Option<RecommendationsResponse> = None;
        let mut completed_calls = 0usize;
        let mut rate_limit_attempts = 0usize;
        let max_rate_limit_retries = self.recs_fetch_config.rate_limit_retries;
        let base_backoff_ms = self.recs_fetch_config.rate_limit_backoff_ms.max(1);

        while completed_calls < fetch_count {
            if let Some(last_call) = self.last_recs_v2_call {
                let elapsed = last_call.elapsed();
                if elapsed < min_delay {
                    sleep(min_delay - elapsed).await;
                }
            }

            let res = self.http_post(&url, &body_json).await?;
            self.last_recs_v2_call = Some(Instant::now());

            let status = res.status();
            if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::SERVICE_UNAVAILABLE
            {
                rate_limit_attempts += 1;
                if rate_limit_attempts > max_rate_limit_retries {
                    log::warn!(
                        "[rec/v2] rate limited after {} retries; returning aggregated data",
                        rate_limit_attempts
                    );
                    break;
                }
                let exponent = rate_limit_attempts.saturating_sub(1) as u32;
                let factor = 1u64
                    .checked_shl(exponent)
                    .filter(|&v| v > 0)
                    .unwrap_or(u64::MAX);
                let backoff = base_backoff_ms.saturating_mul(factor);
                log::warn!(
                    "[rec/v2] rate limited (status {}). backing off {} ms before retry (attempt {}/{})",
                    status,
                    backoff,
                    rate_limit_attempts,
                    max_rate_limit_retries
                );
                sleep(Duration::from_millis(backoff)).await;
                continue;
            }

            rate_limit_attempts = 0;

            let response = self.parse_response::<RecommendationsResponse>(res).await?;
            if let Some(existing) = aggregated.as_mut() {
                merge_recommendation_responses(existing, response);
            } else {
                aggregated = Some(response);
            }

            completed_calls += 1;
        }

        let mut out = aggregated.unwrap_or_else(|| RecommendationsResponse {
            feeds: Vec::new(),
            active_pills: None,
            cache_control: None,
        });

        normalize_recommendations_response(&mut out);

        if self.auto_persist {
            match self.recs_cache_path() {
                Some(path) => {
                    let _ = self.apply_recommendations_and_save(&mut out, Some(&path));
                }
                None => {
                    let _ = self.apply_recommendations_and_save(&mut out, None);
                }
            }
        }
        Ok(out)
    }

    pub fn apply_recommendations_and_save(
        &mut self,
        recs: &mut RecommendationsResponse,
        path: Option<&str>,
    ) -> Result<(), HingeError> {
        for feed in &mut recs.feeds {
            for subj in &mut feed.subjects {
                if subj.origin.is_none() {
                    subj.origin = Some(feed.origin.clone());
                }
                if !self.recommendations.contains_key(&subj.subject_id) {
                    self.recommendations
                        .insert(subj.subject_id.clone(), subj.clone());
                }
            }
        }
        if let Some(p) = path {
            self.save_recommendations(p)?;
        }
        Ok(())
    }

    pub async fn get_self_profile(&self) -> Result<SelfProfileResponse, HingeError> {
        let url = format!("{}/user/v3", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response::<SelfProfileResponse>(res).await
    }

    pub async fn get_self_content(&self) -> Result<SelfContentResponse, HingeError> {
        let url = format!("{}/content/v2", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response::<SelfContentResponse>(res).await
    }

    pub async fn get_self_preferences(&self) -> Result<PreferencesResponse, HingeError> {
        let url = format!("{}/preference/v2/selected", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response::<PreferencesResponse>(res).await
    }

    pub async fn get_like_limit(&self) -> Result<LikeLimit, HingeError> {
        let url = format!("{}/likelimit", self.settings.base_url);
        let res = self
            .http
            .get(url)
            .headers(self.default_headers()?)
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        self.parse_response(res).await
    }

    pub async fn get_likes_v2(&self) -> Result<LikesV2Response, HingeError> {
        let url = format!("{}/like/v2", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_like_subject(
        &self,
        subject_id: &str,
    ) -> Result<crate::models::LikeItemV2, HingeError> {
        let url = format!("{}/like/subject/{}", self.settings.base_url, subject_id);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_match_note(&self, subject_id: &str) -> Result<MatchNoteResponse, HingeError> {
        let url = format!(
            "{}/connection/v2/matchnote/{}",
            self.settings.base_url, subject_id
        );
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    /// Fetch the raw JSON response for likes v2 without mapping to typed structs
    pub async fn get_likes_v2_raw(&self) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/like/v2", self.settings.base_url);
        let res = self.http_get(&url).await?;
        let status = res.status();
        let headers = res.headers().clone();
        let text = res
            .text()
            .await
            .map_err(|e| HingeError::Http(e.to_string()))?;
        if !status.is_success() {
            log::error!("HTTP Error {}: {}", status, text);
            return Err(HingeError::Http(format!("status {}: {}", status, text)));
        }
        let val: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| HingeError::Serde(e.to_string()))?;
        log_response(status, &headers, Some(&val));
        Ok(val)
    }

    /// Raw, unfiltered request to user/v3/public for explicit ids
    pub async fn get_profiles_public_raw_unfiltered(
        &self,
        ids: Vec<String>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!(
            "{}/user/v3/public?ids={}",
            self.settings.base_url,
            ids.join(",")
        );
        let res = self.http_get(&url).await?;
        let status = res.status();
        let headers = res.headers().clone();
        let text = res
            .text()
            .await
            .map_err(|e| HingeError::Http(e.to_string()))?;
        if !status.is_success() {
            log::error!("HTTP Error {}: {}", status, text);
            return Err(HingeError::Http(format!("status {}: {}", status, text)));
        }
        let val: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| HingeError::Serde(e.to_string()))?;
        log_response(status, &headers, Some(&val));
        Ok(val)
    }

    /// Raw, unfiltered request to content/v2/public for explicit ids
    pub async fn get_content_public_raw_unfiltered(
        &self,
        ids: Vec<String>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!(
            "{}/content/v2/public?ids={}",
            self.settings.base_url,
            ids.join(",")
        );
        let res = self.http_get(&url).await?;
        let status = res.status();
        let headers = res.headers().clone();
        let text = res
            .text()
            .await
            .map_err(|e| HingeError::Http(e.to_string()))?;
        if !status.is_success() {
            log::error!("HTTP Error {}: {}", status, text);
            return Err(HingeError::Http(format!("status {}: {}", status, text)));
        }
        let val: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| HingeError::Serde(e.to_string()))?;
        log_response(status, &headers, Some(&val));
        Ok(val)
    }

    pub async fn get_profiles(
        &self,
        user_ids: Vec<String>,
    ) -> Result<Vec<PublicUserProfile>, HingeError> {
        let chunks = self.prepare_user_id_chunks(user_ids);
        if chunks.is_empty() {
            return Ok(Vec::new());
        }
        let mut aggregated: Vec<PublicUserProfile> = Vec::new();
        for batch in chunks {
            let url = format!(
                "{}/user/v3/public?ids={}",
                self.settings.base_url,
                batch.join(",")
            );
            let res = self.http_get(&url).await?;
            let mut part: Vec<PublicUserProfile> = self.parse_response(res).await?;
            aggregated.append(&mut part);
        }
        Ok(aggregated)
    }

    pub async fn get_profile_content(
        &self,
        user_ids: Vec<String>,
    ) -> Result<Vec<ProfileContentFull>, HingeError> {
        let chunks = self.prepare_user_id_chunks(user_ids);
        if chunks.is_empty() {
            return Ok(Vec::new());
        }
        let mut aggregated: Vec<ProfileContentFull> = Vec::new();
        for batch in chunks {
            let url = format!(
                "{}/content/v2/public?ids={}",
                self.settings.base_url,
                batch.join(",")
            );
            let res = self.http_get(&url).await?;
            let mut part: Vec<ProfileContentFull> = self.parse_response(res).await?;
            aggregated.append(&mut part);
        }
        Ok(aggregated)
    }

    // Removed non-DTO rating/skip methods to standardize on DTO-based API

    pub async fn skip(&mut self, input: SkipInput) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/rate/v2/initiate", self.settings.base_url);
        let payload = CreateRate {
            rating_id: Uuid::new_v4().to_string().to_uppercase(),
            hcm_run_id: None,
            session_id: self.session_id.clone(),
            // Explicitly set None; it will be omitted during serialization
            content: None,
            created: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            rating_token: input.rating_token,
            initiated_with: None,
            rating: "skip".into(),
            has_pairing: false,
            origin: Some(input.origin.unwrap_or_else(|| "compatibles".into())),
            subject_id: input.subject_id.clone(),
        };
        let res = self
            .http_post(&url, &serde_json::to_value(&payload).unwrap())
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let body = res.json::<serde_json::Value>().await?;
        self.remove_recommendation(&input.subject_id);
        if self.auto_persist
            && let Some(path) = self.recs_cache_path()
        {
            let _ = self.save_recommendations(&path);
        }
        Ok(body)
    }

    pub async fn rate_user(&mut self, input: RateInput) -> Result<LikeResponse, HingeError> {
        let mut hcm_run_id: Option<String> = None;
        if let Some(text) = input.comment.as_deref() {
            let run_id = self.run_text_review(text, &input.subject_id).await?;
            hcm_run_id = Some(run_id);
        }
        let prompt_answer = input.answer_text.clone().unwrap_or_default();
        let prompt_question = input.question_text.clone().unwrap_or_default();
        let prompt_content_id = input.content_id.clone();

        let content = if let Some(photo) = input.photo {
            let PhotoAssetInput {
                url,
                content_id,
                cdn_id,
                bounding_box,
                selfie_verified,
            } = photo;
            Some(CreateRateContent {
                comment: input.comment.clone(),
                photo: Some(PhotoAsset {
                    id: None,
                    url,
                    cdn_id,
                    content_id,
                    prompt_id: None,
                    caption: None,
                    width: None,
                    height: None,
                    video_url: None,
                    selfie_verified,
                    bounding_box,
                    location: None,
                    source: None,
                    source_id: None,
                    p_hash: None,
                }),
                prompt: None,
            })
        } else {
            let prompt = CreateRateContentPrompt {
                answer: prompt_answer,
                content_id: prompt_content_id,
                question: prompt_question,
            };
            Some(CreateRateContent {
                comment: input.comment.clone(),
                photo: None,
                prompt: Some(prompt),
            })
        };
        let payload = CreateRate {
            rating_id: Uuid::new_v4().to_string().to_uppercase(),
            hcm_run_id,
            session_id: self.session_id.clone(),
            content,
            created: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            rating_token: input.rating_token,
            initiated_with: Some(if input.use_superlike.unwrap_or(false) {
                "superlike".into()
            } else {
                "standard".into()
            }),
            rating: if input.comment.is_some() {
                "note".into()
            } else {
                "like".into()
            },
            has_pairing: false,
            origin: Some(input.origin.unwrap_or_else(|| "compatibles".into())),
            subject_id: input.subject_id,
        };
        let url = format!("{}/rate/v2/initiate", self.settings.base_url);
        let res = self
            .http_post(&url, &serde_json::to_value(&payload).unwrap())
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let body = self.parse_response::<LikeResponse>(res).await?;
        if self.auto_persist
            && let Some(path) = self.recs_cache_path()
        {
            let _ = self.save_recommendations(&path);
        }
        Ok(body)
    }

    pub async fn respond_rate(
        &self,
        mut payload: RateRespondRequest,
    ) -> Result<RateRespondResponse, HingeError> {
        // Generate rating_id if not provided
        if payload.rating_id.is_none() {
            payload.rating_id = Some(Uuid::new_v4().to_string().to_uppercase());
        }

        // Use client session_id if not provided
        if payload.session_id.is_none() {
            payload.session_id = Some(self.session_id.clone());
        }

        // Generate created timestamp if not provided
        if payload.created.is_none() {
            payload.created = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
        }

        let url = format!("{}/rate/v2/respond", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }

    pub async fn update_self_preferences(
        &self,
        preferences: Preferences,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/preference/v2/selected", self.settings.base_url);

        // Convert to API format with numeric enums using type-specific converter
        let prefs_json = preferences_to_api_json(&preferences);
        let payload = serde_json::json!([prefs_json]);

        let res = self.http_patch(&url, &payload).await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let body = res.json::<serde_json::Value>().await?;
        Ok(body)
    }

    pub async fn update_self_profile(
        &self,
        profile_updates: ProfileUpdate,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/user/v3", self.settings.base_url);

        // Convert to API format with numeric enums using type-specific converter
        let profile_json = profile_update_to_api_json(&profile_updates);
        let payload = serde_json::json!({ "profile": profile_json });

        let res = self.http_patch(&url, &payload).await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let body = res.json::<serde_json::Value>().await?;
        Ok(body)
    }

    pub async fn update_answers(
        &self,
        answers: Vec<AnswerContentPayload>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/content/v1/answers", self.settings.base_url);
        let res = self
            .http
            .put(url)
            .headers(self.default_headers()?)
            .json(&answers)
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let body = res.json::<serde_json::Value>().await?;
        Ok(body)
    }

    pub async fn repeat_profiles(&mut self) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/user/repeat", self.settings.base_url);
        let res = self
            .http
            .get(url)
            .headers(self.default_headers()?)
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let body = res.json::<serde_json::Value>().await?;
        if self.auto_persist
            && let Some(path) = self.recs_cache_path()
        {
            let _ = self.save_recommendations(&path);
        }
        Ok(body)
    }

    async fn authenticate_with_sendbird(&mut self) -> Result<(), HingeError> {
        let _hinge = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?;
        let url = format!("{}/message/authenticate", self.settings.base_url);
        let res = self
            .http
            .post(url)
            .headers(self.default_headers()?)
            .json(&json!({"refresh": false}))
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let v = self.parse_response::<SendbirdAuthToken>(res).await?;
        self.sendbird_auth = Some(v);
        // Session key capture is handled in ensure_sendbird_session via WS handshake.
        if self.auto_persist
            && let Some(path) = &self.session_path
        {
            let _ = self.save_session(path);
        }
        Ok(())
    }

    async fn run_text_review(&self, text: &str, receiver_id: &str) -> Result<String, HingeError> {
        let url = format!("{}/flag/textreview", self.settings.base_url);
        let res = self
            .http
            .post(url)
            .headers(self.default_headers()?)
            .json(&json!({ "text": text, "receiverId": receiver_id }))
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let v = res.json::<serde_json::Value>().await?;
        let run_id = v
            .get("hcmRunId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok(run_id)
    }

    pub async fn is_session_valid(&mut self) -> Result<bool, HingeError> {
        // Check if Hinge token exists
        if self.hinge_auth.is_none() {
            log::warn!("Hinge token is empty, session is invalid.");
            return Ok(false);
        }

        // Check if Sendbird token exists, try to authenticate if not
        if self.sendbird_auth.is_none() {
            log::warn!("Sendbird JWT is empty, reauthenticating...");
            if let Err(e) = self.authenticate_with_sendbird().await {
                log::error!("Failed to reauthenticate with Sendbird: {}", e);
                return Ok(false);
            }
        }

        let now = Utc::now();

        // Check Hinge token validity
        let hinge_token_valid = if let Some(hinge_auth) = &self.hinge_auth {
            hinge_auth.expires > now
        } else {
            false
        };

        // Check Sendbird token validity and re-authenticate if expired
        let sendbird_needs_refresh = if let Some(sb_auth) = &self.sendbird_auth {
            sb_auth.expires <= now
        } else {
            true
        };

        if sendbird_needs_refresh {
            log::warn!("Sendbird JWT has expired or is missing, reauthenticating...");
            if let Err(e) = self.authenticate_with_sendbird().await {
                log::error!("Failed to reauthenticate with Sendbird: {}", e);
                return Ok(false);
            }
        }

        // Re-check Sendbird validity after potential re-authentication
        let sendbird_token_valid = if let Some(sb_auth) = &self.sendbird_auth {
            sb_auth.expires > now
        } else {
            false
        };

        let is_valid = hinge_token_valid && sendbird_token_valid;
        log::info!(
            "Session validity check: is_valid={}, hinge_token_valid={}, sendbird_token_valid={}",
            is_valid,
            hinge_token_valid,
            sendbird_token_valid
        );

        Ok(is_valid)
    }

    pub fn save_recommendations(&self, path: &str) -> Result<(), HingeError> {
        let data = serde_json::to_string_pretty(&self.recommendations)
            .map_err(|e| HingeError::Serde(e.to_string()))?;
        self.storage
            .write_string(path, &data)
            .map_err(|e| HingeError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn load_recommendations(&mut self, path: &str) -> Result<(), HingeError> {
        if !self.storage.exists(path) {
            return Ok(());
        }
        let data = self
            .storage
            .read_to_string(path)
            .map_err(|e| HingeError::Storage(e.to_string()))?;
        self.recommendations =
            serde_json::from_str(&data).map_err(|e| HingeError::Serde(e.to_string()))?;
        Ok(())
    }

    pub fn remove_recommendation(&mut self, subject_id: &str) {
        self.recommendations.remove(subject_id);
    }

    pub async fn prompt_payload(&mut self) -> serde_json::Value {
        // Ported from Python client
        if !self.is_session_valid().await.unwrap_or(false) {
            return json!({});
        }
        let preferences = match self.get_self_preferences().await {
            Ok(v) => v,
            Err(_) => return json!({}),
        };
        let profile = match self.get_self_profile().await {
            Ok(v) => v,
            Err(_) => return json!({}),
        };
        let mut preferences_dict = serde_json::to_value(&preferences).unwrap_or(json!({}));
        let profile_dict = serde_json::to_value(&profile).unwrap_or(json!({}));

        let selected: Vec<String> = preferences_dict
            .get("preferences")
            .and_then(|p| p.get("genderPreferences"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_u64().map(|n| n.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let keep_selected = |mut d: serde_json::Value| {
            if let serde_json::Value::Object(map) = &mut d
                && !selected.is_empty()
            {
                map.retain(|k, _| selected.contains(k));
            }
            d
        };

        if let Some(obj) = preferences_dict
            .get_mut("preferences")
            .and_then(|p| p.get_mut("genderedHeightRanges"))
        {
            *obj = keep_selected(obj.clone());
        }
        if let Some(obj) = preferences_dict
            .get_mut("preferences")
            .and_then(|p| p.get_mut("genderedAgeRanges"))
        {
            *obj = keep_selected(obj.clone());
        }

        if let Some(db) = preferences_dict
            .get_mut("preferences")
            .and_then(|p| p.get_mut("dealbreakers"))
        {
            if let Some(obj) = db.get_mut("genderedHeight") {
                *obj = keep_selected(obj.clone());
            }
            if let Some(obj) = db.get_mut("genderedAge") {
                *obj = keep_selected(obj.clone());
            }
        }

        fn unwrap_visible(obj: &serde_json::Value) -> serde_json::Value {
            match obj {
                serde_json::Value::Object(m) => {
                    if m.contains_key("value") && m.contains_key("visible") {
                        unwrap_visible(&m["value"])
                    } else {
                        let mut out = serde_json::Map::new();
                        for (k, v) in m.iter() {
                            out.insert(k.clone(), unwrap_visible(v));
                        }
                        serde_json::Value::Object(out)
                    }
                }
                serde_json::Value::Array(arr) => {
                    serde_json::Value::Array(arr.iter().map(unwrap_visible).collect())
                }
                _ => obj.clone(),
            }
        }

        let p = profile_dict
            .get("content")
            .map(unwrap_visible)
            .unwrap_or(json!({}));
        let loc_name = profile_dict
            .get("content")
            .and_then(|c| c.get("location"))
            .and_then(|l| l.get("name"))
            .cloned()
            .unwrap_or(json!(null));

        let profile_payload = json!({
          "works": match p.get("works") { Some(v) if v.is_string() => json!([v]), _ => p.get("works").cloned().unwrap_or(json!([])) },
          "sexualOrientations": p.get("sexualOrientations").cloned().unwrap_or(json!([])),
          "didJustJoin": false,
          "smoking": p.get("smoking").cloned().unwrap_or(json!(null)),
          "selfieVerified": p.get("selfieVerified").cloned().unwrap_or(json!(false)),
          "politics": p.get("politics").cloned().unwrap_or(json!(null)),
          "relationshipTypesText": p.get("relationshipTypesText").cloned().unwrap_or(json!("")),
          "datingIntention": p.get("datingIntention").cloned().unwrap_or(json!(null)),
          "height": p.get("height").cloned().unwrap_or(json!(null)),
          "children": p.get("children").cloned().unwrap_or(json!(null)),
          "religions": p.get("religions").cloned().unwrap_or(json!([])),
          "relationshipTypes": p.get("relationshipTypeIds").cloned().unwrap_or(json!([])),
          "educations": p.get("educations").cloned().unwrap_or(json!([])),
          "age": p.get("age").cloned().unwrap_or(json!(null)),
          "jobTitle": p.get("jobTitle").cloned().unwrap_or(json!(null)),
          "birthday": p.get("birthday").cloned().unwrap_or(json!(null)),
          "drugs": p.get("drugs").cloned().unwrap_or(json!(null)),
          "content": json!({}),
          "hometown": p.get("hometown").cloned().unwrap_or(json!(null)),
          "firstName": p.get("firstName").cloned().unwrap_or(json!(null)),
          "familyPlans": p.get("familyPlans").cloned().unwrap_or(json!(null)),
          "location": json!({"name": loc_name}),
          "marijuana": p.get("marijuana").cloned().unwrap_or(json!(null)),
          "pets": p.get("pets").cloned().unwrap_or(json!([])),
          "datingIntentionText": p.get("datingIntentionText").cloned().unwrap_or(json!("")),
          "educationAttained": p.get("educationAttained").cloned().unwrap_or(json!(null)),
          "ethnicities": p.get("ethnicities").cloned().unwrap_or(json!([])),
          "pronouns": p.get("pronouns").cloned().unwrap_or(json!([])),
          "languagesSpoken": p.get("languagesSpoken").cloned().unwrap_or(json!([])),
          "lastName": p.get("lastName").cloned().unwrap_or(json!("")),
          "ethnicitiesText": p.get("ethnicitiesText").cloned().unwrap_or(json!("")),
          "drinking": p.get("drinking").cloned().unwrap_or(json!(null)),
          "userId": profile_dict.get("userId").cloned().unwrap_or(json!(null)),
          "genderIdentityId": p.get("genderIdentityId").cloned().unwrap_or(json!(null)),
        });

        json!({
          "preferences": preferences_dict.get("preferences").cloned().unwrap_or(json!({})),
          "profile": profile_payload
        })
    }

    pub async fn send_message(
        &self,
        mut payload: crate::models::SendMessagePayload,
    ) -> Result<serde_json::Value, HingeError> {
        // Ensure Sendbird DM channel exists for this subject before sending via Hinge
        // We clone a mutable self to call sendbird helpers (since self is &self here); alternatively make self &mut.
        let mut cloned = self.clone();
        let self_user_id = cloned
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?
            .identity_id
            .clone();
        let _channel_url = cloned
            .sendbird_get_or_create_dm_channel(&self_user_id, &payload.subject_id)
            .await
            .unwrap_or_else(|e| {
                log::warn!("sendbird get-or-create failed before send: {}", e);
                String::new()
            });
        // Generate dedupId if not provided
        if payload.dedup_id.is_none() {
            payload.dedup_id = Some(Uuid::new_v4().to_string().to_uppercase());
        }

        let url = format!("{}/message/send", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }

    pub async fn evaluate_answer(
        &self,
        payload: AnswerEvaluateRequest,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/content/v1/answer/evaluate", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }

    pub async fn create_prompt_poll(
        &self,
        payload: CreatePromptPollRequest,
    ) -> Result<CreatePromptPollResponse, HingeError> {
        let url = format!("{}/content/v1/prompt_poll", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }

    pub async fn create_video_prompt(
        &self,
        payload: CreateVideoPromptRequest,
    ) -> Result<CreateVideoPromptResponse, HingeError> {
        let url = format!("{}/content/v1/video_prompt", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }

    pub async fn get_connections_v2(&self) -> Result<ConnectionsResponse, HingeError> {
        let url = format!("{}/connection/v2", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_connection_detail(
        &self,
        subject_id: &str,
    ) -> Result<ConnectionDetailApi, HingeError> {
        let url = format!(
            "{}/connection/subject/{}",
            self.settings.base_url, subject_id
        );
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_connection_match_note(
        &self,
        subject_id: &str,
    ) -> Result<MatchNoteResponse, HingeError> {
        let url = format!(
            "{}/connection/v2/matchnote/{}",
            self.settings.base_url, subject_id
        );
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_standouts(&self) -> Result<StandoutsResponse, HingeError> {
        let url = format!("{}/standouts/v3", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn delete_content(&self, content_ids: Vec<String>) -> Result<(), HingeError> {
        let url = format!(
            "{}/content/v1?ids={}",
            self.settings.base_url,
            content_ids.join(",")
        );
        let res = self
            .http
            .delete(url)
            .headers(self.default_headers()?)
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        Ok(())
    }

    pub async fn get_content_settings(&self) -> Result<UserSettings, HingeError> {
        let url = format!("{}/content/v1/settings", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn update_content_settings(
        &self,
        settings: UserSettings,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/content/v1/settings", self.settings.base_url);
        let payload =
            serde_json::to_value(&settings).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_patch(&url, &payload).await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let body = res.json::<serde_json::Value>().await?;
        Ok(body)
    }

    pub async fn get_auth_settings(&self) -> Result<AuthSettings, HingeError> {
        let url = format!("{}/auth/settings", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_notification_settings(&self) -> Result<NotificationSettings, HingeError> {
        let url = format!("{}/notification/v1/settings", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_user_traits(&self) -> Result<Vec<UserTrait>, HingeError> {
        let url = format!("{}/user/v2/traits", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_account_info(&self) -> Result<AccountInfo, HingeError> {
        let url = format!("{}/store/v2/account", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_export_status(&self) -> Result<ExportStatus, HingeError> {
        let url = format!("{}/user/export/status", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn raw_hinge_json(
        &self,
        method: reqwest::Method,
        path_or_url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
            path_or_url.to_string()
        } else {
            format!(
                "{}/{}",
                self.settings.base_url.trim_end_matches('/'),
                path_or_url.trim_start_matches('/')
            )
        };
        let headers = self.default_headers()?;
        log_request(method.as_str(), &url, &headers, body.as_ref());
        let mut request = self.http.request(method, &url).headers(headers);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let res = request.send().await?;
        self.parse_response(res).await
    }

    pub async fn raw_sendbird_json(
        &self,
        method: reqwest::Method,
        path_or_url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
            path_or_url.to_string()
        } else {
            format!(
                "{}/v3/{}",
                self.settings.sendbird_api_url.trim_end_matches('/'),
                path_or_url.trim_start_matches('/')
            )
        };
        let mut headers = self.sendbird_headers()?;
        if body.is_some() {
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/json"),
            );
        }
        log_request(method.as_str(), &url, &headers, body.as_ref());
        let mut request = self.http.request(method, &url).headers(headers);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let res = request.send().await?;
        self.parse_response(res).await
    }
}

fn merge_recommendation_responses(
    base: &mut RecommendationsResponse,
    mut additional: RecommendationsResponse,
) {
    let mut feed_index: HashMap<String, usize> = HashMap::new();
    for (idx, feed) in base.feeds.iter().enumerate() {
        feed_index.insert(feed.origin.clone(), idx);
    }

    for feed in additional.feeds.drain(..) {
        if let Some(&idx) = feed_index.get(&feed.origin) {
            let existing_feed = &mut base.feeds[idx];
            let mut seen: HashSet<String> = existing_feed
                .subjects
                .iter()
                .map(|s| s.subject_id.clone())
                .collect();
            for mut subj in feed.subjects {
                if seen.insert(subj.subject_id.clone()) {
                    if subj.origin.is_none() {
                        subj.origin = Some(feed.origin.clone());
                    }
                    existing_feed.subjects.push(subj);
                }
            }
            if existing_feed.permission.is_none() {
                existing_feed.permission = feed.permission;
            }
            if existing_feed.preview.is_none() {
                existing_feed.preview = feed.preview;
            }
        } else {
            let mut new_feed = feed;
            for subj in &mut new_feed.subjects {
                if subj.origin.is_none() {
                    subj.origin = Some(new_feed.origin.clone());
                }
            }
            feed_index.insert(new_feed.origin.clone(), base.feeds.len());
            base.feeds.push(new_feed);
        }
    }

    match (&mut base.active_pills, additional.active_pills) {
        (Some(existing), Some(mut incoming)) => {
            let mut seen: HashSet<String> = existing.iter().map(|pill| pill.id.clone()).collect();
            for pill in incoming.drain(..) {
                if seen.insert(pill.id.clone()) {
                    existing.push(pill);
                }
            }
        }
        (None, Some(pills)) => base.active_pills = Some(pills),
        _ => {}
    }

    if base.cache_control.is_none() && additional.cache_control.is_some() {
        base.cache_control = additional.cache_control;
    }
}

fn normalize_recommendations_response(response: &mut RecommendationsResponse) {
    let mut ordered_subjects: Vec<RecommendationSubject> = Vec::new();
    let mut seen = HashSet::new();

    for feed in &response.feeds {
        for subj in &feed.subjects {
            if seen.insert(subj.subject_id.clone()) {
                let mut clone = subj.clone();
                if clone.origin.is_none() {
                    clone.origin = Some(feed.origin.clone());
                }
                ordered_subjects.push(clone);
            }
        }
    }

    let (permission, preview) = response
        .feeds
        .first()
        .map(|feed| (feed.permission.clone(), feed.preview.clone()))
        .unwrap_or((None, None));

    let origin = response
        .feeds
        .first()
        .map(|feed| feed.origin.clone())
        .unwrap_or_else(|| "combined".to_string());

    response.feeds = vec![crate::models::RecommendationsFeed {
        id: 0,
        origin,
        subjects: ordered_subjects,
        permission,
        preview,
    }];
}

fn summarize_connection_initiation(
    connection: &ConnectionItem,
    self_user_id: &str,
    peer_user_id: &str,
    peer_display_name: &str,
) -> Option<Vec<String>> {
    let initiator_id = connection.initiator_id.trim();
    let initiator_label = if initiator_id.is_empty() {
        "Unknown".to_string()
    } else if initiator_id == self_user_id {
        "You".to_string()
    } else if initiator_id == peer_user_id {
        peer_display_name.to_string()
    } else {
        initiator_id.to_string()
    };

    let mut lines = Vec::new();
    if let Some(with_label) = prettify_initiated_with(&connection.initiated_with) {
        lines.push(format!(
            "Conversation initiated by {} via {}.",
            initiator_label, with_label
        ));
    } else {
        lines.push(format!("Conversation initiated by {}.", initiator_label));
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut detail_lines = Vec::new();
    for content in &connection.sent_content {
        for description in describe_connection_content_item(content) {
            if seen.insert(description.clone()) {
                detail_lines.push(description);
            }
        }
    }

    for detail in detail_lines {
        lines.push(format!("  • {}", detail));
    }

    Some(lines)
}

fn describe_connection_content_item(item: &ConnectionContentItem) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(prompt) = &item.prompt {
        let question = prompt.question.trim();
        let answer = prompt.answer.trim();
        if !question.is_empty() && !answer.is_empty() {
            lines.push(format!("Prompt \"{}\" – \"{}\"", question, answer));
        } else if !question.is_empty() {
            lines.push(format!("Prompt \"{}\"", question));
        } else if !answer.is_empty() {
            lines.push(format!("Prompt answer \"{}\"", answer));
        }
    }

    if let Some(comment) = &item.comment {
        let trimmed = comment.trim();
        if !trimmed.is_empty() {
            lines.push(format!("Comment: {}", trimmed));
        }
    }

    if let Some(photo) = &item.photo {
        let caption = photo.caption.as_deref().map(str::trim).unwrap_or("");
        if !caption.is_empty() {
            lines.push(format!("Photo liked – {}", caption));
        } else {
            lines.push("Photo liked".to_string());
        }
    }

    if let Some(video) = &item.video {
        if !video.url.trim().is_empty() {
            lines.push("Video shared".to_string());
        } else {
            lines.push("Video interaction".to_string());
        }
    }

    lines
}

fn prettify_initiated_with(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let words: Vec<String> = trimmed
        .split(['_', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                let mut result = first.to_uppercase().collect::<String>();
                result.push_str(&chars.as_str().to_lowercase());
                result
            } else {
                String::new()
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

fn render_profile(
    profile: Option<&PublicUserProfile>,
    content: Option<&ProfileContentFull>,
    prompts: Option<&HingePromptsManager>,
) -> String {
    let mut out = String::new();

    if let Some(wrapper) = profile {
        let p = &wrapper.profile;
        let _ = writeln!(out, "Name: {}", p.first_name);
        if let Some(age) = p.age {
            let _ = writeln!(out, "Age: {}", age);
        }
        if let Some(height) = p.height {
            let _ = writeln!(out, "Height: {} cm", height);
        }
        if let Some(children) = label_from_map(CHILDREN_LABELS, p.children) {
            let _ = writeln!(out, "Children: {}", children);
        }
        if let Some(label) = label_from_map(DATING_LABELS, p.dating_intention) {
            let _ = writeln!(out, "Dating intention: {}", label);
        }
        if let Some(label) = label_from_map(DRINKING_LABELS, p.drinking) {
            let _ = writeln!(out, "Drinking: {}", label);
        }
        if let Some(label) = label_from_map(SMOKING_LABELS, p.smoking) {
            let _ = writeln!(out, "Smoking: {}", label);
        }
        if let Some(label) = label_from_map(MARIJUANA_LABELS, p.marijuana) {
            let _ = writeln!(out, "Marijuana: {}", label);
        }
        if let Some(label) = label_from_map(DRUG_LABELS, p.drugs) {
            let _ = writeln!(out, "Drugs: {}", label);
        }
        let relationship_labels =
            labels_from_map(RELATIONSHIP_TYPE_LABELS, &p.relationship_type_ids);
        if !relationship_labels.is_empty() {
            let _ = writeln!(
                out,
                "Relationship types: {}",
                relationship_labels.join(", ")
            );
        }
        if let Some(job) = p.job_title.as_ref().filter(|v| !v.trim().is_empty()) {
            let _ = writeln!(out, "Job title: {}", job);
        }
        if let Some(work) = p.works.as_ref().filter(|v| !v.trim().is_empty()) {
            let _ = writeln!(out, "Workplace: {}", work);
        }
        if let Some(level) = p.education_attained.as_ref() {
            let _ = writeln!(out, "Education level: {}", education_attained_label(level));
        }
        if let Some(schools) = p.educations.as_ref() {
            let entries: Vec<&str> = schools
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !entries.is_empty() {
                let _ = writeln!(out, "Education: {}", entries.join(", "));
            }
        }
        if !p.location.name.trim().is_empty() {
            let _ = writeln!(out, "Location: {}", p.location.name);
        }
        out.push('\n');
    } else {
        out.push_str("Profile information unavailable.\n\n");
    }

    if let Some(full) = content
        && !full.content.answers.is_empty()
    {
        out.push_str("Prompts:\n");
        for answer in &full.content.answers {
            let response = answer
                .response
                .as_ref()
                .map(|text| text.trim())
                .filter(|text| !text.is_empty());
            if let Some(resp) = response {
                let mut question: Option<String> = None;

                if let Some(mgr) = prompts
                    && let Some(prompt_id) = answer.prompt_id.as_ref()
                {
                    let text = mgr.get_prompt_display_text(prompt_id);
                    if !text.trim().is_empty() && text != "Unknown Question" {
                        question = Some(text);
                    }
                }

                if question.is_none()
                    && let Some(mgr) = prompts
                    && let Some(question_id) = answer.question_id.as_ref()
                {
                    let text = mgr.get_prompt_display_text(question_id);
                    if !text.trim().is_empty() && text != "Unknown Question" {
                        question = Some(text);
                    }
                }

                if question.is_none() {
                    question = answer
                        .content
                        .as_ref()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }

                if question.is_none() {
                    question = answer
                        .question_id
                        .as_ref()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }

                if question.is_none() {
                    question = answer
                        .prompt_id
                        .as_ref()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }

                let question = question.unwrap_or_else(|| "Prompt".to_string());
                let _ = writeln!(out, "- {}: {}", question, resp);
            }
        }
        out.push('\n');
    }

    out
}

impl<S: Storage + Clone> HingeClient<S> {
    fn prepare_user_id_chunks(&self, user_ids: Vec<String>) -> Vec<Vec<String>> {
        // Accept numeric IDs or 32-char hex user IDs (observed in likes feed)
        fn is_user_id_like(id: &str) -> bool {
            if id.is_empty() {
                return false;
            }
            let trimmed = id.trim();
            if trimmed.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
            trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_hexdigit())
        }

        let (mut accepted, mut dropped) = (Vec::new(), 0usize);
        let mut seen: HashSet<String> = HashSet::new();
        for raw in user_ids.into_iter() {
            let id = raw.trim().to_string();
            if is_user_id_like(&id) && seen.insert(id.clone()) {
                accepted.push(id);
            } else {
                dropped += 1;
            }
        }

        if accepted.is_empty() {
            log::warn!("No valid user IDs to fetch (dropped {})", dropped);
            return Vec::new();
        }
        if dropped > 0 {
            log::debug!("Dropped {} non user-like IDs from public fetch", dropped);
        }

        let batch_size = self.public_ids_batch_size.max(1);
        let mut out: Vec<Vec<String>> = Vec::new();
        let mut idx = 0usize;
        while idx < accepted.len() {
            let end = (idx + batch_size).min(accepted.len());
            out.push(accepted[idx..end].to_vec());
            idx = end;
        }
        if out.len() > 1 {
            log::info!(
                "Fetching public user data in {} batches of up to {} IDs",
                out.len(),
                batch_size
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        MessageData, SendMessagePayload, SendbirdChannelsResponse, SendbirdMessagesResponse,
    };
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
}
