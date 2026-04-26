#![recursion_limit = "512"]

use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use hinge_rs::models::*;
use hinge_rs::ws::SendbirdWsEvent;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "openapi".to_string());
    match command.as_str() {
        "openapi" => generate_openapi(OpenApiOptions::parse(args)?),
        "version" => {
            println!("{}", hinge_rs::VERSION);
            Ok(())
        }
        other => Err(format!("unknown xtask command: {other}").into()),
    }
}

#[derive(Debug)]
struct OpenApiOptions {
    site_dir: Option<PathBuf>,
    site_current: String,
    site_root: String,
    site_latest_version: String,
    site_versions: Vec<String>,
    site_versions_url: String,
    write_versions_json: bool,
}

impl Default for OpenApiOptions {
    fn default() -> Self {
        Self {
            site_dir: None,
            site_current: "latest".to_string(),
            site_root: "./".to_string(),
            site_latest_version: hinge_rs::VERSION.to_string(),
            site_versions: Vec::new(),
            site_versions_url: "./versions.json".to_string(),
            write_versions_json: true,
        }
    }
}

impl OpenApiOptions {
    fn parse(args: impl Iterator<Item = String>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut options = Self::default();
        let mut args = args.peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--site-dir" => {
                    options.site_dir = Some(PathBuf::from(next_value(&mut args, "--site-dir")?));
                }
                "--site-current" => {
                    options.site_current = next_value(&mut args, "--site-current")?;
                }
                "--site-root" => {
                    options.site_root = next_value(&mut args, "--site-root")?;
                }
                "--site-latest-version" => {
                    options.site_latest_version = next_value(&mut args, "--site-latest-version")?;
                }
                "--site-versions" => {
                    options.site_versions = next_value(&mut args, "--site-versions")?
                        .split(',')
                        .filter(|version| !version.is_empty())
                        .map(ToOwned::to_owned)
                        .collect();
                }
                "--site-versions-url" => {
                    options.site_versions_url = next_value(&mut args, "--site-versions-url")?;
                }
                "--no-versions-json" => {
                    options.write_versions_json = false;
                }
                other => return Err(format!("unknown openapi option: {other}").into()),
            }
        }
        Ok(options)
    }
}

fn next_value(
    args: &mut impl Iterator<Item = String>,
    name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    args.next()
        .ok_or_else(|| format!("{name} requires a value").into())
}

fn generate_openapi(options: OpenApiOptions) -> Result<(), Box<dyn std::error::Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask should live under crate root")
        .to_path_buf();
    let spec = openapi_document();

    let openapi_dir = crate_root.join("openapi");
    let docs_dir = crate_root.join("docs/api");
    fs::create_dir_all(&openapi_dir)?;
    fs::create_dir_all(&docs_dir)?;

    let spec_text = format!("{}\n", serde_json::to_string_pretty(&spec)?);
    fs::write(openapi_dir.join("hinge-api.openapi.json"), &spec_text)?;
    fs::write(docs_dir.join("openapi.json"), &spec_text)?;
    fs::write(
        docs_dir.join("index.html"),
        scalar_index_html(&ScalarPageConfig::latest(
            hinge_rs::VERSION,
            "./",
            "./versions.json",
        ))?,
    )?;
    fs::write(
        docs_dir.join("versions.json"),
        versions_json(hinge_rs::VERSION, &[])?,
    )?;

    if let Some(site_dir) = options.site_dir {
        fs::create_dir_all(&site_dir)?;
        let page_config = ScalarPageConfig {
            current_version: options.site_current,
            latest_version: options.site_latest_version,
            site_root: options.site_root,
            versions_url: options.site_versions_url,
        };
        fs::write(site_dir.join("openapi.json"), &spec_text)?;
        fs::write(
            site_dir.join("index.html"),
            scalar_index_html(&page_config)?,
        )?;
        if options.write_versions_json {
            fs::write(
                site_dir.join("versions.json"),
                versions_json(&page_config.latest_version, &options.site_versions)?,
            )?;
        }
    }

    println!(
        "generated {}, {}, {}, {}",
        display(&openapi_dir.join("hinge-api.openapi.json")),
        display(&docs_dir.join("openapi.json")),
        display(&docs_dir.join("index.html")),
        display(&docs_dir.join("versions.json"))
    );
    Ok(())
}

fn display(path: &Path) -> String {
    path.strip_prefix(std::env::current_dir().unwrap_or_default())
        .unwrap_or(path)
        .display()
        .to_string()
}

#[derive(Debug)]
struct ScalarPageConfig {
    current_version: String,
    latest_version: String,
    site_root: String,
    versions_url: String,
}

impl ScalarPageConfig {
    fn latest(latest_version: &str, site_root: &str, versions_url: &str) -> Self {
        Self {
            current_version: "latest".to_string(),
            latest_version: latest_version.to_string(),
            site_root: site_root.to_string(),
            versions_url: versions_url.to_string(),
        }
    }
}

