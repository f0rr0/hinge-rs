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
    /// Sendbird rejected a command sent over the WebSocket, reported as an `EROR` frame.
    ///
    /// Distinct from [`Self::Http`] because the socket is healthy and the request well-formed —
    /// the server declined it. The code is what tells the caller whether a retry can help:
    /// [`crate::ws::SENDBIRD_ERROR_GUEST_NOT_ALLOWED`] means the connection never authenticated,
    /// which re-authenticating fixes, whereas a muted sender or frozen channel will not.
    #[error("sendbird refused the command (code {code}): {message}")]
    SendbirdRefused { code: i64, message: String },
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
