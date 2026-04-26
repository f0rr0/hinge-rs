use std::fs;
use std::path::Path;

pub trait Storage {
    fn exists(&self, path: &str) -> bool;
    fn read_to_string(&self, path: &str) -> anyhow::Result<String>;
    fn write_string(&self, path: &str, data: &str) -> anyhow::Result<()>;
}

pub trait SecretStore: Send + Sync {
    fn set_secret(&self, key: &str, secret: &str) -> anyhow::Result<()>;
    fn get_secret(&self, key: &str) -> anyhow::Result<Option<String>>;
}

#[derive(Clone, Default)]
pub struct FsStorage;

impl Storage for FsStorage {
    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }
    fn read_to_string(&self, path: &str) -> anyhow::Result<String> {
        Ok(fs::read_to_string(path)?)
    }
    fn write_string(&self, path: &str, data: &str) -> anyhow::Result<()> {
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        Ok(fs::write(path, data)?)
    }
}

#[derive(Default)]
pub struct InMemorySecretStore(std::sync::Mutex<std::collections::HashMap<String, String>>);

impl InMemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for InMemorySecretStore {
    fn set_secret(&self, key: &str, secret: &str) -> anyhow::Result<()> {
        let mut g = self
            .0
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        g.insert(key.to_string(), secret.to_string());
        Ok(())
    }
    fn get_secret(&self, key: &str) -> anyhow::Result<Option<String>> {
        let g = self
            .0
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(g.get(key).cloned())
    }
}
