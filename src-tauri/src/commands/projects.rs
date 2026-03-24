use tauri::State;

use crate::db::projects;
use crate::error::AppResult;
use crate::models::{CreateProject, Project, UpdateProject};
use crate::DbState;

#[tauri::command]
pub fn create_project(state: State<DbState>, input: CreateProject) -> AppResult<Project> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    projects::create_project(&conn, input)
}

#[tauri::command]
pub fn list_projects(state: State<DbState>, status: Option<String>) -> AppResult<Vec<Project>> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    projects::list_projects(&conn, status.as_deref())
}

#[tauri::command]
pub fn list_projects_by_client(
    state: State<DbState>,
    client_id: String,
) -> AppResult<Vec<Project>> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    projects::list_projects_by_client(&conn, &client_id)
}

#[tauri::command]
pub fn update_project(
    state: State<DbState>,
    id: String,
    input: UpdateProject,
) -> AppResult<Project> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    projects::update_project(&conn, &id, input)
}

#[tauri::command]
pub fn delete_project(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    projects::delete_project(&conn, &id)
}
