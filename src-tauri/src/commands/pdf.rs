use tauri::State;

use crate::error::AppResult;
use crate::services::pdf;
use crate::DbState;

#[tauri::command]
pub fn render_invoice_html(
    state: State<DbState>,
    invoice_id: String,
    business_name: String,
    business_email: String,
    business_address: String,
) -> AppResult<String> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    pdf::render_invoice_html(
        &conn,
        &invoice_id,
        &business_name,
        &business_email,
        &business_address,
    )
}

#[tauri::command]
pub fn export_invoice_pdf(
    state: State<DbState>,
    invoice_id: String,
    business_name: String,
    business_email: String,
    business_address: String,
) -> AppResult<Vec<u8>> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    pdf::export_invoice_pdf_bytes(
        &conn,
        &invoice_id,
        &business_name,
        &business_email,
        &business_address,
    )
}
