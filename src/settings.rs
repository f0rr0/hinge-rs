#[derive(Clone, Debug)]
pub struct Settings {
    pub base_url: String,
    pub sendbird_app_id: String,
    pub sendbird_api_url: String,
    pub sendbird_ws_url: String,
    pub sendbird_sdk_version: String,
    pub hinge_app_version: String,
    pub hinge_build_number: String,
    pub os_version: String,
    /// `x-device-region`. Hardcoded to `IN` before this was configurable, which is wrong for
    /// most accounts; kept as the default only to avoid changing behaviour silently.
    pub device_region: String,
    /// `accept-language`. Same story as `device_region`, previously fixed at `en-GB`.
    pub accept_language: String,
}

impl Default for Settings {
    fn default() -> Self {
        let app_id = std::env::var("SENDBIRD_APP_ID")
            .unwrap_or_else(|_| "3CDAD91C-1E0D-4A0D-BBEE-9671988BF9E9".into());
        let lower = app_id.to_lowercase();
        Self {
            base_url: std::env::var("BASE_URL")
                .unwrap_or_else(|_| "https://prod-api.hingeaws.net".into()),
            sendbird_app_id: app_id.clone(),
            sendbird_api_url: format!("https://api-{}.sendbird.com", lower),
            sendbird_ws_url: format!("wss://ws-{}.sendbird.com", lower),
            sendbird_sdk_version: std::env::var("SENDBIRD_SDK_VERSION")
                .unwrap_or_else(|_| "4.26.0".into()),
            // 9.91.0 was retired server-side and every call answers
            // `426 Upgrade Required`, so the default has to track the shipping app. The build
            // number stays at the 9.91.0 value because Hinge does not publish it, and it is
            // demonstrably not checked against the version: requests are accepted with 10.0.0
            // paired with this build, and sweeping it across six values changed no response.
            hinge_app_version: std::env::var("HINGE_APP_VERSION")
                .unwrap_or_else(|_| "10.0.0".into()),
            hinge_build_number: std::env::var("HINGE_BUILD_NUMBER")
                .unwrap_or_else(|_| "11639".into()),
            os_version: std::env::var("OS_VERSION").unwrap_or_else(|_| "26.0".into()),
            device_region: std::env::var("HINGE_DEVICE_REGION").unwrap_or_else(|_| "IN".into()),
            accept_language: std::env::var("HINGE_ACCEPT_LANGUAGE")
                .unwrap_or_else(|_| "en-GB".into()),
        }
    }
}