fn versions_json(latest_version: &str, versions: &[String]) -> Result<String, serde_json::Error> {
    let value = json!({
        "latest": latest_version,
        "versions": versions,
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

fn scalar_index_html(config: &ScalarPageConfig) -> Result<String, serde_json::Error> {
    let current_version = serde_json::to_string(&config.current_version)?;
    let latest_version = serde_json::to_string(&config.latest_version)?;
    let site_root = serde_json::to_string(&config.site_root)?;
    let versions_url = serde_json::to_string(&config.versions_url)?;

    Ok(r#"<!doctype html>
<html lang="en">
  <head>
    <title>hinge-rs API Reference</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
      body {
        margin: 0;
      }
      .version-picker {
        position: fixed;
        top: 12px;
        right: 16px;
        z-index: 20;
      }
      .version-picker select {
        height: 32px;
        border: 1px solid #d0d5dd;
        border-radius: 6px;
        background: #fff;
        color: #111827;
        font: 13px/1.2 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        padding: 0 28px 0 10px;
      }
    </style>
  </head>
  <body>
    <div class="version-picker">
      <select id="version-select" aria-label="API reference version"></select>
    </div>
    <div id="app"></div>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
    <script>
      const currentVersion = __CURRENT_VERSION__;
      const latestVersion = __LATEST_VERSION__;
      const siteRoot = __SITE_ROOT__;
      const versionsUrl = __VERSIONS_URL__;

      const select = document.getElementById('version-select');
      const addVersionOption = (label, value, selected) => {
        const option = document.createElement('option');
        option.textContent = label;
        option.value = value;
        option.selected = selected;
        select.appendChild(option);
      };

      fetch(versionsUrl)
        .then((response) => response.ok ? response.json() : { latest: latestVersion, versions: [] })
        .then((data) => {
          addVersionOption(`latest (${data.latest || latestVersion})`, siteRoot, currentVersion === 'latest');
          for (const version of data.versions || []) {
            addVersionOption(`v${version}`, `${siteRoot}v/${version}/`, currentVersion === version);
          }
          select.addEventListener('change', () => {
            window.location.href = select.value;
          });
        });

      Scalar.createApiReference('#app', {
        url: './openapi.json',
        layout: 'modern',
        theme: 'default',
        defaultHttpClient: {
          targetKey: 'rust',
          clientKey: 'reqwest'
        },
        expandAllResponses: true,
        persistAuth: false,
        telemetry: false,
        metaData: {
          title: 'hinge-rs API Reference',
          description: 'Unofficial typed Hinge REST and Sendbird chat API reference for Rust.'
        }
      })
    </script>
  </body>
</html>
"#
    .replace("__CURRENT_VERSION__", &current_version)
    .replace("__LATEST_VERSION__", &latest_version)
    .replace("__SITE_ROOT__", &site_root)
    .replace("__VERSIONS_URL__", &versions_url))
}

fn openapi_document() -> Value {
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "hinge-rs",
            "version": hinge_rs::VERSION,
            "description": "Unofficial typed Hinge REST and Sendbird chat API surface generated from hinge-rs.",
            "license": {
                "name": "MIT OR Apache-2.0",
                "identifier": "MIT OR Apache-2.0"
            },
            "x-scalar-sdk-installation": [
                {
                    "lang": "Rust",
                    "description": "Install the typed Rust client from crates.io:",
                    "source": "cargo add hinge-rs"
                }
            ]
        },
        "servers": [
            {
                "url": "https://prod-api.hingeaws.net",
                "description": "Hinge production API"
            },
            {
                "url": "https://api-{sendbirdAppId}.sendbird.com/v3",
                "description": "Sendbird REST API",
                "variables": {
                    "sendbirdAppId": {
                        "default": "3cdad91c-1e0d-4a0d-bbee-9671988bf9e9"
                    }
                }
            }
        ],
        "x-scalar-environments": {
            "production": {
                "description": "Hinge production and Sendbird production",
                "color": "#2563eb",
                "variables": {
                    "hingeBaseUrl": {
                        "description": "Hinge REST API base URL",
                        "default": "https://prod-api.hingeaws.net"
                    },
                    "sendbirdApiUrl": {
                        "description": "Sendbird REST API base URL",
                        "default": "https://api-3cdad91c-1e0d-4a0d-bbee-9671988bf9e9.sendbird.com/v3"
                    }
                }
            },
            "local-mock": {
                "description": "Local Wiremock / WebSocket test server",
                "color": "#059669",
                "variables": {
                    "hingeBaseUrl": "http://127.0.0.1:8080",
                    "sendbirdApiUrl": "http://127.0.0.1:8081/v3"
                }
            }
        },
        "tags": [
            {"name": "Auth", "description": "Device install, SMS OTP, email 2FA, and auth settings."},
            {"name": "Recommendations", "description": "Recommendation feeds, standouts, and repeat profiles."},
            {"name": "Profiles", "description": "Self, public, and profile-content APIs."},
            {"name": "Likes", "description": "Inbound likes, like limits, and like subject lookup."},
            {"name": "Ratings", "description": "Like, note, superlike, skip, and response flows."},
            {"name": "Prompts", "description": "Prompt catalog and prompt-content creation."},
            {"name": "Connections", "description": "Matches, connection detail, and match notes."},
            {"name": "Settings", "description": "Preferences, notifications, user traits, account, and export status."},
            {"name": "Chat", "description": "Hinge chat helpers and local export support."},
            {"name": "Sendbird REST", "description": "Sendbird channel, message, and group-channel endpoints used by Hinge chat."},
            {"name": "Sendbird WebSocket", "description": "Documented Sendbird WebSocket handshake and typed frame events."}
        ],
        "paths": paths(),
        "components": components()
    })
}

