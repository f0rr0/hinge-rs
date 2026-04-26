use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::{LikeResponse, RateInput, RateRespondRequest, RateRespondResponse, SkipInput};
use crate::storage::Storage;

pub struct RatingsApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> RatingsApi<'_, S> {
    pub async fn skip(&mut self, input: SkipInput) -> Result<serde_json::Value, HingeError> {
        self.client.skip(input).await
    }

    pub async fn rate_user(&mut self, input: RateInput) -> Result<LikeResponse, HingeError> {
        self.client.rate_user(input).await
    }

    pub async fn respond(
        &self,
        input: RateRespondRequest,
    ) -> Result<RateRespondResponse, HingeError> {
        self.client.respond_rate(input).await
    }
}
