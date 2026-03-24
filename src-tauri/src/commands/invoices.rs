use tauri::State;

use crate::db::{invoices, time_entries};
use crate::error::AppResult;
use crate::models::{
    CreateInvoiceDraftAtomicInput, Invoice, InvoiceLineItem, InvoiceStatus, TimeEntry,
};
use crate::services::stripe;
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
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    invoices::create_invoice(
        &conn,
        &client_id,
        &issue_date,
        &due_date,
        notes.as_deref(),
        tax_rate,
    )
}

#[tauri::command]
pub fn create_invoice_draft_atomic(
    state: State<DbState>,
    input: CreateInvoiceDraftAtomicInput,
) -> AppResult<Invoice> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    invoices::create_invoice_draft_atomic(&conn, input)
}

#[tauri::command]
pub fn list_invoices(state: State<DbState>, status: Option<String>) -> AppResult<Vec<Invoice>> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    invoices::list_invoices(&conn, status.as_deref())
}

#[tauri::command]
pub fn update_invoice_status(
    state: State<DbState>,
    id: String,
    status: String,
) -> AppResult<Invoice> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
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
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    invoices::add_line_item(
        &conn,
        &invoice_id,
        &description,
        quantity,
        unit_price,
        sort_order,
    )
}

#[tauri::command]
pub fn get_uninvoiced_entries(
    state: State<DbState>,
    client_id: String,
) -> AppResult<Vec<TimeEntry>> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    time_entries::list_uninvoiced_entries_by_client(&conn, &client_id)
}

#[tauri::command]
pub fn set_invoice_payment_link(
    state: State<DbState>,
    invoice_id: String,
    payment_link: Option<String>,
) -> AppResult<Invoice> {
    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    invoices::set_payment_link(&conn, &invoice_id, payment_link.as_deref())
}

#[tauri::command]
pub async fn create_stripe_payment_link(
    state: State<'_, DbState>,
    invoice_id: String,
    stripe_api_key: String,
    success_url: Option<String>,
    cancel_url: Option<String>,
) -> AppResult<Invoice> {
    let (tier, invoice_number, total_amount) = {
        let conn = state.0.lock().map_err(|e| {
            crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
        })?;
        let tier: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'tier'",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "free".to_string());
        let invoice = invoices::get_invoice(&conn, &invoice_id)?;
        (tier, invoice.invoice_number, invoice.total)
    };

    if tier != "premium" {
        return Err(crate::error::AppError::Validation(
            "Stripe payment links require the premium tier".to_string(),
        ));
    }

    let payment_link = stripe::create_payment_link(stripe::CreateStripePaymentLinkRequest {
        api_key: stripe_api_key,
        amount_cents: (total_amount * 100.0).round() as i64,
        invoice_number,
        success_url,
        cancel_url,
    })
    .await?;

    let conn = state.0.lock().map_err(|e| {
        crate::error::AppError::Database(rusqlite::Error::InvalidParameterName(e.to_string()))
    })?;
    invoices::set_payment_link(&conn, &invoice_id, Some(&payment_link))
}
