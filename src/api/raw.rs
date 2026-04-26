use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::storage::Storage;

pub struct RawApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> RawApi<'_, S> {
    pub async fn hinge(
        &self,
        method: reqwest::Method,
        path_or_url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, HingeError> {
        self.client.raw_hinge_json(method, path_or_url, body).await
    }

    pub async fn sendbird(
        &self,
        method: reqwest::Method,
        path_or_url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, HingeError> {
        self.client
            .raw_sendbird_json(method, path_or_url, body)
            .await
    }
}
