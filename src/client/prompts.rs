use super::HingeClient;
use crate::errors::HingeError;
use crate::models::{
    AnswerEvaluateRequest, CreatePromptPollRequest, CreatePromptPollResponse,
    CreateVideoPromptRequest, CreateVideoPromptResponse, Prompt, PromptsResponse,
};
use crate::prompts_manager::HingePromptsManager;
use crate::storage::Storage;
use serde_json::json;
use std::path::Path;

impl<S: Storage + Clone> HingeClient<S> {
    pub async fn fetch_prompts(&mut self) -> Result<PromptsResponse, HingeError> {
        if self.auto_persist
            && let Some(path) = self.prompts_cache_path()
            && Path::new(&path).exists()
            && let Ok(text) = std::fs::read_to_string(&path)
            && let Ok(val) = serde_json::from_str::<PromptsResponse>(&text)
        {
            return Ok(val);
        }

        let url = format!("{}/prompts", self.settings.base_url);
        let payload = self.prompt_payload().await;
        let res = self.http_post(&url, &payload).await?;
        let body = self.parse_response::<PromptsResponse>(res).await?;
        if self.auto_persist
            && let Some(path) = self.prompts_cache_path()
        {
            let _ = std::fs::write(
                &path,
                serde_json::to_string_pretty(&body).unwrap_or_else(|_| "{}".into()),
            );
        }
        Ok(body)
    }

    pub async fn fetch_prompts_manager(&mut self) -> Result<HingePromptsManager, HingeError> {
        let resp = self.fetch_prompts().await?;
        Ok(HingePromptsManager::new(resp))
    }

    pub async fn get_prompt_text(&mut self, prompt_id: &str) -> Result<String, HingeError> {
        let mgr = self.fetch_prompts_manager().await?;
        Ok(mgr.get_prompt_display_text(prompt_id))
    }

    pub async fn search_prompts(&mut self, query: &str) -> Result<Vec<Prompt>, HingeError> {
        let mgr = self.fetch_prompts_manager().await?;
        let items = mgr.search_prompts(query);
        Ok(items.into_iter().cloned().collect())
    }

    pub async fn get_prompts_by_category(
        &mut self,
        category_slug: &str,
    ) -> Result<Vec<Prompt>, HingeError> {
        let mgr = self.fetch_prompts_manager().await?;
        let items = mgr.get_prompts_by_category(category_slug);
        Ok(items.into_iter().cloned().collect())
    }

