use tauri::State;

use crate::db::{invoices, time_entries};
use crate::error::AppResult;
use crate::models::{Invoice, InvoiceLineItem, InvoiceStatus, TimeEntry};
use crate::DbState;

#[tauri::command]
pub fn create_invoice(
    state: State<DbState>,
    client_id: String,
    issue_date: String,
    due_date: String,
    notes: Option<String>,
    tax_rate: Option<f64>,
) -> AppResult<Invoice> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    invoices::create_invoice(&conn, &client_id, &issue_date, &due_date, notes.as_deref(), tax_rate)
}

#[tauri::command]
pub fn list_invoices(state: State<DbState>, status: Option<String>) -> AppResult<Vec<Invoice>> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    invoices::list_invoices(&conn, status.as_deref())
}

#[tauri::command]
pub fn update_invoice_status(
    state: State<DbState>,
    id: String,
    status: String,
) -> AppResult<Invoice> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    let status = InvoiceStatus::from_str(&status)
        .ok_or_else(|| crate::error::AppError::Validation(format!("Invalid status: {status}")))?;
    invoices::update_invoice_status(&conn, &id, status)
}

#[tauri::command]
pub fn add_line_item(
    state: State<DbState>,
    invoice_id: String,
    description: String,
    quantity: f64,
    unit_price: f64,
    sort_order: i32,
) -> AppResult<InvoiceLineItem> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    invoices::add_line_item(&conn, &invoice_id, &description, quantity, unit_price, sort_order)
}

#[tauri::command]
pub fn get_uninvoiced_entries(state: State<DbState>, client_id: String) -> AppResult<Vec<TimeEntry>> {
    let conn = state.0.lock().map_err(|e| crate::error::AppError::Database(
        rusqlite::Error::InvalidParameterName(e.to_string()),
    ))?;
    time_entries::list_uninvoiced_entries_by_client(&conn, &client_id)
}
