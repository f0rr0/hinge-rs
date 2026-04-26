use super::HingeClient;
use crate::errors::HingeError;
use crate::logging::log_request;
use crate::storage::Storage;

impl<S: Storage + Clone> HingeClient<S> {
    pub async fn raw_hinge_json(
        &self,
        method: reqwest::Method,
        path_or_url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
            path_or_url.to_string()
        } else {
            format!(
                "{}/{}",
                self.settings.base_url.trim_end_matches('/'),
                path_or_url.trim_start_matches('/')
            )
        };
        let headers = self.default_headers()?;
        log_request(method.as_str(), &url, &headers, body.as_ref());
        let mut request = self.http.request(method, &url).headers(headers);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let res = request.send().await?;
        self.parse_response(res).await
    }

    pub async fn raw_sendbird_json(
        &self,
        method: reqwest::Method,
        path_or_url: &str,
        body: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, HingeError> {
        let url = if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
            path_or_url.to_string()
        } else {
            format!(
                "{}/v3/{}",
                self.settings.sendbird_api_url.trim_end_matches('/'),
                path_or_url.trim_start_matches('/')
            )
        };
        let mut headers = self.sendbird_headers()?;
        if body.is_some() {
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/json"),
            );
        }
        log_request(method.as_str(), &url, &headers, body.as_ref());
        let mut request = self.http.request(method, &url).headers(headers);
        if let Some(body) = body {
            request = request.json(&body);
        }
        let res = request.send().await?;
        self.parse_response(res).await
    }
}
