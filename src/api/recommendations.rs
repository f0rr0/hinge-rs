use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::{RecommendationSubject, RecommendationsResponse, RecsV2Params};
use crate::storage::Storage;
use std::collections::HashMap;

pub struct RecommendationsApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> RecommendationsApi<'_, S> {
    pub async fn get(&mut self) -> Result<RecommendationsResponse, HingeError> {
        self.client.get_recommendations().await
    }

    pub async fn get_with_params(
        &mut self,
        params: RecsV2Params,
    ) -> Result<RecommendationsResponse, HingeError> {
        self.client.get_recommendations_v2_params(params).await
    }

    pub async fn repeat_profiles(&mut self) -> Result<serde_json::Value, HingeError> {
        self.client.repeat_profiles().await
    }

    pub fn apply_and_save(
        &mut self,
        recs: &mut RecommendationsResponse,
        path: Option<&str>,
    ) -> Result<(), HingeError> {
        self.client.apply_recommendations_and_save(recs, path)
    }

    pub fn save(&self, path: &str) -> Result<(), HingeError> {
        self.client.save_recommendations(path)
    }

    pub fn load(&mut self, path: &str) -> Result<(), HingeError> {
        self.client.load_recommendations(path)
    }

    pub fn remove(&mut self, subject_id: &str) {
        self.client.remove_recommendation(subject_id);
    }

    pub fn cached(&self) -> &HashMap<String, RecommendationSubject> {
        &self.client.recommendations
    }
}
