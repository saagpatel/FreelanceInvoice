use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Connection};
use std::collections::HashSet;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{
    CreateInvoiceDraftAtomicInput, DraftInvoiceLineItemInput, Invoice, InvoiceLineItem,
    InvoiceStatus,
};

fn row_to_invoice(row: &rusqlite::Row) -> rusqlite::Result<Invoice> {
    let status_str: String = row.get("status")?;
    Ok(Invoice {
        id: row.get("id")?,
        invoice_number: row.get("invoice_number")?,
        client_id: row.get("client_id")?,
        status: InvoiceStatus::from_str(&status_str).unwrap_or(InvoiceStatus::Draft),
        issue_date: row.get("issue_date")?,
        due_date: row.get("due_date")?,
        subtotal: row.get("subtotal")?,
        tax_rate: row.get("tax_rate")?,
        tax_amount: row.get("tax_amount")?,
        total: row.get("total")?,
        notes: row.get("notes")?,
        payment_link: row.get("payment_link")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn generate_invoice_number(conn: &Connection) -> AppResult<String> {
    let year = Utc::now().format("%Y").to_string();
    let count: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM invoices WHERE invoice_number LIKE ?1",
            params![format!("INV-{year}-%")],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(format!("INV-{year}-{:03}", count + 1))
}

pub fn create_invoice(
    conn: &Connection,
    client_id: &str,
    issue_date: &str,
    due_date: &str,
    notes: Option<&str>,
    tax_rate: Option<f64>,
) -> AppResult<Invoice> {
    let id = Uuid::new_v4().to_string();
    let invoice_number = generate_invoice_number(conn)?;
    let issue_date = normalize_date_input(issue_date)?;
    let due_date = normalize_date_input(due_date)?;

    conn.execute(
        "INSERT INTO invoices (id, invoice_number, client_id, status, issue_date, due_date, notes, tax_rate, subtotal, tax_amount, total)
         VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, 0, 0, 0)",
        params![id, invoice_number, client_id, issue_date, due_date, notes, tax_rate],
    )?;

    get_invoice(conn, &id)
}

pub fn create_invoice_draft_atomic(
    conn: &Connection,
    input: CreateInvoiceDraftAtomicInput,
) -> AppResult<Invoice> {
    if input.line_items.is_empty() {
        return Err(AppError::Validation(
            "Invoice draft requires at least one line item".to_string(),
        ));
    }

    let invoice_id = Uuid::new_v4().to_string();
    let invoice_number = generate_invoice_number(conn)?;
    let mut subtotal = 0.0_f64;
    let tax_rate = input.tax_rate.unwrap_or(0.0);
    let mut source_entry_ids = HashSet::new();
    let issue_date = normalize_date_input(&input.issue_date)?;
    let due_date = normalize_date_input(&input.due_date)?;

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO invoices (id, invoice_number, client_id, status, issue_date, due_date, notes, tax_rate, subtotal, tax_amount, total)
         VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, 0, 0, 0)",
        params![
            invoice_id,
            invoice_number,
            &input.client_id,
            issue_date,
            due_date,
            input.notes,
            input.tax_rate
        ],
    )?;

    for (index, item) in input.line_items.iter().enumerate() {
        validate_draft_line_item(item, index)?;
        let amount = item.quantity * item.unit_price;
        subtotal += amount;
        tx.execute(
            "INSERT INTO invoice_line_items (id, invoice_id, description, quantity, unit_price, amount, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                Uuid::new_v4().to_string(),
                invoice_id,
                item.description,
                item.quantity,
                item.unit_price,
                amount,
                item.sort_order
            ],
        )?;

        for source_id in &item.source_time_entry_ids {
            if !source_id.trim().is_empty() {
                source_entry_ids.insert(source_id.clone());
            }
        }
    }

    for entry_id in source_entry_ids {
        let valid_entry: Option<String> = tx
            .query_row(
                "SELECT te.id
                 FROM time_entries te
                 JOIN projects p ON p.id = te.project_id
                 WHERE te.id = ?1
                   AND te.invoice_id IS NULL
                   AND te.is_billable = 1
                   AND p.client_id = ?2",
                params![entry_id, &input.client_id],
                |row| row.get(0),
            )
            .ok();

        if valid_entry.is_none() {
            return Err(AppError::Validation(
                "One or more source time entries are invalid, already invoiced, or from another client".to_string(),
            ));
        }

        tx.execute(
            "UPDATE time_entries SET invoice_id = ?1 WHERE id = ?2",
            params![invoice_id, entry_id],
        )?;
    }

    let tax_amount = subtotal * (tax_rate / 100.0);
    let total = subtotal + tax_amount;
    tx.execute(
        "UPDATE invoices
         SET subtotal = ?1, tax_amount = ?2, total = ?3, updated_at = ?4
         WHERE id = ?5",
        params![
            subtotal,
            tax_amount,
            total,
            Utc::now().to_rfc3339(),
            invoice_id
        ],
    )?;

    tx.commit()?;
    get_invoice(conn, &invoice_id)
}