    pub async fn prompt_payload(&mut self) -> serde_json::Value {
        if !self.is_session_valid().await.unwrap_or(false) {
            return json!({});
        }
        let preferences = match self.get_self_preferences().await {
            Ok(v) => v,
            Err(_) => return json!({}),
        };
        let profile = match self.get_self_profile().await {
            Ok(v) => v,
            Err(_) => return json!({}),
        };
        let mut preferences_dict = serde_json::to_value(&preferences).unwrap_or(json!({}));
        let profile_dict = serde_json::to_value(&profile).unwrap_or(json!({}));

        let selected: Vec<String> = preferences_dict
            .get("preferences")
            .and_then(|p| p.get("genderPreferences"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_u64().map(|n| n.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let keep_selected = |mut d: serde_json::Value| {
            if let serde_json::Value::Object(map) = &mut d
                && !selected.is_empty()
            {
                map.retain(|k, _| selected.contains(k));
            }
            d
        };

        if let Some(obj) = preferences_dict
            .get_mut("preferences")
            .and_then(|p| p.get_mut("genderedHeightRanges"))
        {
            *obj = keep_selected(obj.clone());
        }
        if let Some(obj) = preferences_dict
            .get_mut("preferences")
            .and_then(|p| p.get_mut("genderedAgeRanges"))
        {
            *obj = keep_selected(obj.clone());
        }

        if let Some(db) = preferences_dict
            .get_mut("preferences")
            .and_then(|p| p.get_mut("dealbreakers"))
        {
            if let Some(obj) = db.get_mut("genderedHeight") {
                *obj = keep_selected(obj.clone());
            }
            if let Some(obj) = db.get_mut("genderedAge") {
                *obj = keep_selected(obj.clone());
            }
        }

        let p = profile_dict
            .get("content")
            .map(unwrap_visible)
            .unwrap_or(json!({}));
        let loc_name = profile_dict
            .get("content")
            .and_then(|c| c.get("location"))
            .and_then(|l| l.get("name"))
            .cloned()
            .unwrap_or(json!(null));

        let profile_payload = json!({
          "works": match p.get("works") { Some(v) if v.is_string() => json!([v]), _ => field_or(&p, "works", json!([])) },
          "sexualOrientations": field_or(&p, "sexualOrientations", json!([])),
          "didJustJoin": false,
          "smoking": field_or(&p, "smoking", json!(null)),
          "selfieVerified": field_or(&p, "selfieVerified", json!(false)),
          "politics": field_or(&p, "politics", json!(null)),
          "relationshipTypesText": field_or(&p, "relationshipTypesText", json!("")),
          "datingIntention": field_or(&p, "datingIntention", json!(null)),
          "height": field_or(&p, "height", json!(null)),
          "children": field_or(&p, "children", json!(null)),
          "religions": field_or(&p, "religions", json!([])),
          "relationshipTypes": field_or(&p, "relationshipTypeIds", json!([])),
          "educations": field_or(&p, "educations", json!([])),
          "age": field_or(&p, "age", json!(null)),
          "jobTitle": field_or(&p, "jobTitle", json!(null)),
          "birthday": field_or(&p, "birthday", json!(null)),
          "drugs": field_or(&p, "drugs", json!(null)),
          "content": json!({}),
          "hometown": field_or(&p, "hometown", json!(null)),
          "firstName": field_or(&p, "firstName", json!(null)),
          "familyPlans": field_or(&p, "familyPlans", json!(null)),
          "location": json!({"name": loc_name}),
          "marijuana": field_or(&p, "marijuana", json!(null)),
          "pets": field_or(&p, "pets", json!([])),
          "datingIntentionText": field_or(&p, "datingIntentionText", json!("")),
          "educationAttained": field_or(&p, "educationAttained", json!(null)),
          "ethnicities": field_or(&p, "ethnicities", json!([])),
          "pronouns": field_or(&p, "pronouns", json!([])),
          "languagesSpoken": field_or(&p, "languagesSpoken", json!([])),
          "lastName": field_or(&p, "lastName", json!("")),
          "ethnicitiesText": field_or(&p, "ethnicitiesText", json!("")),
          "drinking": field_or(&p, "drinking", json!(null)),
          "userId": field_or(&profile_dict, "userId", json!(null)),
          "genderIdentityId": field_or(&p, "genderIdentityId", json!(null)),
        });

        json!({
          "preferences": preferences_dict.get("preferences").cloned().unwrap_or(json!({})),
          "profile": profile_payload
        })
    }

    pub async fn evaluate_answer(
        &self,
        payload: AnswerEvaluateRequest,
    ) -> Result<serde_json::Value, HingeError> {
        let url = format!("{}/content/v1/answer/evaluate", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }

    pub async fn create_prompt_poll(
        &self,
        payload: CreatePromptPollRequest,
    ) -> Result<CreatePromptPollResponse, HingeError> {
        let url = format!("{}/content/v1/prompt_poll", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }

    pub async fn create_video_prompt(
        &self,
        payload: CreateVideoPromptRequest,
    ) -> Result<CreateVideoPromptResponse, HingeError> {
        let url = format!("{}/content/v1/video_prompt", self.settings.base_url);
        let body = serde_json::to_value(&payload).map_err(|e| HingeError::Serde(e.to_string()))?;
        let res = self.http_post(&url, &body).await?;
        self.parse_response(res).await
    }
}

fn field_or(value: &serde_json::Value, key: &str, default: serde_json::Value) -> serde_json::Value {
    value.get(key).cloned().unwrap_or(default)
}

fn unwrap_visible(obj: &serde_json::Value) -> serde_json::Value {
    match obj {
        serde_json::Value::Object(m) => {
            if m.contains_key("value") && m.contains_key("visible") {
                unwrap_visible(&m["value"])
            } else {
                let mut out = serde_json::Map::new();
                for (k, v) in m.iter() {
                    out.insert(k.clone(), unwrap_visible(v));
                }
                serde_json::Value::Object(out)
            }
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(unwrap_visible).collect())
        }
        _ => obj.clone(),
    }
}
