use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Invoice, InvoiceLineItem, InvoiceStatus};

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

    conn.execute(
        "INSERT INTO invoices (id, invoice_number, client_id, status, issue_date, due_date, notes, tax_rate, subtotal, tax_amount, total)
         VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?6, ?7, 0, 0, 0)",
        params![id, invoice_number, client_id, issue_date, due_date, notes, tax_rate],
    )?;

    get_invoice(conn, &id)
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
    let mut stmt = conn
        .prepare("SELECT * FROM invoice_line_items WHERE invoice_id = ?1 ORDER BY sort_order")?;
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
    use crate::models::CreateClient;

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

}
