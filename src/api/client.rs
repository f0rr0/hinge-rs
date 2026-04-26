use super::{
    AuthApi, ChatApi, ConnectionsApi, LikesApi, PersistenceApi, ProfilesApi, PromptsApi,
    RatingsApi, RawApi, RecommendationsApi, SettingsApi,
};
use crate::client::{DEFAULT_PUBLIC_IDS_BATCH_SIZE, HingeClient, RecsFetchConfig};
use crate::errors::HingeError;
use crate::settings::Settings;
use crate::storage::{FsStorage, SecretStore, Storage};
use secrecy::SecretString;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Config {
    pub settings: Settings,
    pub recs_fetch_config: RecsFetchConfig,
    pub public_ids_batch_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            recs_fetch_config: RecsFetchConfig::default(),
            public_ids_batch_size: DEFAULT_PUBLIC_IDS_BATCH_SIZE,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceProfile {
    pub device_id: String,
    pub install_id: String,
    pub session_id: String,
    pub installed: bool,
}

#[derive(Clone)]
pub struct Session {
    pub phone_number: String,
    pub device: DeviceProfile,
    pub hinge_identity_id: Option<String>,
    pub hinge_auth_token: Option<SecretString>,
    pub sendbird_auth_token: Option<SecretString>,
    pub sendbird_session_key: Option<SecretString>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("phone_number", &self.phone_number)
            .field("device", &self.device)
            .field("hinge_identity_id", &self.hinge_identity_id)
            .field(
                "hinge_auth_token",
                &self.hinge_auth_token.as_ref().map(|_| "[redacted]"),
            )
            .field(
                "sendbird_auth_token",
                &self.sendbird_auth_token.as_ref().map(|_| "[redacted]"),
            )
            .field(
                "sendbird_session_key",
                &self.sendbird_session_key.as_ref().map(|_| "[redacted]"),
            )
            .finish()
    }
}

impl Session {
    pub fn from_inner<S: Storage + Clone>(client: &HingeClient<S>) -> Self {
        Self {
            phone_number: client.phone_number.clone(),
            device: DeviceProfile {
                device_id: client.device_id.clone(),
                install_id: client.install_id.clone(),
                session_id: client.session_id.clone(),
                installed: client.installed,
            },
            hinge_identity_id: client
                .hinge_auth
                .as_ref()
                .map(|token| token.identity_id.clone()),
            hinge_auth_token: client
                .hinge_auth
                .as_ref()
                .map(|token| SecretString::new(token.token.clone().into())),
            sendbird_auth_token: client
                .sendbird_auth
                .as_ref()
                .map(|token| SecretString::new(token.token.clone().into())),
            sendbird_session_key: client
                .sendbird_session_key
                .as_ref()
                .map(|key| SecretString::new(key.clone().into())),
        }
    }
}

pub struct ClientBuilder {
    phone_number: Option<String>,
    config: Config,
    secret_store: Option<Arc<dyn SecretStore>>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            phone_number: None,
            config: Config::default(),
            secret_store: None,
        }
    }

    pub fn phone_number(mut self, phone_number: impl Into<String>) -> Self {
        self.phone_number = Some(phone_number.into());
        self
    }

    pub fn settings(mut self, settings: Settings) -> Self {
        self.config.settings = settings;
        self
    }

    pub fn recs_fetch_config(mut self, config: RecsFetchConfig) -> Self {
        self.config.recs_fetch_config = config;
        self
    }

    pub fn public_ids_batch_size(mut self, batch_size: usize) -> Self {
        self.config.public_ids_batch_size = batch_size.max(1);
        self
    }

    pub fn secret_store(mut self, store: Arc<dyn SecretStore>) -> Self {
        self.secret_store = Some(store);
        self
    }

    pub fn build(self) -> Result<Client<FsStorage>, HingeError> {
        let phone_number = self
            .phone_number
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| HingeError::Auth("phone number is required".into()))?;
        let mut client = Client::with_storage(phone_number, FsStorage, self.config);
        if let Some(store) = self.secret_store {
            client.inner = client.inner.with_secret_store(store);
        }
        Ok(client)
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct Client<S: Storage + Clone = FsStorage> {
    pub(super) inner: HingeClient<S>,
}