fn paths() -> Value {
    json!({
        "/identity/install": {
            "post": operation("Auth", "Register device install", "Registers a generated install ID before SMS login.", Some(schema_ref("InstallRequest")), Some(schema_ref("EmptyObject")), false)
        },
        "/auth/sms/v2/initiate": {
            "post": operation("Auth", "Initiate SMS login", "Starts the SMS OTP login flow for a phone number.", Some(schema_ref("SmsInitiateRequest")), Some(schema_ref("EmptyObject")), false)
        },
        "/auth/sms/v2": {
            "post": operation("Auth", "Submit SMS OTP", "Exchanges a phone number and OTP for Hinge and Sendbird tokens. A 412 response means email 2FA is required.", Some(schema_ref("OtpSubmitRequest")), Some(schema_ref("LoginTokens")), false)
        },
        "/auth/device/validate": {
            "post": operation("Auth", "Submit email 2FA code", "Completes the email-device validation flow when OTP returns email 2FA.", Some(schema_ref("EmailCodeRequest")), Some(schema_ref("LoginTokens")), false)
        },
        "/user/v3": {
            "get": operation("Profiles", "Get self profile", "Fetches the authenticated user's profile.", None, Some(schema_ref("SelfProfileResponse")), true),
            "patch": operation("Profiles", "Update self profile", "Applies profile updates using Hinge's numeric enum wire format.", Some(schema_ref("ProfileUpdateRequest")), Some(schema_ref("RawJson")), true)
        },
        "/content/v2": {
            "get": operation("Profiles", "Get self content", "Fetches photos, prompts, and other profile content for the authenticated user.", None, Some(schema_ref("SelfContentResponse")), true)
        },
        "/preference/v2/selected": {
            "get": operation("Profiles", "Get self preferences", "Fetches selected dating preferences.", None, Some(schema_ref("PreferencesResponse")), true),
            "patch": operation("Settings", "Update self preferences", "Updates selected dating preferences.", Some(schema_ref("PreferencesUpdateRequest")), Some(schema_ref("RawJson")), true)
        },
        "/rec/v2": {
            "post": operation("Recommendations", "Get recommendations", "Fetches recommendation feeds for the authenticated user.", Some(schema_ref("RecommendationsRequest")), Some(schema_ref("RecommendationsResponse")), true)
        },
        "/user/v3/public": {
            "get": operation_with_params("Profiles", "Get public profiles", "Fetches public profile records by comma-separated user IDs.", vec![query_param("ids", "Comma-separated user IDs", true)], None, Some(schema_ref("PublicProfilesResponse")), true)
        },
        "/content/v2/public": {
            "get": operation_with_params("Profiles", "Get public profile content", "Fetches public content records by comma-separated user IDs.", vec![query_param("ids", "Comma-separated user IDs", true)], None, Some(schema_ref("PublicContentResponse")), true)
        },
        "/likelimit": {
            "get": operation("Likes", "Get like limit", "Fetches remaining standard and super likes.", None, Some(schema_ref("LikeLimit")), true)
        },
        "/like/v2": {
            "get": operation("Likes", "List likes", "Fetches inbound likes.", None, Some(schema_ref("LikesV2Response")), true)
        },
        "/like/subject/{subjectId}": {
            "get": operation_with_params("Likes", "Get like subject", "Fetches one like by subject ID.", vec![path_param("subjectId", "Subject ID")], None, Some(schema_ref("LikeItemV2")), true)
        },
        "/rate/v2/initiate": {
            "post": operation("Ratings", "Rate or skip user", "Creates a like, note, superlike, or skip rating.", Some(schema_ref("CreateRate")), Some(schema_ref("LikeResponse")), true)
        },
        "/rate/v2/respond": {
            "post": operation("Ratings", "Respond to rate", "Responds to an inbound like/match flow.", Some(schema_ref("RateRespondRequest")), Some(schema_ref("RateRespondResponse")), true)
        },
        "/prompts": {
            "post": operation("Prompts", "List prompts", "Fetches prompt catalog using a payload derived from profile and preferences.", Some(schema_ref("PromptPayload")), Some(schema_ref("PromptsResponse")), true)
        },
        "/content/v1/answer/evaluate": {
            "post": operation("Prompts", "Evaluate answer", "Runs Hinge answer review before saving prompt content.", Some(schema_ref("AnswerEvaluateRequest")), Some(schema_ref("RawJson")), true)
        },
        "/content/v1/prompt_poll": {
            "post": operation("Prompts", "Create prompt poll", "Creates prompt poll content.", Some(schema_ref("CreatePromptPollRequest")), Some(schema_ref("CreatePromptPollResponse")), true)
        },
        "/content/v1/video_prompt": {
            "post": operation("Prompts", "Create video prompt", "Creates video prompt content.", Some(schema_ref("CreateVideoPromptRequest")), Some(schema_ref("CreateVideoPromptResponse")), true)
        },
        "/connection/v2": {
            "get": operation("Connections", "List connections", "Fetches current connections/matches.", None, Some(schema_ref("ConnectionsResponse")), true)
        },
        "/connection/subject/{subjectId}": {
            "get": operation_with_params("Connections", "Get connection detail", "Fetches full detail for a connection subject.", vec![path_param("subjectId", "Subject ID")], None, Some(schema_ref("ConnectionDetailApi")), true)
        },
        "/connection/v2/matchnote/{subjectId}": {
            "get": operation_with_params("Connections", "Get match note", "Fetches match note for a subject.", vec![path_param("subjectId", "Subject ID")], None, Some(schema_ref("MatchNoteResponse")), true)
        },
        "/standouts/v3": {
            "get": operation("Recommendations", "Get standouts", "Fetches standout recommendations.", None, Some(schema_ref("StandoutsResponse")), true)
        },
        "/content/v1/settings": {
            "get": operation("Settings", "Get content settings", "Fetches content settings.", None, Some(schema_ref("UserSettings")), true),
            "patch": operation("Settings", "Update content settings", "Updates content settings.", Some(schema_ref("UserSettings")), Some(schema_ref("RawJson")), true)
        },
        "/content/v1/answers": {
            "put": operation("Settings", "Update answers", "Replaces answer content.", Some(schema_ref("AnswersUpdateRequest")), Some(schema_ref("RawJson")), true)
        },
        "/content/v1": {
            "delete": operation_with_params("Profiles", "Delete content", "Deletes profile content by comma-separated content IDs.", vec![query_param("ids", "Comma-separated content IDs", true)], None, Some(schema_ref("EmptyObject")), true)
        },
        "/auth/settings": {
            "get": operation("Settings", "Get auth settings", "Fetches auth settings.", None, Some(schema_ref("AuthSettings")), true)
        },
        "/notification/v1/settings": {
            "get": operation("Settings", "Get notification settings", "Fetches notification settings.", None, Some(schema_ref("NotificationSettings")), true)
        },
        "/user/v2/traits": {
            "get": operation("Settings", "Get user traits", "Fetches user traits.", None, Some(schema_ref("UserTraitsResponse")), true)
        },
        "/store/v2/account": {
            "get": operation("Settings", "Get account info", "Fetches account and subscription information.", None, Some(schema_ref("AccountInfo")), true)
        },
        "/user/export/status": {
            "get": operation("Settings", "Get export status", "Fetches account data export status.", None, Some(schema_ref("ExportStatus")), true)
        },
        "/user/repeat": {
            "get": operation("Recommendations", "Repeat profiles", "Requests repeated profiles from Hinge.", None, Some(schema_ref("RawJson")), true)
        },
        "/message/authenticate": {
            "post": operation("Chat", "Authenticate Sendbird", "Exchanges Hinge auth for a Sendbird JWT.", Some(schema_ref("SendbirdAuthenticateRequest")), Some(schema_ref("SendbirdAuthToken")), true)
        },
        "/flag/textreview": {
            "post": operation("Chat", "Review message text", "Runs Hinge text review and returns an HCM run ID used when sending notes/likes with comments.", Some(schema_ref("TextReviewRequest")), Some(schema_ref("TextReviewResponse")), true)
        },
        "/message/send": {
            "post": operation("Chat", "Send Hinge message", "Sends a message through Hinge after ensuring a Sendbird DM channel exists.", Some(schema_ref("SendMessagePayload")), Some(schema_ref("RawJson")), true)
        },
        "/users/{userId}/my_group_channels": {
            "get": sendbird_operation_with_params("Sendbird REST", "List my Sendbird channels", "Lists Sendbird group channels for the authenticated user using the mobile SDK query shape.", vec![path_param("userId", "Sendbird user ID"), query_param("user_id", "Sendbird user ID repeated by the SDK", true), query_param("limit", "Maximum channels", false), query_param("members_exactly_in", "Peer user ID for exact one-to-one channel lookup", false), query_param("order", "Sendbird channel ordering", false)], None, Some(schema_ref("SendbirdChannelsResponse")))
        },
        "/group_channels": {
            "post": sendbird_operation("Sendbird REST", "Create Sendbird DM channel", "Creates a distinct Sendbird group channel.", Some(schema_ref("SendbirdCreateChannelRequest")), Some(schema_ref("RawJson")))
        },
        "/sdk/group_channels/{channelUrl}": {
            "get": sendbird_operation_with_params("Sendbird REST", "Get Sendbird channel", "Fetches a Sendbird group channel using the SDK endpoint.", vec![path_param("channelUrl", "URL-encoded channel URL")], None, Some(schema_ref("SendbirdGroupChannel")))
        },
        "/group_channels/{channelUrl}/messages": {
            "get": sendbird_operation_with_params("Sendbird REST", "Get Sendbird messages", "Fetches message history around an optional timestamp anchor.", vec![path_param("channelUrl", "URL-encoded channel URL"), query_param("message_ts", "Anchor timestamp", false), query_param("prev_limit", "Number of previous messages", false)], None, Some(schema_ref("SendbirdMessagesResponse")))
        },
        "/sdk/chat/export": {
            "post": operation("Chat", "Export chat helper", "SDK helper that fetches channel/profile context and writes a local markdown export. This is not a remote Hinge endpoint.", Some(schema_ref("ExportChatInput")), Some(schema_ref("ExportChatResult")), true)
        },
        "/sendbird/ws": {
            "get": {
                "tags": ["Sendbird WebSocket"],
                "summary": "Connect Sendbird WebSocket",
                "operationId": "connectSendbirdWebSocket",
                "description": "Pseudo-operation documenting the Sendbird WebSocket handshake and typed events. The Rust client connects to wss://ws-{appId}.sendbird.com/.",
                "security": [{"sendbirdWsAuth": []}],
                "x-codeSamples": rust_code_samples("Connect Sendbird WebSocket"),
                "responses": {
                    "101": {
                        "description": "WebSocket upgrade accepted"
                    },
                    "401": error_response("Unauthorized")
                },
                "x-websocket-events": {
                    "client": ["READ", "PING", "TPST", "TPEN", "ENTR", "EXIT", "MACK", "CLOSE"],
                    "server": ["LOGI", "READ", "SYEV", "PING", "PONG", "CLOSE"]
                }
            }
        }
    })
}

