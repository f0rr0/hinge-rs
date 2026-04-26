use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::models::LoginTokens;
use crate::storage::Storage;

pub struct AuthApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> AuthApi<'_, S> {
    pub async fn initiate_sms(&mut self) -> Result<(), HingeError> {
        self.client.initiate_login().await
    }

    pub async fn submit_otp(&mut self, otp: &str) -> Result<LoginTokens, HingeError> {
        self.client.submit_otp(otp).await
    }

    pub async fn submit_email_code(
        &mut self,
        case_id: &str,
        email_code: &str,
    ) -> Result<LoginTokens, HingeError> {
        self.client.submit_email_code(case_id, email_code).await
    }

    pub fn load_tokens_secure(&mut self) -> Result<(), HingeError> {
        self.client.load_tokens_secure()
    }

    pub async fn is_session_valid(&mut self) -> Result<bool, HingeError> {
        self.client.is_session_valid().await
    }
}
