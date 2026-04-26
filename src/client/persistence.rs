use super::HingeClient;
use crate::errors::HingeError;
use crate::storage::Storage;
use serde_json::json;
use std::path::PathBuf;

impl<S: Storage + Clone> HingeClient<S> {
    pub fn save_session(&self, path: &str) -> Result<(), HingeError> {
        let session = json!({
          "phoneNumber": self.phone_number,
          "deviceId": self.device_id,
          "installId": self.install_id,
          "sessionId": self.session_id,
          "installed": self.installed,
          "hingeAuth": self.hinge_auth,
          "sendbirdAuth": self.sendbird_auth,
          "sendbirdSessionKey": self.sendbird_session_key,
        });
        let data =
            serde_json::to_string_pretty(&session).map_err(|e| HingeError::Serde(e.to_string()))?;
        self.storage
            .write_string(path, &data)
            .map_err(|e| HingeError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn load_session(&mut self, path: &str) -> Result<(), HingeError> {
        if !self.storage.exists(path) {
            return Ok(());
        }

        let data = self
            .storage
            .read_to_string(path)
            .map_err(|e| HingeError::Storage(e.to_string()))?;
        let v: serde_json::Value =
            serde_json::from_str(&data).map_err(|e| HingeError::Serde(e.to_string()))?;

        if let Some(s) = v.get("phoneNumber").and_then(|v| v.as_str()) {
            self.phone_number = s.to_string();
        }
        if let Some(s) = v.get("deviceId").and_then(|v| v.as_str()) {
            self.device_id = s.to_string();
        }
        if let Some(s) = v.get("installId").and_then(|v| v.as_str()) {
            self.install_id = s.to_string();
        }
        if let Some(s) = v.get("sessionId").and_then(|v| v.as_str()) {
            self.session_id = s.to_string();
        }
        if let Some(b) = v.get("installed").and_then(|v| v.as_bool()) {
            self.installed = b;
        }
        if let Some(t) = v.get("hingeAuth").cloned() {
            self.hinge_auth = serde_json::from_value(t).ok();
        }
        if let Some(t) = v.get("sendbirdAuth").cloned() {
            self.sendbird_auth = serde_json::from_value(t).ok();
        }
        if let Some(k) = v.get("sendbirdSessionKey").and_then(|v| v.as_str()) {
            self.sendbird_session_key = Some(k.to_string());
        }
        Ok(())
    }

    pub fn load_tokens_secure(&mut self) -> Result<(), HingeError> {
        if let Some(store) = &self.secret_store {
            if let Some(v) = store
                .get_secret("hinge_auth")
                .map_err(|e| HingeError::Storage(e.to_string()))?
            {
                self.hinge_auth = serde_json::from_str(&v).ok();
            }
            if let Some(v) = store
                .get_secret("sendbird_auth")
                .map_err(|e| HingeError::Storage(e.to_string()))?
            {
                self.sendbird_auth = serde_json::from_str(&v).ok();
            }
        }
        Ok(())
    }

    pub fn with_persistence(
        mut self,
        session_path: Option<String>,
        cache_dir: Option<PathBuf>,
        auto_persist: bool,
    ) -> Self {
        self.session_path = session_path;
        self.cache_dir = cache_dir;
        self.auto_persist = auto_persist;

        if let Some(path) = self.session_path.clone() {
            let _ = self.load_session(&path);
        }
        if let Some(dir) = &self.cache_dir {
            let rec_path = dir.join(format!("recommendations_{}.json", self.session_id));
            let _ = self.load_recommendations(rec_path.to_string_lossy().as_ref());
        }
        self
    }

    pub(super) fn recs_cache_path(&self) -> Option<String> {
        self.cache_dir.as_ref().map(|d| {
            d.join(format!("recommendations_{}.json", self.session_id))
                .to_string_lossy()
                .into_owned()
        })
    }

    pub(super) fn prompts_cache_path(&self) -> Option<String> {
        self.cache_dir
            .as_ref()
            .map(|d| d.join("prompts_cache.json").to_string_lossy().into_owned())
    }
}