fn components() -> Value {
    let schemas = schemas();
    json!({
        "securitySchemes": {
            "bearerAuth": {
                "type": "http",
                "scheme": "bearer",
                "bearerFormat": "JWT"
            },
            "sendbirdSessionKey": {
                "type": "apiKey",
                "in": "header",
                "name": "Session-Key"
            },
            "sendbirdWsAuth": {
                "type": "apiKey",
                "in": "header",
                "name": "SENDBIRD-WS-AUTH"
            }
        },
        "schemas": schemas
    })
}

fn schemas() -> Value {
    let mut schemas = serde_json::Map::new();

    schemas.insert("EmptyObject".into(), object_schema(vec![]));
    schemas.insert(
        "RawJson".into(),
        json!({
            "description": "Raw JSON object retained as an explicit escape hatch for undocumented response drift.",
            "type": "object",
            "additionalProperties": true
        }),
    );
    schemas.insert(
        "InstallRequest".into(),
        object_schema(vec![("installId", "string")]),
    );
    schemas.insert(
        "SmsInitiateRequest".into(),
        object_schema(vec![("deviceId", "string"), ("phoneNumber", "string")]),
    );
    schemas.insert(
        "OtpSubmitRequest".into(),
        object_schema(vec![
            ("installId", "string"),
            ("deviceId", "string"),
            ("phoneNumber", "string"),
            ("otp", "string"),
        ]),
    );
    schemas.insert(
        "EmailCodeRequest".into(),
        object_schema(vec![
            ("installId", "string"),
            ("deviceId", "string"),
            ("caseId", "string"),
            ("code", "string"),
        ]),
    );
    schemas.insert(
        "RecommendationsRequest".into(),
        object_schema(vec![
            ("playerId", "string"),
            ("newHere", "boolean"),
            ("activeToday", "boolean"),
        ]),
    );
    schemas.insert(
        "PreferencesUpdateRequest".into(),
        json!({"type": "array", "items": schema_ref("Preferences")}),
    );
    schemas.insert(
        "ProfileUpdateRequest".into(),
        schema_for_component::<ProfileUpdate>("ProfileUpdateRequest"),
    );
    schemas.insert(
        "PublicProfilesResponse".into(),
        json!({"type": "array", "items": schema_ref("PublicUserProfile")}),
    );
    schemas.insert(
        "PublicContentResponse".into(),
        json!({"type": "array", "items": schema_ref("ProfileContentFull")}),
    );
    schemas.insert(
        "AnswersUpdateRequest".into(),
        json!({"type": "array", "items": schema_ref("AnswerContentPayload")}),
    );
    schemas.insert(
        "UserTraitsResponse".into(),
        json!({"type": "array", "items": schema_ref("UserTrait")}),
    );
    schemas.insert(
        "PromptPayload".into(),
        generic_object("Prompt catalog request generated from profile and preferences."),
    );
    schemas.insert(
        "SendbirdAuthenticateRequest".into(),
        object_schema(vec![("refresh", "boolean")]),
    );
    schemas.insert(
        "TextReviewRequest".into(),
        object_schema(vec![("text", "string"), ("receiverId", "string")]),
    );
    schemas.insert(
        "TextReviewResponse".into(),
        object_schema(vec![("hcmRunId", "string")]),
    );
    schemas.insert(
        "SendbirdCreateChannelRequest".into(),
        generic_object("Sendbird distinct group channel creation request."),
    );
    schemas.insert("Error".into(), object_schema(vec![("error", "string")]));

    macro_rules! add_schema {
        ($ty:ty) => {
            schemas.insert(
                stringify!($ty).to_string(),
                schema_for_component::<$ty>(stringify!($ty)),
            );
        };
    }

    add_schema!(AccountInfo);
    add_schema!(ActivePill);
    add_schema!(AnswerContentPayload);
    add_schema!(AnswerEvaluateRequest);
    add_schema!(AuthSettings);
    add_schema!(BoundingBox);
    add_schema!(ConnectionContentItem);
    add_schema!(ConnectionDetailApi);
    add_schema!(ConnectionItem);
    add_schema!(ConnectionPrompt);
    add_schema!(ConnectionVideo);
    add_schema!(ConnectionsResponse);
    add_schema!(ContentData);
    add_schema!(CreatePromptPollRequest);
    add_schema!(CreatePromptPollResponse);
    add_schema!(CreateRate);
    add_schema!(CreateRateContent);
    add_schema!(CreateRateContentPrompt);
    add_schema!(CreateVideoPromptRequest);
    add_schema!(CreateVideoPromptResponse);
    add_schema!(Dealbreakers);
    add_schema!(ExportChatInput);
    add_schema!(ExportChatResult);
    add_schema!(ExportStatus);
    add_schema!(ExportedMediaFile);
    add_schema!(Feedback);
    add_schema!(GenderedDealbreaker);
    add_schema!(GenderedRange);
    add_schema!(HingeAuthToken);
    add_schema!(LikeItemV2);
    add_schema!(LikeLimit);
    add_schema!(LikePromptPoll);
    add_schema!(LikeRating);
    add_schema!(LikeRatingContentItem);
    add_schema!(LikeResponse);
    add_schema!(LikeSortV2);
    add_schema!(LikesV2Response);
    add_schema!(Location);
    add_schema!(LoginTokens);
    add_schema!(MatchNoteResponse);
    add_schema!(MessageData);
    add_schema!(NotificationSettings);
    add_schema!(PhotoAsset);
    add_schema!(PhotoAssetInput);
    add_schema!(Preferences);
    add_schema!(PreferencesResponse);
    add_schema!(Profile);
    add_schema!(ProfileAnswer);
    add_schema!(ProfileContent);
    add_schema!(ProfileContentContent);
    add_schema!(ProfileContentFull);
    add_schema!(ProfileName);
    add_schema!(ProfileUpdate);
    add_schema!(Prompt);
    add_schema!(PromptCategory);
    add_schema!(PromptPoll);
    add_schema!(PromptsResponse);
    add_schema!(PublicProfile);
    add_schema!(PublicUserProfile);
    add_schema!(RangeDetails);
    add_schema!(RateContentPayload);
    add_schema!(RateInput);
    add_schema!(RatePayload);
    add_schema!(RateRespondRequest);
    add_schema!(RateRespondResponse);
    add_schema!(RecsV2Params);
    add_schema!(RecommendationSubject);
    add_schema!(RecommendationsFeed);
    add_schema!(RecommendationsPreview);
    add_schema!(RecommendationsResponse);
    add_schema!(SelfContentResponse);
    add_schema!(SelfProfileResponse);
    add_schema!(SendMessagePayload);
    add_schema!(SendbirdAuthToken);
    add_schema!(SendbirdChannelHandle);
    add_schema!(SendbirdChannelMember);
    add_schema!(SendbirdChannelsInput);
    add_schema!(SendbirdChannelsResponse);
    add_schema!(SendbirdCloseRequest);
    add_schema!(SendbirdGetMessagesInput);
    add_schema!(SendbirdGroupChannel);
    add_schema!(SendbirdMessage);
    add_schema!(SendbirdMessageMetaItem);
    add_schema!(SendbirdMessageUser);
    add_schema!(SendbirdMessagesResponse);
    add_schema!(SendbirdReadResponse);
    add_schema!(SendbirdReadUser);
    add_schema!(SendbirdSyevEvent);
    add_schema!(SendbirdSyevUserData);
    add_schema!(SkipInput);
    add_schema!(SortedLikeIdV2);
    add_schema!(SortedLikesGroupV2);
    add_schema!(StandoutContent);
    add_schema!(StandoutItem);
    add_schema!(StandoutMediaRef);
    add_schema!(StandoutsResponse);
    add_schema!(UserProfile);
    add_schema!(UserSettings);
    add_schema!(UserTrait);
    add_schema!(VideoPrompt);
    add_schema!(VoiceAnswerPayload);
    add_schema!(SendbirdWsEvent);

    flatten_component_defs(&mut schemas);

    Value::Object(schemas)
}

