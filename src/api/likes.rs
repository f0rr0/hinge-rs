use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::{LikeItemV2, LikeLimit, LikesV2Response, MatchNoteResponse};
use crate::storage::Storage;

pub struct LikesApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> LikesApi<'_, S> {
    pub async fn limit(&self) -> Result<LikeLimit, HingeError> {
        self.client.get_like_limit().await
    }

    pub async fn list(&self) -> Result<LikesV2Response, HingeError> {
        self.client.get_likes_v2().await
    }

    pub async fn list_raw(&self) -> Result<serde_json::Value, HingeError> {
        self.client.get_likes_v2_raw().await
    }

    pub async fn subject(&self, subject_id: &str) -> Result<LikeItemV2, HingeError> {
        self.client.get_like_subject(subject_id).await
    }

    pub async fn match_note(&self, subject_id: &str) -> Result<MatchNoteResponse, HingeError> {
        self.client.get_match_note(subject_id).await
    }
}
