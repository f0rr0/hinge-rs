use thiserror::Error;

#[derive(Debug, Error)]
pub enum HingeError {
    #[error("http: {0}")]
    Http(String),
    #[error("auth: {0}")]
    Auth(String),
    #[error("email_2fa required: case_id={case_id} email={email}")]
    Email2FA { case_id: String, email: String },
    #[error("storage: {0}")]
    Storage(String),
    #[error("serde: {0}")]
    Serde(String),
}

impl From<reqwest::Error> for HingeError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}
impl From<serde_json::Error> for HingeError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e.to_string())
    }
}
