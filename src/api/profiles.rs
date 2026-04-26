use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    ProfileContentFull, ProfileUpdate, PublicUserProfile, SelfContentResponse, SelfProfileResponse,
};
use crate::storage::Storage;

pub struct ProfilesApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> ProfilesApi<'_, S> {
    pub async fn rendered_text_for_user(&mut self, user_id: &str) -> Result<String, HingeError> {
        self.client.rendered_profile_text_for_user(user_id).await
    }

    pub async fn me(&self) -> Result<SelfProfileResponse, HingeError> {
        self.client.get_self_profile().await
    }

    pub async fn content(&self) -> Result<SelfContentResponse, HingeError> {
        self.client.get_self_content().await
    }

    pub async fn public(
        &self,
        user_ids: Vec<String>,
    ) -> Result<Vec<PublicUserProfile>, HingeError> {
        self.client.get_profiles(user_ids).await
    }

    pub async fn public_raw_unfiltered(
        &self,
        user_ids: Vec<String>,
    ) -> Result<serde_json::Value, HingeError> {
        self.client
            .get_profiles_public_raw_unfiltered(user_ids)
            .await
    }

    pub async fn public_content(
        &self,
        user_ids: Vec<String>,
    ) -> Result<Vec<ProfileContentFull>, HingeError> {
        self.client.get_profile_content(user_ids).await
    }

    pub async fn public_content_raw_unfiltered(
        &self,
        user_ids: Vec<String>,
    ) -> Result<serde_json::Value, HingeError> {
        self.client
            .get_content_public_raw_unfiltered(user_ids)
            .await
    }

    pub async fn update(&self, update: ProfileUpdate) -> Result<serde_json::Value, HingeError> {
        self.client.update_self_profile(update).await
    }

    pub async fn delete_content(&self, content_ids: Vec<String>) -> Result<(), HingeError> {
        self.client.delete_content(content_ids).await
    }
}
