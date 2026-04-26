use crate::models::{Prompt, PromptCategory, PromptsResponse};
use std::collections::HashMap;

pub struct HingePromptsManager {
    pub prompts_data: PromptsResponse,
    prompts_by_id: HashMap<String, Prompt>,
    categories_by_slug: HashMap<String, PromptCategory>,
}

impl HingePromptsManager {
    pub fn new(prompts_data: PromptsResponse) -> Self {
        let mut prompts_by_id = HashMap::new();
        let mut categories_by_slug = HashMap::new();

        for prompt in &prompts_data.prompts {
            prompts_by_id.insert(prompt.id.clone(), prompt.clone());
        }

        for category in &prompts_data.categories {
            categories_by_slug.insert(category.slug.clone(), category.clone());
        }

        Self {
            prompts_data,
            prompts_by_id,
            categories_by_slug,
        }
    }

    pub fn get_prompt_by_id(&self, prompt_id: &str) -> Option<&Prompt> {
        self.prompts_by_id.get(prompt_id)
    }

    pub fn get_category_by_slug(&self, slug: &str) -> Option<&PromptCategory> {
        self.categories_by_slug.get(slug)
    }

    pub fn get_prompts_by_category(&self, category_slug: &str) -> Vec<&Prompt> {
        self.prompts_data
            .prompts
            .iter()
            .filter(|p| p.categories.contains(&category_slug.to_string()))
            .collect()
    }

    pub fn get_selectable_prompts(&self) -> Vec<&Prompt> {
        self.prompts_data
            .prompts
            .iter()
            .filter(|p| p.is_selectable)
            .collect()
    }

    pub fn get_new_prompts(&self) -> Vec<&Prompt> {
        self.prompts_data
            .prompts
            .iter()
            .filter(|p| p.is_new)
            .collect()
    }

    pub fn search_prompts(&self, query: &str) -> Vec<&Prompt> {
        let query_lower = query.to_lowercase();
        self.prompts_data
            .prompts
            .iter()
            .filter(|p| {
                p.prompt.to_lowercase().contains(&query_lower)
                    || p.placeholder.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    pub fn get_prompt_display_text(&self, prompt_id: &str) -> String {
        self.get_prompt_by_id(prompt_id)
            .map(|p| p.prompt.clone())
            .unwrap_or_else(|| "Unknown Question".to_string())
    }

    pub fn get_visible_categories(&self) -> Vec<&PromptCategory> {
        self.prompts_data
            .categories
            .iter()
            .filter(|c| c.is_visible)
            .collect()
    }
}
