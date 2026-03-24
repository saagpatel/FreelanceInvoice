use tauri::State;

use crate::db::dashboard;
use crate::error::{AppError, AppResult};
use crate::DbState;

#[tauri::command]
pub fn get_dashboard_summary(state: State<DbState>) -> AppResult<dashboard::DashboardSummary> {
    let conn = state
        .0
        .lock()
        .map_err(|e| AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string())))?;
    dashboard::get_dashboard_summary(&conn)
}

#[tauri::command]
pub fn get_revenue_by_client(state: State<DbState>) -> AppResult<Vec<dashboard::RevenueByClient>> {
    let conn = state
        .0
        .lock()
        .map_err(|e| AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string())))?;
    dashboard::get_revenue_by_client(&conn)
}

#[tauri::command]
pub fn get_hours_by_project(
    state: State<DbState>,
    days: Option<i32>,
) -> AppResult<Vec<dashboard::HoursByProject>> {
    let conn = state
        .0
        .lock()
        .map_err(|e| AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string())))?;
    dashboard::get_hours_by_project(&conn, days)
}

#[tauri::command]
pub fn get_monthly_revenue(
    state: State<DbState>,
    months: Option<i32>,
) -> AppResult<Vec<dashboard::MonthlyRevenue>> {
    let conn = state
        .0
        .lock()
        .map_err(|e| AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string())))?;
    dashboard::get_monthly_revenue(&conn, months)
}
