use tauri::State;

use crate::db::time_entries;
use crate::error::AppResult;
use crate::models::{ActiveTimer, TimeEntry, TimerState};
use crate::DbState;

#[tauri::command]
pub fn start_timer(
    state: State<DbState>,
    project_id: String,
    description: Option<String>,
) -> AppResult<ActiveTimer> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    time_entries::start_timer(&conn, &project_id, description.as_deref())
}

#[tauri::command]
pub fn stop_timer(state: State<DbState>) -> AppResult<TimeEntry> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    time_entries::stop_timer(&conn)
}

#[tauri::command]
pub fn pause_timer(state: State<DbState>) -> AppResult<ActiveTimer> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    time_entries::pause_timer(&conn)
}

#[tauri::command]
pub fn resume_timer(state: State<DbState>) -> AppResult<ActiveTimer> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    time_entries::resume_timer(&conn)
}

#[tauri::command]
pub fn get_timer_state(state: State<DbState>) -> AppResult<TimerState> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    time_entries::get_timer_state(&conn)
}

#[tauri::command]
pub fn list_time_entries(
    state: State<DbState>,
    project_id: String,
) -> AppResult<Vec<TimeEntry>> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    time_entries::list_time_entries_by_project(&conn, &project_id)
}

#[tauri::command]
pub fn delete_time_entry(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    time_entries::delete_time_entry(&conn, &id)
}