fn normalize_date_input(input: &str) -> AppResult<String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt.with_timezone(&Utc).to_rfc3339());
    }
    if let Ok(date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return Ok(format!("{date}T00:00:00Z"));
    }
    Err(AppError::Validation(format!(
        "Invalid date format: {input}. Use YYYY-MM-DD or RFC3339"
    )))
}

fn validate_draft_line_item(item: &DraftInvoiceLineItemInput, index: usize) -> AppResult<()> {
    if item.description.trim().is_empty() {
        return Err(AppError::Validation(format!(
            "Line item {} requires a description",
            index + 1
        )));
    }
    if item.quantity <= 0.0 {
        return Err(AppError::Validation(format!(
            "Line item {} must have quantity greater than zero",
            index + 1
        )));
    }
    if item.unit_price < 0.0 {
        return Err(AppError::Validation(format!(
            "Line item {} cannot have a negative unit price",
            index + 1
        )));
    }
    Ok(())
}

pub fn get_invoice(conn: &Connection, id: &str) -> AppResult<Invoice> {
    conn.query_row("SELECT * FROM invoices WHERE id = ?1", params![id], |row| {
        row_to_invoice(row)
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Invoice not found: {id}"))
        }
        _ => AppError::Database(e),
    })
}

pub fn list_invoices(conn: &Connection, status: Option<&str>) -> AppResult<Vec<Invoice>> {
    if let Some(status) = status {
        let mut stmt =
            conn.prepare("SELECT * FROM invoices WHERE status = ?1 ORDER BY created_at DESC")?;
        let invoices = stmt
            .query_map(params![status], |row| row_to_invoice(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(invoices)
    } else {
        let mut stmt = conn.prepare("SELECT * FROM invoices ORDER BY created_at DESC")?;
        let invoices = stmt
            .query_map([], |row| row_to_invoice(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(invoices)
    }
}

pub fn update_invoice_status(
    conn: &Connection,
    id: &str,
    status: InvoiceStatus,
) -> AppResult<Invoice> {
    let now = Utc::now();
    let affected = conn.execute(
        "UPDATE invoices SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.as_str(), now.to_rfc3339(), id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("Invoice not found: {id}")));
    }
    get_invoice(conn, id)
}

pub fn update_invoice_totals(conn: &Connection, invoice_id: &str) -> AppResult<Invoice> {
    let subtotal: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM invoice_line_items WHERE invoice_id = ?1",
            params![invoice_id],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let invoice = get_invoice(conn, invoice_id)?;
    let tax_rate = invoice.tax_rate.unwrap_or(0.0);
    let tax_amount = subtotal * (tax_rate / 100.0);
    let total = subtotal + tax_amount;

    conn.execute(
        "UPDATE invoices SET subtotal = ?1, tax_amount = ?2, total = ?3, updated_at = ?4 WHERE id = ?5",
        params![subtotal, tax_amount, total, Utc::now().to_rfc3339(), invoice_id],
    )?;

    get_invoice(conn, invoice_id)
}

pub fn set_payment_link(
    conn: &Connection,
    invoice_id: &str,
    payment_link: Option<&str>,
) -> AppResult<Invoice> {
    if let Some(link) = payment_link {
        let parsed = reqwest::Url::parse(link)
            .map_err(|_| AppError::Validation("Payment link must be a valid URL".to_string()))?;
        if parsed.scheme() != "https" && parsed.scheme() != "http" {
            return Err(AppError::Validation(
                "Payment link must use http or https".to_string(),
            ));
        }
    }

    let affected = conn.execute(
        "UPDATE invoices SET payment_link = ?1, updated_at = ?2 WHERE id = ?3",
        params![payment_link, Utc::now().to_rfc3339(), invoice_id],
    )?;
    if affected == 0 {
        return Err(AppError::NotFound(format!(
            "Invoice not found: {invoice_id}"
        )));
    }
    get_invoice(conn, invoice_id)
}

// Line items
pub fn add_line_item(
    conn: &Connection,
    invoice_id: &str,
    description: &str,
    quantity: f64,
    unit_price: f64,
    sort_order: i32,
) -> AppResult<InvoiceLineItem> {
    let id = Uuid::new_v4().to_string();
    let amount = quantity * unit_price;

    conn.execute(
        "INSERT INTO invoice_line_items (id, invoice_id, description, quantity, unit_price, amount, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, invoice_id, description, quantity, unit_price, amount, sort_order],
    )?;

    update_invoice_totals(conn, invoice_id)?;

    Ok(InvoiceLineItem {
        id,
        invoice_id: invoice_id.to_string(),
        description: description.to_string(),
        quantity,
        unit_price,
        amount,
        sort_order,
    })
}

pub fn get_line_items(conn: &Connection, invoice_id: &str) -> AppResult<Vec<InvoiceLineItem>> {
    let mut stmt =
        conn.prepare("SELECT * FROM invoice_line_items WHERE invoice_id = ?1 ORDER BY sort_order")?;
    let items = stmt
        .query_map(params![invoice_id], |row| {
            Ok(InvoiceLineItem {
                id: row.get("id")?,
                invoice_id: row.get("invoice_id")?,
                description: row.get("description")?,
                quantity: row.get("quantity")?,
                unit_price: row.get("unit_price")?,
                amount: row.get("amount")?,
                sort_order: row.get("sort_order")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::clients::create_client;
    use crate::db::init_db_in_memory;
    use crate::db::projects::create_project;
    use crate::models::{CreateClient, CreateProject};

    fn setup() -> (Connection, String) {
        let conn = init_db_in_memory().expect("Failed to init test DB");
        let client = create_client(
            &conn,
            CreateClient {
                name: "Test".to_string(),
                email: None,
                company: None,
                address: None,
                phone: None,
                notes: None,
                hourly_rate: None,
            },
        )
        .unwrap();
        (conn, client.id)
    }

    fn setup_with_project() -> (Connection, String, String) {
        let (conn, client_id) = setup();
        let project = create_project(
            &conn,
            CreateProject {
                client_id: client_id.clone(),
                name: "Retainer".to_string(),
                description: None,
                status: None,
                hourly_rate: Some(150.0),
                budget_hours: None,
            },
        )
        .unwrap();
        (conn, client_id, project.id)
    }

    #[test]
    fn test_invoice_number_generation() {
        let (conn, _) = setup();
        let num1 = generate_invoice_number(&conn).unwrap();
        let year = Utc::now().format("%Y").to_string();
        assert_eq!(num1, format!("INV-{year}-001"));
    }

    #[test]
    fn test_create_invoice_with_line_items() {
        let (conn, client_id) = setup();
        let invoice = create_invoice(
            &conn,
            &client_id,
            "2025-01-01T00:00:00Z",
            "2025-01-31T00:00:00Z",
            None,
            Some(10.0),
        )
        .unwrap();

        assert_eq!(invoice.status, InvoiceStatus::Draft);
        assert_eq!(invoice.total, 0.0);

        add_line_item(&conn, &invoice.id, "Web Development", 10.0, 150.0, 0).unwrap();
        add_line_item(&conn, &invoice.id, "Design Work", 5.0, 120.0, 1).unwrap();

        let updated = get_invoice(&conn, &invoice.id).unwrap();
        assert_eq!(updated.subtotal, 2100.0); // 1500 + 600
        assert_eq!(updated.tax_amount, 210.0); // 10% of 2100
        assert_eq!(updated.total, 2310.0);

        let items = get_line_items(&conn, &invoice.id).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_invoice_status_transitions() {
        let (conn, client_id) = setup();
        let invoice = create_invoice(
            &conn,
            &client_id,
            "2025-01-01T00:00:00Z",
            "2025-01-31T00:00:00Z",
            None,
            None,
        )
        .unwrap();

        let sent = update_invoice_status(&conn, &invoice.id, InvoiceStatus::Sent).unwrap();
        assert_eq!(sent.status, InvoiceStatus::Sent);

        let paid = update_invoice_status(&conn, &invoice.id, InvoiceStatus::Paid).unwrap();
        assert_eq!(paid.status, InvoiceStatus::Paid);
    }

    #[test]
    fn test_create_atomic_invoice_links_time_entries() {
        let (conn, client_id, project_id) = setup_with_project();
        conn.execute(
            "INSERT INTO time_entries (id, project_id, description, start_time, end_time, duration_secs, is_billable, is_manual)
             VALUES ('te-1', ?1, 'Feature work', datetime('now', '-2 hours'), datetime('now', '-1 hours'), 3600, 1, 0)",
            params![project_id],
        )
        .unwrap();

        let invoice = create_invoice_draft_atomic(
            &conn,
            CreateInvoiceDraftAtomicInput {
                client_id,
                issue_date: "2025-01-01T00:00:00Z".to_string(),
                due_date: "2025-01-31T00:00:00Z".to_string(),
                notes: Some("Atomic save".to_string()),
                tax_rate: Some(10.0),
                line_items: vec![DraftInvoiceLineItemInput {
                    description: "Feature work".to_string(),
                    quantity: 1.0,
                    unit_price: 150.0,
                    sort_order: 0,
                    source_time_entry_ids: vec!["te-1".to_string()],
                }],
            },
        )
        .unwrap();

        assert_eq!(invoice.subtotal, 150.0);
        assert_eq!(invoice.tax_amount, 15.0);
        assert_eq!(invoice.total, 165.0);

        let linked_invoice_id: Option<String> = conn
            .query_row(
                "SELECT invoice_id FROM time_entries WHERE id = 'te-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(linked_invoice_id, Some(invoice.id));
    }

    #[test]
    fn test_atomic_invoice_rolls_back_on_invalid_time_entry() {
        let (conn, client_id, _project_id) = setup_with_project();

        let result = create_invoice_draft_atomic(
            &conn,
            CreateInvoiceDraftAtomicInput {
                client_id,
                issue_date: "2025-01-01T00:00:00Z".to_string(),
                due_date: "2025-01-31T00:00:00Z".to_string(),
                notes: None,
                tax_rate: None,
                line_items: vec![DraftInvoiceLineItemInput {
                    description: "Invalid source".to_string(),
                    quantity: 1.0,
                    unit_price: 100.0,
                    sort_order: 0,
                    source_time_entry_ids: vec!["missing-entry".to_string()],
                }],
            },
        );
        assert!(result.is_err());

        let invoice_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))
            .unwrap();
        assert_eq!(invoice_count, 0);
    }

    #[test]
    fn test_set_and_clear_payment_link() {
        let (conn, client_id) = setup();
        let invoice = create_invoice(
            &conn,
            &client_id,
            "2025-01-01T00:00:00Z",
            "2025-01-31T00:00:00Z",
            None,
            None,
        )
        .unwrap();

        let with_link =
            set_payment_link(&conn, &invoice.id, Some("https://pay.example/link")).unwrap();
        assert_eq!(
            with_link.payment_link,
            Some("https://pay.example/link".to_string())
        );

        let cleared = set_payment_link(&conn, &invoice.id, None).unwrap();
        assert_eq!(cleared.payment_link, None);
    }
}
