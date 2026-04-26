use super::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    AccountInfo, AuthSettings, ExportStatus, NotificationSettings, UserSettings, UserTrait,
};
use crate::storage::Storage;

impl<S: Storage + Clone> HingeClient<S> {
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
        self.parse_response(res).await
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
}
