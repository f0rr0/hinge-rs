use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    AccountInfo, AnswerContentPayload, AuthSettings, ExportStatus, NotificationSettings,
    Preferences, PreferencesResponse, UserSettings, UserTrait,
};
use crate::storage::Storage;

pub struct SettingsApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> SettingsApi<'_, S> {
    pub async fn preferences(&self) -> Result<PreferencesResponse, HingeError> {
        self.client.get_self_preferences().await
    }

    pub async fn update_preferences(
        &self,
        preferences: Preferences,
    ) -> Result<serde_json::Value, HingeError> {
        self.client.update_self_preferences(preferences).await
    }

    pub async fn content(&self) -> Result<UserSettings, HingeError> {
        self.client.get_content_settings().await
    }

    pub async fn update_content(
        &self,
        settings: UserSettings,
    ) -> Result<serde_json::Value, HingeError> {
        self.client.update_content_settings(settings).await
    }

    pub async fn update_answers(
        &self,
        answers: Vec<AnswerContentPayload>,
    ) -> Result<serde_json::Value, HingeError> {
        self.client.update_answers(answers).await
    }

    pub async fn auth(&self) -> Result<AuthSettings, HingeError> {
        self.client.get_auth_settings().await
    }

    pub async fn notifications(&self) -> Result<NotificationSettings, HingeError> {
        self.client.get_notification_settings().await
    }

    pub async fn user_traits(&self) -> Result<Vec<UserTrait>, HingeError> {
        self.client.get_user_traits().await
    }

    pub async fn account_info(&self) -> Result<AccountInfo, HingeError> {
        self.client.get_account_info().await
    }

    pub async fn export_status(&self) -> Result<ExportStatus, HingeError> {
        self.client.get_export_status().await
    }
}
