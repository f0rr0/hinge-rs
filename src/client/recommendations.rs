use super::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    RecommendationSubject, RecommendationsFeed, RecommendationsResponse, RecsV2Params,
};
use crate::storage::Storage;
use reqwest::StatusCode;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tokio::time::sleep;

impl<S: Storage + Clone> HingeClient<S> {
    pub async fn get_recommendations(&mut self) -> Result<RecommendationsResponse, HingeError> {
        self.get_recommendations_v2_params(RecsV2Params {
            new_here: false,
            active_today: false,
        })
        .await
    }

    pub async fn get_recommendations_v2_params(
        &mut self,
        params: RecsV2Params,
    ) -> Result<RecommendationsResponse, HingeError> {
        let url = format!("{}/rec/v2", self.settings.base_url);
        let identity_id = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?
            .identity_id
            .clone();

        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body {
            player_id: String,
            new_here: bool,
            active_today: bool,
        }

        let body = Body {
            player_id: identity_id,
            new_here: params.new_here,
            active_today: params.active_today,
        };
        let body_json =
            serde_json::to_value(&body).map_err(|e| HingeError::Serde(e.to_string()))?;

        let fetch_count = self.recs_fetch_config.multi_fetch_count.max(1);
        let min_delay = Duration::from_millis(self.recs_fetch_config.request_delay_ms);
        let mut aggregated: Option<RecommendationsResponse> = None;
        let mut completed_calls = 0usize;
        let mut rate_limit_attempts = 0usize;
        let max_rate_limit_retries = self.recs_fetch_config.rate_limit_retries;
        let base_backoff_ms = self.recs_fetch_config.rate_limit_backoff_ms.max(1);

        while completed_calls < fetch_count {
            if let Some(last_call) = self.last_recs_v2_call {
                let elapsed = last_call.elapsed();
                if elapsed < min_delay {
                    sleep(min_delay - elapsed).await;
                }
            }

            let res = self.http_post(&url, &body_json).await?;
            self.last_recs_v2_call = Some(Instant::now());

            let status = res.status();
            if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::SERVICE_UNAVAILABLE
            {
                rate_limit_attempts += 1;
                if rate_limit_attempts > max_rate_limit_retries {
                    log::warn!(
                        "[rec/v2] rate limited after {} retries; returning aggregated data",
                        rate_limit_attempts
                    );
                    break;
                }

                let exponent = rate_limit_attempts.saturating_sub(1) as u32;
                let factor = 1u64
                    .checked_shl(exponent)
                    .filter(|&v| v > 0)
                    .unwrap_or(u64::MAX);
                let backoff = base_backoff_ms.saturating_mul(factor);
                log::warn!(
                    "[rec/v2] rate limited (status {}). backing off {} ms before retry (attempt {}/{})",
                    status,
                    backoff,
                    rate_limit_attempts,
                    max_rate_limit_retries
                );
                sleep(Duration::from_millis(backoff)).await;
                continue;
            }

            rate_limit_attempts = 0;
            let response = self.parse_response::<RecommendationsResponse>(res).await?;
            if let Some(existing) = aggregated.as_mut() {
                merge_recommendation_responses(existing, response);
            } else {
                aggregated = Some(response);
            }
            completed_calls += 1;
        }

        let mut out = aggregated.unwrap_or_else(|| RecommendationsResponse {
            feeds: Vec::new(),
            active_pills: None,
            cache_control: None,
        });
        normalize_recommendations_response(&mut out);

        if self.auto_persist {
            match self.recs_cache_path() {
                Some(path) => {
                    let _ = self.apply_recommendations_and_save(&mut out, Some(&path));
                }
                None => {
                    let _ = self.apply_recommendations_and_save(&mut out, None);
                }
            }
        }
        Ok(out)
    }

    pub fn apply_recommendations_and_save(
        &mut self,
        recs: &mut RecommendationsResponse,
        path: Option<&str>,
    ) -> Result<(), HingeError> {
        for feed in &mut recs.feeds {
            for subj in &mut feed.subjects {
                if subj.origin.is_none() {
                    subj.origin = Some(feed.origin.clone());
                }
                self.recommendations
                    .entry(subj.subject_id.clone())
                    .or_insert_with(|| subj.clone());
            }
        }
        if let Some(p) = path {
            self.save_recommendations(p)?;
        }
        Ok(())
    }

