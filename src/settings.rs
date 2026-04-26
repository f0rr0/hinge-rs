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
            hinge_app_version: std::env::var("HINGE_APP_VERSION")
                .unwrap_or_else(|_| "9.91.0".into()),
            hinge_build_number: std::env::var("HINGE_BUILD_NUMBER")
                .unwrap_or_else(|_| "11639".into()),
            os_version: std::env::var("OS_VERSION").unwrap_or_else(|_| "26.0".into()),
        }
    }
}