fn schema_for_component<T: schemars::JsonSchema>(component_name: &str) -> Value {
    let mut schema = serde_json::to_value(schemars::schema_for!(T)).expect("schema serializes");
    strip_schema_meta(&mut schema);
    rewrite_local_schema_refs(&mut schema, component_name);
    schema
}

fn strip_schema_meta(schema: &mut Value) {
    if let Value::Object(map) = schema {
        map.remove("$schema");
    }
}

fn rewrite_local_schema_refs(schema: &mut Value, component_name: &str) {
    match schema {
        Value::Object(map) => {
            if let Some(Value::String(reference)) = map.get_mut("$ref")
                && let Some(suffix) = reference.strip_prefix("#/$defs/")
            {
                *reference = format!("#/components/schemas/{component_name}/$defs/{suffix}");
            }
            for value in map.values_mut() {
                rewrite_local_schema_refs(value, component_name);
            }
        }
        Value::Array(values) => {
            for value in values {
                rewrite_local_schema_refs(value, component_name);
            }
        }
        _ => {}
    }
}

fn flatten_component_defs(schemas: &mut Map<String, Value>) {
    let mut component_defs = Vec::new();
    for component_name in schemas.keys().cloned().collect::<Vec<_>>() {
        let Some(schema) = schemas.get_mut(&component_name) else {
            continue;
        };
        let Some(defs) = take_component_defs(schema) else {
            continue;
        };
        component_defs.push((component_name, defs));
    }

    let mut ref_rewrites = BTreeMap::new();
    let mut pending_schemas = BTreeMap::new();

    for (component_name, defs) in component_defs {
        for (def_name, def_schema) in defs {
            let target_name = def_name.clone();
            ref_rewrites.insert(
                format!("#/components/schemas/{component_name}/$defs/{def_name}"),
                format!("#/components/schemas/{target_name}"),
            );

            if !schemas.contains_key(&target_name) {
                pending_schemas.entry(target_name).or_insert(def_schema);
            }
        }
    }

    for (name, schema) in pending_schemas {
        schemas.entry(name).or_insert(schema);
    }

    for schema in schemas.values_mut() {
        rewrite_schema_refs(schema, &ref_rewrites);
    }
}

