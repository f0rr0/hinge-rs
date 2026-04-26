use super::HingeClient;
use super::serde_helpers::parse_json_with_path;
use crate::errors::HingeError;
use crate::logging::{log_request, log_response};
use crate::storage::Storage;

impl<S: Storage + Clone> HingeClient<S> {
    pub(super) async fn http_get(&self, url: &str) -> Result<reqwest::Response, HingeError> {
        let headers = self.default_headers()?;
        log_request("GET", url, &headers, None);

        let res = self.http.get(url).headers(headers.clone()).send().await?;

        log::info!("GET {} -> {}", url, res.status());
        Ok(res)
    }

    pub(super) async fn http_get_bytes(&self, url: &str) -> Result<Vec<u8>, HingeError> {
        log::info!("GET (bytes) {}", url);
        let res = self.http.get(url).send().await?;
        let status = res.status();
        if !status.is_success() {
            let text = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read response body".into());
            return Err(HingeError::Http(format!("status {}: {}", status, text)));
        }
        res.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| HingeError::Http(format!("Failed to download media: {}", e)))
    }

    pub(super) async fn http_post(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, HingeError> {
        let headers = self.default_headers()?;
        log_request("POST", url, &headers, Some(body));

        let res = self
            .http
            .post(url)
            .headers(headers.clone())
            .json(body)
            .send()
            .await?;

        log::info!("POST {} -> {}", url, res.status());
        Ok(res)
    }

    pub(super) async fn http_patch(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, HingeError> {
        let headers = self.default_headers()?;
        log_request("PATCH", url, &headers, Some(body));

        let res = self
            .http
            .patch(url)
            .headers(headers.clone())
            .json(body)
            .send()
            .await?;

        log::info!("PATCH {} -> {}", url, res.status());
        Ok(res)
    }

    pub(super) async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        res: reqwest::Response,
    ) -> Result<T, HingeError> {
        let status = res.status();
        let headers = res.headers().clone();

        if !status.is_success() {
            let text = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to get response text".to_string());
            log::error!("HTTP Error {}: {}", status, text);
            return Err(HingeError::Http(format!("status {}: {}", status, text)));
        }

        let text = res.text().await?;
        match parse_json_with_path::<T>(&text) {
            Ok(data) => {
                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&text) {
                    log_response(status, &headers, Some(&json_val));
                }
                Ok(data)
            }
            Err(e) => {
                log::error!("Failed to parse response: {}", e);
                log::error!("Response text: {}", text);
                Err(e)
            }
        }
    }

    pub(super) fn hinge_user_agent(&self) -> String {
        format!(
            "Hinge/{} CFNetwork/3859.100.1 Darwin/25.0.0",
            self.settings.hinge_build_number
        )
    }

    pub(super) fn default_headers(&self) -> Result<reqwest::header::HeaderMap, HingeError> {
        use reqwest::header::{HeaderMap, HeaderValue};

        let mut h = HeaderMap::new();
        h.insert("content-type", HeaderValue::from_static("application/json"));
        h.insert("accept", HeaderValue::from_static("*/*"));
        h.insert("accept-language", HeaderValue::from_static("en-GB"));
        h.insert("connection", HeaderValue::from_static("keep-alive"));
        h.insert(
            "accept-encoding",
            HeaderValue::from_static("gzip, deflate, br"),
        );
        h.insert(
            "x-device-model-code",
            HeaderValue::from_static("iPhone15,2"),
        );
        h.insert("x-device-model", HeaderValue::from_static("unknown"));
        h.insert("x-device-region", HeaderValue::from_static("IN"));
        h.insert(
            "x-session-id",
            HeaderValue::from_str(&self.session_id)
                .map_err(|e| HingeError::Http(format!("Invalid session id header: {}", e)))?,
        );
        h.insert(
            "x-device-id",
            HeaderValue::from_str(&self.device_id)
                .map_err(|e| HingeError::Http(format!("Invalid device id header: {}", e)))?,
        );
        h.insert(
            "x-install-id",
            HeaderValue::from_str(&self.install_id)
                .map_err(|e| HingeError::Http(format!("Invalid install id header: {}", e)))?,
        );
        h.insert("x-device-platform", HeaderValue::from_static("iOS"));
        h.insert(
            "x-app-version",
            HeaderValue::from_str(&self.settings.hinge_app_version)
                .map_err(|e| HingeError::Http(format!("Invalid app version header: {}", e)))?,
        );
        h.insert(
            "x-build-number",
            HeaderValue::from_str(&self.settings.hinge_build_number)
                .map_err(|e| HingeError::Http(format!("Invalid build number header: {}", e)))?,
        );
        h.insert(
            "x-os-version",
            HeaderValue::from_str(&self.settings.os_version)
                .map_err(|e| HingeError::Http(format!("Invalid OS version header: {}", e)))?,
        );
        h.insert(
            "user-agent",
            HeaderValue::from_str(&self.hinge_user_agent())
                .map_err(|e| HingeError::Http(format!("Invalid user agent header: {}", e)))?,
        );
        if let Some(token) = &self.hinge_auth {
            h.insert(
                "authorization",
                HeaderValue::from_str(&format!("Bearer {}", token.token))
                    .map_err(|e| HingeError::Http(format!("Invalid auth token header: {}", e)))?,
            );
        }
        Ok(h)
    }
}
