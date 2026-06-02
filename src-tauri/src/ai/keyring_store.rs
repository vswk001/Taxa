// src-tauri/src/ai/keyring_store.rs
use crate::error::{AppError, AppResult};

const SERVICE_NAME: &str = "Taxis";

pub struct KeyringStore;

impl KeyringStore {
    pub fn save_key(provider_id: &str, api_key: &str) -> AppResult<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
            .map_err(|e| AppError::Keyring(e.to_string()))?;
        entry.set_password(api_key)
            .map_err(|e| AppError::Keyring(e.to_string()))?;
        Ok(())
    }

    pub fn get_key(provider_id: &str) -> AppResult<String> {
        let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
            .map_err(|e| AppError::Keyring(e.to_string()))?;
        entry.get_password()
            .map_err(|e| AppError::Keyring(e.to_string()))
    }

    pub fn delete_key(provider_id: &str) -> AppResult<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
            .map_err(|e| AppError::Keyring(e.to_string()))?;
        entry.delete_credential()
            .map_err(|e| AppError::Keyring(e.to_string()))?;
        Ok(())
    }
}
