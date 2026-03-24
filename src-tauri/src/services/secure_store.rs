#[cfg(not(test))]
use crate::error::AppError;
use crate::error::AppResult;

pub const SECRET_SETTING_KEYS: [&str; 2] = ["claude_api_key", "stripe_api_key"];
#[cfg(not(test))]
const SECURE_STORE_SERVICE: &str = "com.freelanceinvoice.desktop";

pub fn is_secret_setting(key: &str) -> bool {
    SECRET_SETTING_KEYS.contains(&key)
}

#[cfg(not(test))]
pub fn set_secret(key: &str, value: &str) -> AppResult<()> {
    let entry = keyring::Entry::new(SECURE_STORE_SERVICE, key)
        .map_err(|err| AppError::Security(err.to_string()))?;

    if value.trim().is_empty() {
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(AppError::Security(err.to_string())),
        }
    } else {
        entry
            .set_password(value)
            .map_err(|err| AppError::Security(err.to_string()))
    }
}

#[cfg(not(test))]
pub fn get_secret(key: &str) -> AppResult<Option<String>> {
    let entry = keyring::Entry::new(SECURE_STORE_SERVICE, key)
        .map_err(|err| AppError::Security(err.to_string()))?;

    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(AppError::Security(err.to_string())),
    }
}

#[cfg(test)]
mod memory_store {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};

    static STORE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

    fn inner() -> &'static Mutex<HashMap<String, String>> {
        STORE.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub fn set_secret(key: &str, value: &str) {
        let mut store = inner().lock().expect("lock secure store");
        if value.trim().is_empty() {
            store.remove(key);
        } else {
            store.insert(key.to_string(), value.to_string());
        }
    }

    pub fn get_secret(key: &str) -> Option<String> {
        inner().lock().expect("lock secure store").get(key).cloned()
    }
}

#[cfg(test)]
pub fn set_secret(key: &str, value: &str) -> AppResult<()> {
    memory_store::set_secret(key, value);
    Ok(())
}

#[cfg(test)]
pub fn get_secret(key: &str) -> AppResult<Option<String>> {
    Ok(memory_store::get_secret(key))
}

#[cfg(test)]
mod tests {
    use super::{get_secret, is_secret_setting, set_secret};

    #[test]
    fn recognizes_secret_keys() {
        assert!(is_secret_setting("claude_api_key"));
        assert!(!is_secret_setting("business_name"));
    }

    #[test]
    fn stores_and_clears_secret_values() {
        set_secret("claude_api_key", "top-secret").unwrap();
        assert_eq!(
            get_secret("claude_api_key").unwrap(),
            Some("top-secret".to_string())
        );

        set_secret("claude_api_key", "").unwrap();
        assert_eq!(get_secret("claude_api_key").unwrap(), None);
    }
}
