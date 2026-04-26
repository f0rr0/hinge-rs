use super::HingeClient;
use crate::errors::HingeError;
use crate::models::SendMessagePayload;
use crate::storage::Storage;
use uuid::Uuid;

impl<S: Storage + Clone> HingeClient<S> {
    pub async fn send_message(
        &self,
        mut payload: SendMessagePayload,
    ) -> Result<serde_json::Value, HingeError> {
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

        if payload.dedup_id.is_none() {
            payload.dedup_id = Some(Uuid::new_v4().to_string().to_uppercase());
        }

        let url = format!("{}/message/send", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }
}