fn take_component_defs(schema: &mut Value) -> Option<Map<String, Value>> {
    let Value::Object(map) = schema else {
        return None;
    };
    match map.remove("$defs") {
        Some(Value::Object(defs)) => Some(defs),
        Some(defs) => {
            map.insert("$defs".to_string(), defs);
            None
        }
        None => None,
    }
}

fn rewrite_schema_refs(schema: &mut Value, ref_rewrites: &BTreeMap<String, String>) {
    match schema {
        Value::Object(map) => {
            if let Some(Value::String(reference)) = map.get_mut("$ref")
                && let Some(replacement) = ref_rewrites.get(reference)
            {
                *reference = replacement.clone();
            }
            for value in map.values_mut() {
                rewrite_schema_refs(value, ref_rewrites);
            }
        }
        Value::Array(values) => {
            for value in values {
                rewrite_schema_refs(value, ref_rewrites);
            }
        }
        _ => {}
    }
}

fn operation(
    tag: &str,
    summary: &str,
    description: &str,
    request_schema: Option<Value>,
    response_schema: Option<Value>,
    hinge_auth: bool,
) -> Value {
    operation_with_params(
        tag,
        summary,
        description,
        Vec::new(),
        request_schema,
        response_schema,
        hinge_auth,
    )
}

fn sendbird_operation(
    tag: &str,
    summary: &str,
    description: &str,
    request_schema: Option<Value>,
    response_schema: Option<Value>,
) -> Value {
    sendbird_operation_with_params(
        tag,
        summary,
        description,
        Vec::new(),
        request_schema,
        response_schema,
    )
}

fn sendbird_operation_with_params(
    tag: &str,
    summary: &str,
    description: &str,
    parameters: Vec<Value>,
    request_schema: Option<Value>,
    response_schema: Option<Value>,
) -> Value {
    let mut op = operation_with_params(
        tag,
        summary,
        description,
        parameters,
        request_schema,
        response_schema,
        false,
    );
    op["security"] = json!([{"sendbirdSessionKey": []}]);
    op
}

fn operation_with_params(
    tag: &str,
    summary: &str,
    description: &str,
    parameters: Vec<Value>,
    request_schema: Option<Value>,
    response_schema: Option<Value>,
    hinge_auth: bool,
) -> Value {
    let mut response_media = json!({
        "schema": response_schema.unwrap_or_else(|| schema_ref("EmptyObject"))
    });
    if let Some(example) = example_for(summary) {
        response_media["examples"] = json!({
            "success": {
                "value": example
            }
        });
    }

    let mut op = json!({
        "tags": [tag],
        "summary": summary,
        "operationId": operation_id(summary),
        "description": description,
        "parameters": parameters,
        "responses": {
            "200": {
                "description": "Success",
                "content": {
                    "application/json": response_media
                }
            },
            "400": error_response("Bad request"),
            "401": error_response("Unauthorized"),
            "429": error_response("Rate limited"),
            "500": error_response("Server error")
        }
    });

    if let Some(schema) = request_schema {
        let mut request_media = json!({
            "schema": schema
        });
        if let Some(example) = request_example(summary) {
            request_media["examples"] = json!({
                "request": {
                    "value": example
                }
            });
        }

        op["requestBody"] = json!({
            "required": true,
            "content": {
                "application/json": request_media
            }
        });
    }

    if hinge_auth {
        op["security"] = json!([{"bearerAuth": []}]);
    } else {
        op["security"] = json!([]);
    }

    let samples = rust_code_samples(summary);
    if !samples.as_array().is_some_and(|items| items.is_empty()) {
        op["x-codeSamples"] = samples;
    }

    op
}

fn rust_code_samples(summary: &str) -> Value {
    let Some(source) = rust_usage_source(summary) else {
        return json!([]);
    };
    json!([
        {
            "lang": "Rust",
            "label": "hinge-rs",
            "source": source
        }
    ])
}

const CLIENT_SETUP_SAMPLE: &str = r#"let mut client = hinge_rs::Client::builder()
    .phone_number("+15555550123")
    .build()?;"#;

const AUTHENTICATED_CLIENT_SETUP_SAMPLE: &str = r#"let mut client = hinge_rs::Client::builder()
    .phone_number("+15555550123")
    .build()?;
client.persistence().load_session("session.json")?;"#;

fn rust_usage_source(summary: &str) -> Option<String> {
    let setup = if matches!(
        summary,
        "Initiate SMS login" | "Submit SMS OTP" | "Submit email 2FA code"
    ) {
        CLIENT_SETUP_SAMPLE
    } else {
        AUTHENTICATED_CLIENT_SETUP_SAMPLE
    };
    let body = rust_usage_body(summary)?;
    Some(format!("{setup}\n\n{body}"))
}

