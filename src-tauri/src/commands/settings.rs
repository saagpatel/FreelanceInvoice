use rusqlite::params;
use tauri::State;

use crate::error::AppResult;
use crate::models::AppSetting;
use crate::DbState;

#[tauri::command]
pub fn set_setting(state: State<DbState>, key: String, value: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

#[tauri::command]
pub fn get_all_settings(state: State<DbState>) -> AppResult<Vec<AppSetting>> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    let mut stmt = conn.prepare("SELECT key, value FROM app_settings")?;
    let settings = stmt
        .query_map([], |row| {
            Ok(AppSetting {
                key: row.get(0)?,
                value: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(settings)
}