impl Client<FsStorage> {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }
}

impl<S: Storage + Clone> Client<S> {
    pub fn with_storage(phone_number: impl Into<String>, storage: S, config: Config) -> Self {
        let mut inner = HingeClient::new(phone_number, storage, Some(config.settings));
        inner.set_recs_fetch_config(config.recs_fetch_config);
        inner.set_public_ids_batch_size(config.public_ids_batch_size);
        Self { inner }
    }

    pub fn from_inner(inner: HingeClient<S>) -> Self {
        Self { inner }
    }

    pub fn with_secret_store(mut self, store: Arc<dyn SecretStore>) -> Self {
        self.inner = self.inner.with_secret_store(store);
        self
    }

    pub fn inner(&self) -> &HingeClient<S> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut HingeClient<S> {
        &mut self.inner
    }

    pub fn into_inner(self) -> HingeClient<S> {
        self.inner
    }

    pub fn session(&self) -> Session {
        Session::from_inner(&self.inner)
    }

    pub fn set_recs_fetch_config(&mut self, config: RecsFetchConfig) {
        self.inner.set_recs_fetch_config(config);
    }

    pub fn set_public_ids_batch_size(&mut self, batch_size: usize) {
        self.inner.set_public_ids_batch_size(batch_size);
    }

    pub fn auth(&mut self) -> AuthApi<'_, S> {
        AuthApi {
            client: &mut self.inner,
        }
    }

    pub fn recommendations(&mut self) -> RecommendationsApi<'_, S> {
        RecommendationsApi {
            client: &mut self.inner,
        }
    }

    pub fn profiles(&mut self) -> ProfilesApi<'_, S> {
        ProfilesApi {
            client: &mut self.inner,
        }
    }

    pub fn likes(&mut self) -> LikesApi<'_, S> {
        LikesApi {
            client: &mut self.inner,
        }
    }

    pub fn ratings(&mut self) -> RatingsApi<'_, S> {
        RatingsApi {
            client: &mut self.inner,
        }
    }

    pub fn prompts(&mut self) -> PromptsApi<'_, S> {
        PromptsApi {
            client: &mut self.inner,
        }
    }

    pub fn connections(&mut self) -> ConnectionsApi<'_, S> {
        ConnectionsApi {
            client: &mut self.inner,
        }
    }

    pub fn settings(&mut self) -> SettingsApi<'_, S> {
        SettingsApi {
            client: &mut self.inner,
        }
    }

    pub fn chat(&mut self) -> ChatApi<'_, S> {
        ChatApi {
            client: &mut self.inner,
        }
    }

    pub fn persistence(&mut self) -> PersistenceApi<'_, S> {
        PersistenceApi {
            client: &mut self.inner,
        }
    }

    pub fn raw(&mut self) -> RawApi<'_, S> {
        RawApi {
            client: &mut self.inner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HingeAuthToken, SendbirdAuthToken};
    use chrono::Utc;

    #[test]
    fn session_debug_redacts_secrets() {
        let mut inner = HingeClient::new("+15555550123", FsStorage, None);
        inner.hinge_auth = Some(HingeAuthToken {
            identity_id: "user-1".into(),
            token: "hinge-secret-token".into(),
            expires: Utc::now(),
        });
        inner.sendbird_auth = Some(SendbirdAuthToken {
            token: "sendbird-secret-token".into(),
            expires: Utc::now(),
        });
        inner.sendbird_session_key = Some("session-secret-key".into());

        let session = Session::from_inner(&inner);
        let debug = format!("{session:?}");

        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("hinge-secret-token"));
        assert!(!debug.contains("sendbird-secret-token"));
        assert!(!debug.contains("session-secret-key"));
    }

    #[test]
    fn builder_requires_phone_number() {
        let error = match Client::builder().build() {
            Ok(_) => panic!("builder unexpectedly succeeded without a phone number"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("phone number is required"));
    }
}
