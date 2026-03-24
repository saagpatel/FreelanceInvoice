use rusqlite::params;
use tauri::State;

use crate::error::AppResult;
use crate::models::AppSetting;
use crate::services::secure_store;
use crate::DbState;

#[tauri::command]
pub fn set_setting(state: State<DbState>, key: String, value: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    persist_setting(&conn, &key, &value)?;
    Ok(())
}

#[tauri::command]
pub fn get_all_settings(state: State<DbState>) -> AppResult<Vec<AppSetting>> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    load_all_settings(&conn)
}

fn persist_setting(conn: &rusqlite::Connection, key: &str, value: &str) -> AppResult<()> {
    if secure_store::is_secret_setting(key) {
        secure_store::set_secret(key, value)?;
        conn.execute("DELETE FROM app_settings WHERE key = ?1", params![key])?;
        return Ok(());
    }

    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

fn load_all_settings(conn: &rusqlite::Connection) -> AppResult<Vec<AppSetting>> {
    let mut stmt = conn.prepare("SELECT key, value FROM app_settings")?;
    let mut settings = stmt
        .query_map([], |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    for secret_key in secure_store::SECRET_SETTING_KEYS {
        if let Some(existing_index) = settings
            .iter()
            .position(|setting| setting.key == secret_key)
        {
            let legacy_secret = settings.remove(existing_index);
            if !legacy_secret.value.trim().is_empty() {
                secure_store::set_secret(secret_key, &legacy_secret.value)?;
            }
            conn.execute(
                "DELETE FROM app_settings WHERE key = ?1",
                params![secret_key],
            )?;
        }

        if let Some(secret_value) = secure_store::get_secret(secret_key)? {
            settings.push(AppSetting {
                key: secret_key.to_string(),
                value: secret_value,
            });
        }
    }

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::{load_all_settings, persist_setting};
    use crate::db::init_db_in_memory;

    #[test]
    fn stores_secrets_outside_plaintext_settings() {
        let conn = init_db_in_memory().expect("init db");
        persist_setting(&conn, "claude_api_key", "secret-value").unwrap();

        let db_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM app_settings WHERE key = 'claude_api_key'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(db_count, 0);

        let settings = load_all_settings(&conn).unwrap();
        let secret = settings
            .into_iter()
            .find(|setting| setting.key == "claude_api_key")
            .map(|setting| setting.value);
        assert_eq!(secret, Some("secret-value".to_string()));
    }

    #[test]
    fn migrates_legacy_plaintext_secrets_on_load() {
        let conn = init_db_in_memory().expect("init db");
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES ('stripe_api_key', 'legacy-secret')",
            [],
        )
        .unwrap();

        let settings = load_all_settings(&conn).unwrap();
        let secret = settings
            .iter()
            .find(|setting| setting.key == "stripe_api_key")
            .map(|setting| setting.value.clone());
        assert_eq!(secret, Some("legacy-secret".to_string()));

        let db_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM app_settings WHERE key = 'stripe_api_key'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(db_count, 0);
    }
}