fn rust_usage_body(summary: &str) -> Option<&'static str> {
    match summary {
        "Initiate SMS login" => Some(r#"client.auth().initiate_sms().await?;"#),
        "Submit SMS OTP" => Some(
            r#"let tokens = client.auth().submit_otp("123456").await?;
client.persistence().save_session("session.json")?;"#,
        ),
        "Submit email 2FA code" => Some(
            r#"let tokens = client
    .auth()
    .submit_email_code("case-id", "123456")
    .await?;
client.persistence().save_session("session.json")?;"#,
        ),
        "Get recommendations" => Some(r#"let recs = client.recommendations().get().await?;"#),
        "Get public profiles" => Some(
            r#"let profiles = client
    .profiles()
    .public(vec!["user_id".to_string()])
    .await?;"#,
        ),
        "Get public profile content" => Some(
            r#"let content = client
    .profiles()
    .public_content(vec!["user_id".to_string()])
    .await?;"#,
        ),
        "Get self profile" => Some(r#"let profile = client.profiles().me().await?;"#),
        "Get self content" => Some(r#"let content = client.profiles().content().await?;"#),
        "Get self preferences" => {
            Some(r#"let preferences = client.settings().preferences().await?;"#)
        }
        "Update self preferences" => Some(
            r#"let response = client
    .settings()
    .update_preferences(preferences)
    .await?;"#,
        ),
        "Update self profile" => {
            Some(r#"let response = client.profiles().update(profile_update).await?;"#)
        }
        "Get like limit" => Some(r#"let limit = client.likes().limit().await?;"#),
        "List likes" => Some(r#"let likes = client.likes().list().await?;"#),
        "Get like subject" => Some(r#"let like = client.likes().subject("subject_id").await?;"#),
        "Rate or skip user" => {
            Some(r#"let response = client.ratings().rate_user(rate_input).await?;"#)
        }
        "Respond to rate" => {
            Some(r#"let response = client.ratings().respond(rate_response).await?;"#)
        }
        "List prompts" => Some(r#"let prompts = client.prompts().list().await?;"#),
        "Evaluate answer" => {
            Some(r#"let review = client.prompts().evaluate_answer(payload).await?;"#)
        }
        "Create prompt poll" => {
            Some(r#"let created = client.prompts().create_prompt_poll(payload).await?;"#)
        }
        "Create video prompt" => {
            Some(r#"let created = client.prompts().create_video_prompt(payload).await?;"#)
        }
        "List connections" => Some(r#"let matches = client.connections().list().await?;"#),
        "Get connection detail" => {
            Some(r#"let detail = client.connections().detail("subject_id").await?;"#)
        }
        "Get match note" => {
            Some(r#"let note = client.connections().match_note("subject_id").await?;"#)
        }
        "Get standouts" => Some(r#"let standouts = client.connections().standouts().await?;"#),
        "Get content settings" => Some(r#"let settings = client.settings().content().await?;"#),
        "Update content settings" => {
            Some(r#"let response = client.settings().update_content(settings).await?;"#)
        }
        "Update answers" => {
            Some(r#"let response = client.settings().update_answers(answers).await?;"#)
        }
        "Get auth settings" => Some(r#"let settings = client.settings().auth().await?;"#),
        "Get notification settings" => {
            Some(r#"let settings = client.settings().notifications().await?;"#)
        }
        "Get user traits" => Some(r#"let traits = client.settings().user_traits().await?;"#),
        "Get account info" => Some(r#"let account = client.settings().account_info().await?;"#),
        "Get export status" => Some(r#"let status = client.settings().export_status().await?;"#),
        "Repeat profiles" => {
            Some(r#"let repeated = client.recommendations().repeat_profiles().await?;"#)
        }
        "Authenticate Sendbird" => Some(r#"let credentials = client.chat().credentials().await?;"#),
        "Review message text" => Some(
            r#"let raw = client
    .raw()
    .hinge(
        reqwest::Method::POST,
        "/flag/textreview",
        Some(serde_json::json!({
            "text": "hey",
            "receiverId": "user_id"
        })),
    )
    .await?;"#,
        ),
        "Send Hinge message" => Some(r#"let sent = client.chat().send_message(payload).await?;"#),
        "List my Sendbird channels" => Some(r#"let channels = client.chat().channels(30).await?;"#),
        "Create Sendbird DM channel" => Some(
            r#"let channel = client
    .chat()
    .get_or_create_dm_channel("self_user_id", "peer_user_id")
    .await?;"#,
        ),
        "Get Sendbird channel" => Some(
            r#"let channel = client
    .chat()
    .channel("sendbird_group_channel")
    .await?;"#,
        ),
        "Get Sendbird messages" => Some(
            r#"let messages = client
    .chat()
    .full_messages("sendbird_group_channel")
    .await?;"#,
        ),
        "Export chat helper" => Some(
            r#"let result = client
    .chat()
    .export_chat(hinge_rs::models::ExportChatInput {
        channel_url: "sendbird_group_channel".to_string(),
        output_dir: "exports/chat".to_string(),
        include_media: true,
        initiation_summary_lines: None,
    })
    .await?;"#,
        ),
        "Connect Sendbird WebSocket" => Some(
            r#"let mut events = client.chat().subscribe_events().await?;
let event = events.recv().await?;
println!("{event:?}");"#,
        ),
        _ => None,
    }
}

fn operation_id(summary: &str) -> String {
    let mut words = summary
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty());

    let Some(first_word) = words.next() else {
        return "operation".to_string();
    };

    let mut operation_id = first_word.to_ascii_lowercase();
    for word in words {
        let word = word.to_ascii_lowercase();
        let mut chars = word.chars();
        if let Some(first_character) = chars.next() {
            operation_id.extend(first_character.to_uppercase());
            operation_id.push_str(chars.as_str());
        }
    }
    operation_id
}

fn schema_ref(name: &str) -> Value {
    json!({"$ref": format!("#/components/schemas/{name}")})
}

fn error_response(description: &str) -> Value {
    json!({
        "description": description,
        "content": {
            "application/json": {
                "schema": schema_ref("Error"),
                "examples": {
                    "error": {
                        "value": {"error": description}
                    }
                }
            }
        }
    })
}

fn query_param(name: &str, description: &str, required: bool) -> Value {
    json!({
        "name": name,
        "in": "query",
        "required": required,
        "description": description,
        "schema": {"type": "string"}
    })
}

fn path_param(name: &str, description: &str) -> Value {
    json!({
        "name": name,
        "in": "path",
        "required": true,
        "description": description,
        "schema": {"type": "string"}
    })
}

fn object_schema(fields: Vec<(&str, &str)>) -> Value {
    let properties = fields
        .into_iter()
        .map(|(name, typ)| {
            let schema = match typ {
                "integer" => json!({"type": "integer"}),
                "boolean" => json!({"type": "boolean"}),
                "object" => json!({"type": "object", "additionalProperties": true}),
                _ => json!({"type": "string"}),
            };
            (name.to_string(), schema)
        })
        .collect::<serde_json::Map<_, _>>();
    json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": true
    })
}

fn generic_object(description: &str) -> Value {
    json!({
        "description": description,
        "type": "object",
        "additionalProperties": true
    })
}

fn request_example(summary: &str) -> Option<Value> {
    match summary {
        "Register device install" => {
            Some(json!({"installId": "00000000-0000-0000-0000-000000000000"}))
        }
        "Initiate SMS login" => Some(
            json!({"deviceId": "00000000-0000-0000-0000-000000000000", "phoneNumber": "+15555550123"}),
        ),
        "Submit SMS OTP" => Some(
            json!({"installId": "00000000-0000-0000-0000-000000000000", "deviceId": "00000000-0000-0000-0000-000000000000", "phoneNumber": "+15555550123", "otp": "123456"}),
        ),
        "Get recommendations" => {
            Some(json!({"playerId": "user_123", "newHere": false, "activeToday": false}))
        }
        "Authenticate Sendbird" => Some(json!({"refresh": false})),
        "Review message text" => Some(json!({"text": "hey", "receiverId": "user_456"})),
        "Send Hinge message" => Some(json!({
            "ays": false,
            "matchMessage": true,
            "messageType": "text",
            "messageData": {"message": "hello"},
            "subjectId": "user_456",
            "origin": "connection"
        })),
        _ => None,
    }
}

fn example_for(summary: &str) -> Option<Value> {
    match summary {
        "Submit SMS OTP" => Some(json!({
            "hingeAuthToken": {
                "identityId": "user_123",
                "token": "[redacted]",
                "expires": "2026-04-26T00:00:00Z"
            },
            "sendbirdAuthToken": {
                "token": "[redacted]",
                "expires": "2026-04-26T00:00:00Z"
            }
        })),
        "Get like limit" => Some(json!({"likes": 8, "superlikes": 1})),
        "Get match note" => Some(json!({"note": "Liked your prompt"})),
        "Authenticate Sendbird" => {
            Some(json!({"token": "[redacted]", "expires": "2026-04-26T00:00:00Z"}))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_contains_scalar_environment_and_chat_paths() {
        let spec = openapi_document();
        assert!(spec.get("x-scalar-environments").is_some());
        assert!(spec["info"].get("x-scalar-sdk-installation").is_some());
        assert!(spec["paths"].get("/message/send").is_some());
        assert!(spec["paths"].get("/flag/textreview").is_some());
        assert_eq!(
            spec["paths"]["/identity/install"]["post"]["security"],
            json!([])
        );
        assert_eq!(
            spec["paths"]["/rec/v2"]["post"]["operationId"],
            "getRecommendations"
        );
        assert!(
            spec["paths"]
                .get("/users/{userId}/my_group_channels")
                .is_some()
        );
        assert!(
            spec["paths"]
                .get("/sdk/group_channels/{channelUrl}")
                .is_some()
        );
        assert!(spec["paths"].get("/sendbird/ws").is_some());
        assert!(
            spec["components"]["securitySchemes"]
                .get("sendbirdSessionKey")
                .is_some()
        );
    }

    #[test]
    fn scalar_html_references_local_openapi() {
        let html = scalar_index_html(&ScalarPageConfig::latest(
            hinge_rs::VERSION,
            "./",
            "./versions.json",
        ))
        .expect("scalar html should render");
        assert!(html.contains("@scalar/api-reference"));
        assert!(html.contains("url: './openapi.json'"));
        assert!(html.contains("version-select"));
        assert!(html.contains("./versions.json"));
    }

    #[test]
    fn scalar_html_can_reference_parent_versions() {
        let html = scalar_index_html(&ScalarPageConfig {
            current_version: "0.1.0".to_string(),
            latest_version: "0.1.0".to_string(),
            site_root: "../../".to_string(),
            versions_url: "../../versions.json".to_string(),
        })
        .expect("scalar html should render");
        assert!(html.contains("const currentVersion = \"0.1.0\";"));
        assert!(html.contains("const siteRoot = \"../../\";"));
        assert!(html.contains("const versionsUrl = \"../../versions.json\";"));
    }

    #[test]
    fn versions_json_lists_release_versions() {
        let json = versions_json("0.2.0", &["0.2.0".to_string(), "0.1.0".to_string()])
            .expect("versions json should render");
        let value: Value = serde_json::from_str(&json).expect("versions json should parse");
        assert_eq!(value["latest"], "0.2.0");
        assert_eq!(value["versions"][0], "0.2.0");
        assert_eq!(value["versions"][1], "0.1.0");
    }

    #[test]
    fn generated_openapi_artifact_is_stable() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask should live under crate root")
            .to_path_buf();
        let artifact_path = crate_root.join("openapi/hinge-api.openapi.json");
        let expected = format!(
            "{}\n",
            serde_json::to_string_pretty(&openapi_document()).expect("spec should serialize")
        );
        let actual = fs::read_to_string(artifact_path).expect("generated spec should be readable");
        assert_eq!(actual, expected);
    }

    #[test]
    fn generated_openapi_uses_scalar_renderable_response_schemas() {
        let spec = openapi_document();

        assert!(
            !has_defs_key(&spec),
            "OpenAPI components should not keep $defs"
        );
        assert!(
            !has_defs_ref(&spec),
            "OpenAPI refs should not point into nested $defs"
        );

        let rec_media =
            &spec["paths"]["/rec/v2"]["post"]["responses"]["200"]["content"]["application/json"];
        assert_eq!(
            rec_media["schema"]["$ref"],
            "#/components/schemas/RecommendationsResponse"
        );
        assert!(
            rec_media.get("examples").is_none(),
            "unknown examples should be omitted instead of showing empty objects"
        );
        assert_eq!(
            spec["components"]["schemas"]["RecommendationsResponse"]["properties"]["feeds"]["items"]
                ["$ref"],
            "#/components/schemas/RecommendationsFeed"
        );
    }

    #[test]
    fn generated_versions_artifact_is_stable() {
        let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask should live under crate root")
            .to_path_buf();
        let artifact_path = crate_root.join("docs/api/versions.json");
        let expected =
            versions_json(hinge_rs::VERSION, &[]).expect("versions json should serialize");
        let actual =
            fs::read_to_string(artifact_path).expect("generated versions should be readable");
        assert_eq!(actual, expected);
    }

    #[test]
    fn every_operation_has_operation_id_and_rust_samples() {
        let spec = openapi_document();
        let mut operations = 0usize;
        let mut operations_with_samples = 0usize;
        for path in spec["paths"].as_object().expect("paths object").values() {
            for operation in path.as_object().expect("path item").values() {
                operations += 1;
                assert!(
                    operation.get("operationId").is_some(),
                    "operation missing operationId: {operation:#}"
                );
                if operation
                    .get("x-codeSamples")
                    .and_then(|samples| samples.as_array())
                    .is_some_and(|samples| !samples.is_empty())
                {
                    operations_with_samples += 1;
                }
            }
        }
        assert_eq!(operations, 44);
        assert_eq!(operations_with_samples, 42);
    }

    fn has_defs_key(value: &Value) -> bool {
        match value {
            Value::Object(map) => map.contains_key("$defs") || map.values().any(has_defs_key),
            Value::Array(values) => values.iter().any(has_defs_key),
            _ => false,
        }
    }

    fn has_defs_ref(value: &Value) -> bool {
        match value {
            Value::Object(map) => {
                if let Some(Value::String(reference)) = map.get("$ref")
                    && reference.contains("/$defs/")
                {
                    return true;
                }
                map.values().any(has_defs_ref)
            }
            Value::Array(values) => values.iter().any(has_defs_ref),
            _ => false,
        }
    }
}
