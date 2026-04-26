use super::HingeClient;
use crate::errors::HingeError;
use crate::models::{LikeItemV2, LikeLimit, LikesV2Response, MatchNoteResponse};
use crate::storage::Storage;

impl<S: Storage + Clone> HingeClient<S> {
    pub async fn get_like_limit(&self) -> Result<LikeLimit, HingeError> {
        let url = format!("{}/likelimit", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_likes_v2(&self) -> Result<LikesV2Response, HingeError> {
        let url = format!("{}/like/v2", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_like_subject(&self, subject_id: &str) -> Result<LikeItemV2, HingeError> {
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

    pub async fn get_likes_v2_raw(&self) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/like/v2", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }
}
