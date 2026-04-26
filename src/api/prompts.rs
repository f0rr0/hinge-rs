use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    AnswerEvaluateRequest, CreatePromptPollRequest, CreatePromptPollResponse,
    CreateVideoPromptRequest, CreateVideoPromptResponse, Prompt, PromptsResponse,
};
use crate::prompts_manager::HingePromptsManager;
use crate::storage::Storage;

pub struct PromptsApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> PromptsApi<'_, S> {
    pub async fn list(&mut self) -> Result<PromptsResponse, HingeError> {
        self.client.fetch_prompts().await
    }

    pub async fn manager(&mut self) -> Result<HingePromptsManager, HingeError> {
        self.client.fetch_prompts_manager().await
    }

    pub async fn text(&mut self, prompt_id: &str) -> Result<String, HingeError> {
        self.client.get_prompt_text(prompt_id).await
    }

    pub async fn search(&mut self, query: &str) -> Result<Vec<Prompt>, HingeError> {
        self.client.search_prompts(query).await
    }

    pub async fn by_category(&mut self, category_slug: &str) -> Result<Vec<Prompt>, HingeError> {
        self.client.get_prompts_by_category(category_slug).await
    }

    pub async fn payload(&mut self) -> serde_json::Value {
        self.client.prompt_payload().await
    }

    pub async fn evaluate_answer(
        &self,
        payload: AnswerEvaluateRequest,
    ) -> Result<serde_json::Value, HingeError> {
        self.client.evaluate_answer(payload).await
    }

    pub async fn create_prompt_poll(
        &self,
        payload: CreatePromptPollRequest,
    ) -> Result<CreatePromptPollResponse, HingeError> {
        self.client.create_prompt_poll(payload).await
    }

    pub async fn create_video_prompt(
        &self,
        payload: CreateVideoPromptRequest,
    ) -> Result<CreateVideoPromptResponse, HingeError> {
        self.client.create_video_prompt(payload).await
    }
}
