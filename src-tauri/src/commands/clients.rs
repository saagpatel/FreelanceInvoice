use tauri::State;

use crate::db::clients;
use crate::error::AppResult;
use crate::models::{Client, CreateClient, UpdateClient};
use crate::DbState;

#[tauri::command]
pub fn create_client(state: State<DbState>, input: CreateClient) -> AppResult<Client> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    clients::create_client(&conn, input)
}

#[tauri::command]
pub fn get_client(state: State<DbState>, id: String) -> AppResult<Client> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    clients::get_client(&conn, &id)
}

#[tauri::command]
pub fn list_clients(state: State<DbState>) -> AppResult<Vec<Client>> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    clients::list_clients(&conn)
}

#[tauri::command]
pub fn update_client(state: State<DbState>, id: String, input: UpdateClient) -> AppResult<Client> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    clients::update_client(&conn, &id, input)
}

#[tauri::command]
pub fn delete_client(state: State<DbState>, id: String) -> AppResult<()> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    clients::delete_client(&conn, &id)
}
