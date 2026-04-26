use super::HingeClient;
use crate::errors::HingeError;
use crate::models::{HingeAuthToken, LoginTokens, SendbirdAuthToken};
use crate::storage::Storage;
use chrono::Utc;
use serde_json::json;

impl<S: Storage + Clone> HingeClient<S> {
    async fn ensure_device_registered(&mut self) -> Result<(), HingeError> {
        if self.installed {
            return Ok(());
        }

        let url = format!("{}/identity/install", self.settings.base_url);
        let body = json!({"installId": self.install_id});
        let res = self
            .http_post(&url, &body)
            .await
            .map_err(|e| HingeError::Http(format!("Failed to register device: {}", e)))?;

        if !res.status().is_success() {
            return Err(HingeError::Http(format!(
                "Device registration failed with status {}",
                res.status()
            )));
        }
        self.installed = true;
        Ok(())
    }

    pub async fn initiate_login(&mut self) -> Result<(), HingeError> {
        self.ensure_device_registered().await?;

        let url = format!("{}/auth/sms/v2/initiate", self.settings.base_url);
        let body = json!({"deviceId": self.device_id, "phoneNumber": self.phone_number});
        let res = self
            .http_post(&url, &body)
            .await
            .map_err(|e| HingeError::Http(format!("Failed to initiate SMS login: {}", e)))?;

        if !res.status().is_success() {
            return Err(HingeError::Http(format!(
                "SMS initiation failed with status {}",
                res.status()
            )));
        }
        Ok(())
    }

    pub async fn submit_otp(&mut self, otp: &str) -> Result<LoginTokens, HingeError> {
        let url = format!("{}/auth/sms/v2", self.settings.base_url);
        let body = json!({
            "installId": self.install_id,
            "deviceId": self.device_id,
            "phoneNumber": self.phone_number,
            "otp": otp,
        });
        let res = self.http_post(&url, &body).await?;

        if res.status() == reqwest::StatusCode::PRECONDITION_FAILED {
            let v: serde_json::Value = res.json().await?;
            let case_id = v
                .get("caseId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let email = v
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            return Err(HingeError::Email2FA { case_id, email });
        }
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }

        let v = self.parse_response::<LoginTokens>(res).await?;
        if let Some(t) = v.hinge_auth_token.clone() {
            self.hinge_auth = Some(t);
        }
        if let Some(t) = v.sendbird_auth_token.clone() {
            self.sendbird_auth = Some(t);
        }
        Ok(v)
    }

    pub async fn submit_email_code(
        &mut self,
        case_id: &str,
        email_code: &str,
    ) -> Result<LoginTokens, HingeError> {
        let url = format!("{}/auth/device/validate", self.settings.base_url);
        let body = json!({
            "installId": self.install_id,
            "code": email_code,
            "caseId": case_id,
            "deviceId": self.device_id,
        });
        let res = self.http_post(&url, &body).await?;
        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }

        let t = self.parse_response::<HingeAuthToken>(res).await?;
        self.hinge_auth = Some(t);
        let _ = self.authenticate_with_sendbird().await;
        Ok(LoginTokens {
            hinge_auth_token: self.hinge_auth.clone(),
            sendbird_auth_token: self.sendbird_auth.clone(),
        })
    }

    pub(super) async fn authenticate_with_sendbird(&mut self) -> Result<(), HingeError> {
        let _hinge = self
            .hinge_auth
            .as_ref()
            .ok_or_else(|| HingeError::Auth("hinge token missing".into()))?;
        let url = format!("{}/message/authenticate", self.settings.base_url);
        let res = self
            .http
            .post(url)
            .headers(self.default_headers()?)
            .json(&json!({"refresh": false}))
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(HingeError::Http(format!("status {}", res.status())));
        }
        let v = self.parse_response::<SendbirdAuthToken>(res).await?;
        self.sendbird_auth = Some(v);

        if self.auto_persist
            && let Some(path) = &self.session_path
        {
            let _ = self.save_session(path);
        }
        Ok(())
    }

    pub async fn is_session_valid(&mut self) -> Result<bool, HingeError> {
        if self.hinge_auth.is_none() {
            log::warn!("Hinge token is empty, session is invalid.");
            return Ok(false);
        }

        if self.sendbird_auth.is_none() {
            log::warn!("Sendbird JWT is empty, reauthenticating...");
            if let Err(e) = self.authenticate_with_sendbird().await {
                log::error!("Failed to reauthenticate with Sendbird: {}", e);
                return Ok(false);
            }
        }

        let now = Utc::now();
        let hinge_token_valid = self
            .hinge_auth
            .as_ref()
            .is_some_and(|hinge_auth| hinge_auth.expires > now);
        let sendbird_needs_refresh = self
            .sendbird_auth
            .as_ref()
            .is_none_or(|sb_auth| sb_auth.expires <= now);

        if sendbird_needs_refresh {
            log::warn!("Sendbird JWT has expired or is missing, reauthenticating...");
            if let Err(e) = self.authenticate_with_sendbird().await {
                log::error!("Failed to reauthenticate with Sendbird: {}", e);
                return Ok(false);
            }
        }

        let sendbird_token_valid = self
            .sendbird_auth
            .as_ref()
            .is_some_and(|sb_auth| sb_auth.expires > now);
        let is_valid = hinge_token_valid && sendbird_token_valid;
        log::info!(
            "Session validity check: is_valid={}, hinge_token_valid={}, sendbird_token_valid={}",
            is_valid,
            hinge_token_valid,
            sendbird_token_valid
        );

        Ok(is_valid)
    }
}
