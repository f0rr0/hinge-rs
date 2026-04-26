use crate::client::HingeClient;
use crate::errors::HingeError;
use crate::storage::Storage;
use std::path::PathBuf;

pub struct PersistenceApi<'a, S: Storage + Clone> {
    pub(super) client: &'a mut HingeClient<S>,
}

impl<S: Storage + Clone> PersistenceApi<'_, S> {
    pub fn save_session(&self, path: &str) -> Result<(), HingeError> {
        self.client.save_session(path)
    }

    pub fn load_session(&mut self, path: &str) -> Result<(), HingeError> {
        self.client.load_session(path)
    }

    pub fn configure(
        &mut self,
        session_path: Option<String>,
        cache_dir: Option<PathBuf>,
        auto_persist: bool,
    ) {
        let cloned = self.client.clone();
        *self.client = cloned.with_persistence(session_path, cache_dir, auto_persist);
    }
}