    pub async fn repeat_profiles(&mut self) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/user/repeat", self.settings.base_url);
        let res = self.http_get(&url).await?;
        let body = self.parse_response(res).await?;
        if self.auto_persist
            && let Some(path) = self.recs_cache_path()
        {
            let _ = self.save_recommendations(&path);
        }
        Ok(body)
    }

    pub fn save_recommendations(&self, path: &str) -> Result<(), HingeError> {
        let data = serde_json::to_string_pretty(&self.recommendations)
            .map_err(|e| HingeError::Serde(e.to_string()))?;
        self.storage
            .write_string(path, &data)
            .map_err(|e| HingeError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn load_recommendations(&mut self, path: &str) -> Result<(), HingeError> {
        if !self.storage.exists(path) {
            return Ok(());
        }
        let data = self
            .storage
            .read_to_string(path)
            .map_err(|e| HingeError::Storage(e.to_string()))?;
        self.recommendations =
            serde_json::from_str(&data).map_err(|e| HingeError::Serde(e.to_string()))?;
        Ok(())
    }

    pub fn remove_recommendation(&mut self, subject_id: &str) {
        self.recommendations.remove(subject_id);
    }
}

fn merge_recommendation_responses(
    base: &mut RecommendationsResponse,
    mut additional: RecommendationsResponse,
) {
    let mut feed_index: HashMap<String, usize> = HashMap::new();
    for (idx, feed) in base.feeds.iter().enumerate() {
        feed_index.insert(feed.origin.clone(), idx);
    }

    for feed in additional.feeds.drain(..) {
        if let Some(&idx) = feed_index.get(&feed.origin) {
            let existing_feed = &mut base.feeds[idx];
            let mut seen: HashSet<String> = existing_feed
                .subjects
                .iter()
                .map(|s| s.subject_id.clone())
                .collect();
            for mut subj in feed.subjects {
                if seen.insert(subj.subject_id.clone()) {
                    if subj.origin.is_none() {
                        subj.origin = Some(feed.origin.clone());
                    }
                    existing_feed.subjects.push(subj);
                }
            }
            if existing_feed.permission.is_none() {
                existing_feed.permission = feed.permission;
            }
            if existing_feed.preview.is_none() {
                existing_feed.preview = feed.preview;
            }
        } else {
            let mut new_feed = feed;
            for subj in &mut new_feed.subjects {
                if subj.origin.is_none() {
                    subj.origin = Some(new_feed.origin.clone());
                }
            }
            feed_index.insert(new_feed.origin.clone(), base.feeds.len());
            base.feeds.push(new_feed);
        }
    }

    match (&mut base.active_pills, additional.active_pills) {
        (Some(existing), Some(mut incoming)) => {
            let mut seen: HashSet<String> = existing.iter().map(|pill| pill.id.clone()).collect();
            for pill in incoming.drain(..) {
                if seen.insert(pill.id.clone()) {
                    existing.push(pill);
                }
            }
        }
        (None, Some(pills)) => base.active_pills = Some(pills),
        _ => {}
    }

    if base.cache_control.is_none() && additional.cache_control.is_some() {
        base.cache_control = additional.cache_control;
    }
}

fn normalize_recommendations_response(response: &mut RecommendationsResponse) {
    let mut ordered_subjects: Vec<RecommendationSubject> = Vec::new();
    let mut seen = HashSet::new();

    for feed in &response.feeds {
        for subj in &feed.subjects {
            if seen.insert(subj.subject_id.clone()) {
                let mut clone = subj.clone();
                if clone.origin.is_none() {
                    clone.origin = Some(feed.origin.clone());
                }
                ordered_subjects.push(clone);
            }
        }
    }

    let (permission, preview) = response
        .feeds
        .first()
        .map(|feed| (feed.permission.clone(), feed.preview.clone()))
        .unwrap_or((None, None));
    let origin = response
        .feeds
        .first()
        .map(|feed| feed.origin.clone())
        .unwrap_or_else(|| "combined".to_string());

    response.feeds = vec![RecommendationsFeed {
        id: 0,
        origin,
        subjects: ordered_subjects,
        permission,
        preview,
    }];
}
