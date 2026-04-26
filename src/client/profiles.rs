use super::HingeClient;
use super::payload::{preferences_to_api_json, profile_update_to_api_json};
use super::render::render_profile;
use crate::errors::HingeError;
use crate::models::{
    AnswerContentPayload, Preferences, PreferencesResponse, ProfileContentFull, ProfileUpdate,
    PublicUserProfile, SelfContentResponse, SelfProfileResponse,
};
use crate::storage::Storage;
use std::collections::HashSet;

impl<S: Storage + Clone> HingeClient<S> {
    pub async fn rendered_profile_text_for_user(
        &mut self,
        user_id: &str,
    ) -> Result<String, HingeError> {
        let uid = user_id.trim();
        if uid.is_empty() {
            return Ok(String::new());
        }

        let prompts_manager = match self.fetch_prompts_manager().await {
            Ok(mgr) => Some(mgr),
            Err(err) => {
                log::warn!("Failed to prefetch prompts for rendered profile: {}", err);
                None
            }
        };
        let profile = self
            .get_profiles(vec![uid.to_string()])
            .await?
            .into_iter()
            .next();
        let profile_content = self
            .get_profile_content(vec![uid.to_string()])
            .await?
            .into_iter()
            .next();

        Ok(render_profile(
            profile.as_ref(),
            profile_content.as_ref(),
            prompts_manager.as_ref(),
        ))
    }

    pub async fn get_self_profile(&self) -> Result<SelfProfileResponse, HingeError> {
        let url = format!("{}/user/v3", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response::<SelfProfileResponse>(res).await
    }

    pub async fn get_self_content(&self) -> Result<SelfContentResponse, HingeError> {
        let url = format!("{}/content/v2", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response::<SelfContentResponse>(res).await
    }

    pub async fn get_self_preferences(&self) -> Result<PreferencesResponse, HingeError> {
        let url = format!("{}/preference/v2/selected", self.settings.base_url);
        let res = self.http_get(&url).await?;
        self.parse_response::<PreferencesResponse>(res).await
    }

    pub async fn get_profiles_public_raw_unfiltered(
        &self,
        ids: Vec<String>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!(
            "{}/user/v3/public?ids={}",
            self.settings.base_url,
            ids.join(",")
        );
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_content_public_raw_unfiltered(
        &self,
        ids: Vec<String>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!(
            "{}/content/v2/public?ids={}",
            self.settings.base_url,
            ids.join(",")
        );
        let res = self.http_get(&url).await?;
        self.parse_response(res).await
    }

    pub async fn get_profiles(
        &self,
        user_ids: Vec<String>,
    ) -> Result<Vec<PublicUserProfile>, HingeError> {
        let chunks = self.prepare_user_id_chunks(user_ids);
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        let mut aggregated: Vec<PublicUserProfile> = Vec::new();
        for batch in chunks {
            let url = format!(
                "{}/user/v3/public?ids={}",
                self.settings.base_url,
                batch.join(",")
            );
            let res = self.http_get(&url).await?;
            let mut part: Vec<PublicUserProfile> = self.parse_response(res).await?;
            aggregated.append(&mut part);
        }
        Ok(aggregated)
    }

    pub async fn get_profile_content(
        &self,
        user_ids: Vec<String>,
    ) -> Result<Vec<ProfileContentFull>, HingeError> {
        let chunks = self.prepare_user_id_chunks(user_ids);
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        let mut aggregated: Vec<ProfileContentFull> = Vec::new();
        for batch in chunks {
            let url = format!(
                "{}/content/v2/public?ids={}",
                self.settings.base_url,
                batch.join(",")
            );
            let res = self.http_get(&url).await?;
            let mut part: Vec<ProfileContentFull> = self.parse_response(res).await?;
            aggregated.append(&mut part);
        }
        Ok(aggregated)
    }

    pub async fn update_self_preferences(
        &self,
        preferences: Preferences,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/preference/v2/selected", self.settings.base_url);
        let prefs_json = preferences_to_api_json(&preferences);
        let payload = serde_json::json!([prefs_json]);
        let res = self.http_patch(&url, &payload).await?;
        self.parse_response(res).await
    }

    pub async fn update_self_profile(
        &self,
        profile_updates: ProfileUpdate,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/user/v3", self.settings.base_url);
        let profile_json = profile_update_to_api_json(&profile_updates);
        let payload = serde_json::json!({ "profile": profile_json });
        let res = self.http_patch(&url, &payload).await?;
        self.parse_response(res).await
    }

    pub async fn update_answers(
        &self,
        answers: Vec<AnswerContentPayload>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/content/v1/answers", self.settings.base_url);
        let res = self
            .http
            .put(url)
            .headers(self.default_headers()?)
            .json(&answers)
            .send()
            .await?;
        self.parse_response(res).await
    }

    pub async fn delete_content(&self, content_ids: Vec<String>) -> Result<(), HingeError> {
        let url = format!(
            "{}/content/v1?ids={}",
            self.settings.base_url,
            content_ids.join(",")
        );
        let res = self
            .http
            .delete(url)
            .headers(self.default_headers()?)
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        Ok(())
    }

    fn prepare_user_id_chunks(&self, user_ids: Vec<String>) -> Vec<Vec<String>> {
        fn is_user_id_like(id: &str) -> bool {
            if id.is_empty() {
                return false;
            }
            let trimmed = id.trim();
            if trimmed.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
            trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_hexdigit())
        }

        let (mut accepted, mut dropped) = (Vec::new(), 0usize);
        let mut seen: HashSet<String> = HashSet::new();
        for raw in user_ids {
            let id = raw.trim().to_string();
            if is_user_id_like(&id) && seen.insert(id.clone()) {
                accepted.push(id);
            } else {
                dropped += 1;
            }
        }

        if accepted.is_empty() {
            log::warn!("No valid user IDs to fetch (dropped {})", dropped);
            return Vec::new();
        }
        if dropped > 0 {
            log::debug!("Dropped {} non user-like IDs from public fetch", dropped);
        }

        let batch_size = self.public_ids_batch_size.max(1);
        let mut out: Vec<Vec<String>> = Vec::new();
        let mut idx = 0usize;
        while idx < accepted.len() {
            let end = (idx + batch_size).min(accepted.len());
            out.push(accepted[idx..end].to_vec());
            idx = end;
        }
        if out.len() > 1 {
            log::info!(
                "Fetching public user data in {} batches of up to {} IDs",
                out.len(),
                batch_size
            );
        }
        out
    }
}
