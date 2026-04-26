use super::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    CreateRate, CreateRateContent, CreateRateContentPrompt, LikeResponse, PhotoAsset,
    PhotoAssetInput, RateInput, RateRespondRequest, RateRespondResponse, SkipInput,
};
use crate::storage::Storage;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

impl<S: Storage + Clone> HingeClient<S> {
    pub async fn skip(&mut self, input: SkipInput) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/rate/v2/initiate", self.settings.base_url);
        let payload = CreateRate {
            rating_id: Uuid::new_v4().to_string().to_uppercase(),
            hcm_run_id: None,
            session_id: self.session_id.clone(),
            content: None,
            created: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            rating_token: input.rating_token,
            initiated_with: None,
            rating: "skip".into(),
            has_pairing: false,
            origin: Some(input.origin.unwrap_or_else(|| "compatibles".into())),
            subject_id: input.subject_id.clone(),
        };
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        let body = self.parse_response(res).await?;

        self.remove_recommendation(&input.subject_id);
        if self.auto_persist
            && let Some(path) = self.recs_cache_path()
        {
            let _ = self.save_recommendations(&path);
        }
        Ok(body)
    }

    pub async fn rate_user(&mut self, input: RateInput) -> Result<LikeResponse, HingeError> {
        let mut hcm_run_id: Option<String> = None;
        if let Some(text) = input.comment.as_deref() {
            let run_id = self.run_text_review(text, &input.subject_id).await?;
            hcm_run_id = Some(run_id);
        }
        let prompt_answer = input.answer_text.clone().unwrap_or_default();
        let prompt_question = input.question_text.clone().unwrap_or_default();
        let prompt_content_id = input.content_id.clone();

        let content = if let Some(photo) = input.photo {
            let PhotoAssetInput {
                url,
                content_id,
                cdn_id,
                bounding_box,
                selfie_verified,
            } = photo;
            Some(CreateRateContent {
                comment: input.comment.clone(),
                photo: Some(PhotoAsset {
                    id: None,
                    url,
                    cdn_id,
                    content_id,
                    prompt_id: None,
                    caption: None,
                    width: None,
                    height: None,
                    video_url: None,
                    selfie_verified,
                    bounding_box,
                    location: None,
                    source: None,
                    source_id: None,
                    p_hash: None,
                }),
                prompt: None,
            })
        } else {
            let prompt = CreateRateContentPrompt {
                answer: prompt_answer,
                content_id: prompt_content_id,
                question: prompt_question,
            };
            Some(CreateRateContent {
                comment: input.comment.clone(),
                photo: None,
                prompt: Some(prompt),
            })
        };
        let payload = CreateRate {
            rating_id: Uuid::new_v4().to_string().to_uppercase(),
            hcm_run_id,
            session_id: self.session_id.clone(),
            content,
            created: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            rating_token: input.rating_token,
            initiated_with: Some(if input.use_superlike.unwrap_or(false) {
                "superlike".into()
            } else {
                "standard".into()
            }),
            rating: if input.comment.is_some() {
                "note".into()
            } else {
                "like".into()
            },
            has_pairing: false,
            origin: Some(input.origin.unwrap_or_else(|| "compatibles".into())),
            subject_id: input.subject_id,
        };
        let url = format!("{}/rate/v2/initiate", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        let body = self.parse_response::<LikeResponse>(res).await?;
        if self.auto_persist
            && let Some(path) = self.recs_cache_path()
        {
            let _ = self.save_recommendations(&path);
        }
        Ok(body)
    }

    pub async fn respond_rate(
        &self,
        mut payload: RateRespondRequest,
    ) -> Result<RateRespondResponse, HingeError> {
        if payload.rating_id.is_none() {
            payload.rating_id = Some(Uuid::new_v4().to_string().to_uppercase());
        }
        if payload.session_id.is_none() {
            payload.session_id = Some(self.session_id.clone());
        }
        if payload.created.is_none() {
            payload.created = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
        }

        let url = format!("{}/rate/v2/respond", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }

    async fn run_text_review(&self, text: &str, receiver_id: &str) -> Result<String, HingeError> {
        let url = format!("{}/flag/textreview", self.settings.base_url);
        let res = self
            .http
            .post(url)
            .headers(self.default_headers()?)
            .json(&json!({ "text": text, "receiverId": receiver_id }))
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let v = self.parse_response::<serde_json::Value>(res).await?;
        Ok(v.get("hcmRunId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string())
    }
}
