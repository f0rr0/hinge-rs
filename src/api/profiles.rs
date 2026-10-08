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

    /// Public vitals for one person, authorised by their `viewToken`.
    ///
    /// The token-less [`Self::public`] is refused with `412` by the current API; this is the
    /// call that returns data. See
    /// [`HingeClient::get_profiles_with_view_token`](crate::client::HingeClient::get_profiles_with_view_token).
    pub async fn public_with_view_token(
        &self,
        user_ids: Vec<String>,
        view_token: &str,
    ) -> Result<Vec<PublicUserProfile>, HingeError> {
        self.client
            .get_profiles_with_view_token(user_ids, view_token)
            .await
    }

    /// Public content — photos and prompt answers — for one person, by `viewToken`.
    pub async fn public_content_with_view_token(
        &self,
        user_ids: Vec<String>,
        view_token: &str,
    ) -> Result<Vec<ProfileContentFull>, HingeError> {
        self.client
            .get_profile_content_with_view_token(user_ids, view_token)
            .await
    }

    /// The `viewToken` for someone who liked you.
    pub async fn view_token_for_like(&self, subject_id: &str) -> Result<String, HingeError> {
        self.client.view_token_for_like(subject_id).await
    }

    /// The `viewToken` for one of your matches.
    pub async fn view_token_for_connection(&self, subject_id: &str) -> Result<String, HingeError> {
        self.client.view_token_for_connection(subject_id).await
    }

    /// Everything about someone who liked you: vitals and content, token resolved for you.
    pub async fn for_like(
        &mut self,
        subject_id: &str,
    ) -> Result<(Option<PublicUserProfile>, Option<ProfileContentFull>), HingeError> {
        let token = self.client.view_token_for_like(subject_id).await?;
        self.fetch_pair(subject_id, &token).await
    }

    /// Everything about one of your matches: vitals and content, token resolved for you.
    pub async fn for_match(
        &mut self,
        subject_id: &str,
    ) -> Result<(Option<PublicUserProfile>, Option<ProfileContentFull>), HingeError> {
        let token = self.client.view_token_for_connection(subject_id).await?;
        self.fetch_pair(subject_id, &token).await
    }

    /// Rendered profile text, authorised by a `viewToken`.
    pub async fn rendered_text_with_view_token(
        &mut self,
        user_id: &str,
        view_token: &str,
    ) -> Result<String, HingeError> {
        self.client
            .rendered_profile_text_with_view_token(user_id, view_token)
            .await
    }

    async fn fetch_pair(
        &mut self,
        subject_id: &str,
        view_token: &str,
    ) -> Result<(Option<PublicUserProfile>, Option<ProfileContentFull>), HingeError> {
        let ids = vec![subject_id.to_string()];
        let vitals = self
            .client
            .get_profiles_with_view_token(ids.clone(), view_token)
            .await?
            .into_iter()
            .next();
        let content = self
            .client
            .get_profile_content_with_view_token(ids, view_token)
            .await?
            .into_iter()
            .next();
        Ok((vitals, content))
    }

    pub async fn update(&self, update: ProfileUpdate) -> Result<serde_json::Value, HingeError> {
        self.client.update_self_profile(update).await
    }

    pub async fn delete_content(&self, content_ids: Vec<String>) -> Result<(), HingeError> {
        self.client.delete_content(content_ids).await
    }
}
