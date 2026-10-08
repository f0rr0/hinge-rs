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

/// Public profile vitals, by id.
const PUBLIC_PROFILES_PATH: &str = "/user/v3/public";
/// Public profile content (photos, prompt answers), by id.
const PUBLIC_CONTENT_PATH: &str = "/content/v2/public";

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
        self.post_public_ids(PUBLIC_PROFILES_PATH, &ids, None).await
    }

    pub async fn get_content_public_raw_unfiltered(
        &self,
        ids: Vec<String>,
    ) -> Result<serde_json::Value, HingeError> {
        self.post_public_ids(PUBLIC_CONTENT_PATH, &ids, None).await
    }

    /// Ask a public endpoint for a set of ids.
    ///
    /// POST rather than GET: as of Hinge app 10.0.0 both public routes answer
    /// `405 Method Not Allowed` to the `GET ...?ids=` they used to serve. They accept
    /// `POST {"ids": ["<id>", ...]}`, and the ids must be JSON *strings* — a number, or a
    /// comma-joined string, is rejected with 400.
    ///
    /// Without a `viewToken` the server answers `412 Precondition Failed` for any non-empty
    /// id list — see [`HingeClient::get_profiles_with_view_token`] for where a token comes
    /// from and what it covers.
    async fn post_public_ids<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        ids: &[String],
        view_token: Option<&str>,
    ) -> Result<T, HingeError> {
        let url = format!("{}{}", self.settings.base_url, path);
        let res = self
            .http_post(&url, &public_ids_body(ids, view_token))
            .await?;
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
            let mut part: Vec<PublicUserProfile> = self
                .post_public_ids(PUBLIC_PROFILES_PATH, &batch, None)
                .await?;
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
            let mut part: Vec<ProfileContentFull> = self
                .post_public_ids(PUBLIC_CONTENT_PATH, &batch, None)
                .await?;
            aggregated.append(&mut part);
        }
        Ok(aggregated)
    }

    /// Fetch public profile vitals for one person, authorised by their `viewToken`.
    ///
    /// The public endpoints refuse an unauthorised read with `412 Precondition Failed`, no
    /// matter whose id is asked for — the caller's own included. The `viewToken` is what
    /// lifts it: proof that Hinge showed you this person. It is issued per subject by the
    /// single-subject reads, [`HingeClient::view_token_for_like`] and
    /// [`HingeClient::view_token_for_connection`], and it does not appear on the list reads.
    ///
    /// Scoped to its subject: a token minted for one person does not authorise another, so
    /// there is no batching to be had here even though the wire format takes a list. The
    /// `ids` list stays a list only because that is the shape the endpoint wants.
    ///
    /// Not everyone is reachable this way. A token exists for people who have liked you and
    /// for your matches; a recommendation from the discovery feed carries a `ratingToken`,
    /// which is a different thing and is refused in this slot. Those profiles cannot
    /// currently be read over REST at all.
    pub async fn get_profiles_with_view_token(
        &self,
        user_ids: Vec<String>,
        view_token: &str,
    ) -> Result<Vec<PublicUserProfile>, HingeError> {
        self.post_public_ids(PUBLIC_PROFILES_PATH, &user_ids, Some(view_token))
            .await
    }

    /// Fetch public profile content — photos and prompt answers — for one person.
    ///
    /// Same authorisation rules as [`Self::get_profiles_with_view_token`]: the vitals and the
    /// content live on separate routes, and one token opens both.
    pub async fn get_profile_content_with_view_token(
        &self,
        user_ids: Vec<String>,
        view_token: &str,
    ) -> Result<Vec<ProfileContentFull>, HingeError> {
        self.post_public_ids(PUBLIC_CONTENT_PATH, &user_ids, Some(view_token))
            .await
    }

    /// The `viewToken` for someone who has liked you.
    ///
    /// Reads `/like/subject/{id}`, which carries the token; the `/like/v2` list does not.
    pub async fn view_token_for_like(&self, subject_id: &str) -> Result<String, HingeError> {
        self.get_like_subject(subject_id)
            .await?
            .view_token
            .ok_or_else(|| HingeError::Http(format!("no viewToken on like subject {}", subject_id)))
    }

    /// The `viewToken` for one of your matches.
    ///
    /// Reads `/connection/subject/{id}`, which carries the token; `/connection/v2` does not.
    pub async fn view_token_for_connection(&self, subject_id: &str) -> Result<String, HingeError> {
        self.get_connection_detail(subject_id)
            .await?
            .connection
            .view_token
            .ok_or_else(|| {
                HingeError::Http(format!("no viewToken on connection subject {}", subject_id))
            })
    }

    /// Render one person's profile as text, authorised by their `viewToken`.
    ///
    /// The token-less [`Self::rendered_profile_text_for_user`] cannot work against the
    /// current API; this is the version that does.
    pub async fn rendered_profile_text_with_view_token(
        &mut self,
        user_id: &str,
        view_token: &str,
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
            .get_profiles_with_view_token(vec![uid.to_string()], view_token)
            .await?
            .into_iter()
            .next();
        let profile_content = self
            .get_profile_content_with_view_token(vec![uid.to_string()], view_token)
            .await?
            .into_iter()
            .next();

        Ok(render_profile(
            profile.as_ref(),
            profile_content.as_ref(),
            prompts_manager.as_ref(),
        ))
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

/// The request body both public endpoints expect: ids as JSON strings under `ids`, plus the
/// `viewToken` that authorises the read when one is available.
fn public_ids_body(ids: &[String], view_token: Option<&str>) -> serde_json::Value {
    match view_token {
        Some(token) => serde_json::json!({ "ids": ids, "viewToken": token }),
        None => serde_json::json!({ "ids": ids }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_sent_as_strings_under_ids() {
        let body = public_ids_body(&["1000000000000000001".to_string(), "42".to_string()], None);
        assert_eq!(
            body,
            serde_json::json!({ "ids": ["1000000000000000001", "42"] })
        );
        // A JSON number here is rejected by the server with 400, so the type matters.
        assert!(body["ids"][0].is_string());
    }

    #[test]
    fn an_empty_request_is_still_an_ids_array() {
        assert_eq!(public_ids_body(&[], None), serde_json::json!({ "ids": [] }));
    }

    #[test]
    fn a_view_token_rides_alongside_the_ids() {
        let body = public_ids_body(&["1000000000000000001".to_string()], Some("tok-abc"));
        assert_eq!(
            body,
            serde_json::json!({ "ids": ["1000000000000000001"], "viewToken": "tok-abc" })
        );
    }

    #[test]
    fn no_view_token_key_is_sent_when_there_is_none() {
        // The server rejects a non-empty request without it, but an empty probe still has to
        // go out clean rather than carrying a null.
        let body = public_ids_body(&["42".to_string()], None);
        assert!(body.get("viewToken").is_none());
    }

    #[test]
    fn public_paths_are_the_ones_the_server_still_routes() {
        assert_eq!(PUBLIC_PROFILES_PATH, "/user/v3/public");
        assert_eq!(PUBLIC_CONTENT_PATH, "/content/v2/public");
        // /user/v2/public answers 404 - it is not a fallback.
        assert_ne!(PUBLIC_PROFILES_PATH, "/user/v2/public");
    }
}
