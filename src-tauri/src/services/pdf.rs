use handlebars::Handlebars;
use rusqlite::Connection;
use serde::Serialize;

use crate::db::{clients, invoices};
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize)]
struct InvoiceTemplateData {
    business_name: String,
    business_email: String,
    business_address: String,
    invoice_number: String,
    client_name: String,
    client_company: String,
    client_email: String,
    client_address: String,
    issue_date: String,
    due_date: String,
    status: String,
    line_items: Vec<LineItemData>,
    subtotal: String,
    tax_rate: Option<f64>,
    tax_amount: String,
    total: String,
    notes: Option<String>,
    payment_link: Option<String>,
}

#[derive(Debug, Serialize)]
struct LineItemData {
    description: String,
    quantity: String,
    unit_price: String,
    amount: String,
}

fn format_money(amount: f64) -> String {
    format!("{:.2}", amount)
}

fn format_date_short(date_str: &str) -> String {
    // Parse ISO date and format nicely
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        dt.format("%b %d, %Y").to_string()
    } else {
        date_str.to_string()
    }
}

pub fn render_invoice_html(
    conn: &Connection,
    invoice_id: &str,
    business_name: &str,
    business_email: &str,
    business_address: &str,
) -> AppResult<String> {
    let invoice = invoices::get_invoice(conn, invoice_id)?;
    let client = clients::get_client(conn, &invoice.client_id)?;
    let line_items = invoices::get_line_items(conn, invoice_id)?;

    let template_str = include_str!("../../templates/invoice.html");
    let mut hbs = Handlebars::new();
    hbs.register_template_string("invoice", template_str)
        .map_err(|e| AppError::Template(handlebars::RenderError::from(e)))?;

    let data = InvoiceTemplateData {
        business_name: business_name.to_string(),
        business_email: business_email.to_string(),
        business_address: business_address.to_string(),
        invoice_number: invoice.invoice_number,
        client_name: client.name,
        client_company: client.company.unwrap_or_default(),
        client_email: client.email.unwrap_or_default(),
        client_address: client.address.unwrap_or_default(),
        issue_date: format_date_short(&invoice.issue_date.to_rfc3339()),
        due_date: format_date_short(&invoice.due_date.to_rfc3339()),
        status: invoice.status.as_str().to_string(),
        line_items: line_items
            .into_iter()
            .map(|li| LineItemData {
                description: li.description,
                quantity: format!("{}", li.quantity),
                unit_price: format_money(li.unit_price),
                amount: format_money(li.amount),
            })
            .collect(),
        subtotal: format_money(invoice.subtotal),
        tax_rate: invoice.tax_rate,
        tax_amount: format_money(invoice.tax_amount),
        total: format_money(invoice.total),
        notes: invoice.notes,
        payment_link: invoice.payment_link,
    };

    let html = hbs.render("invoice", &data)?;
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, clients as db_clients, invoices as db_invoices};
    use crate::models::CreateClient;

    #[test]
    fn test_render_invoice_html() {
        let conn = db::init_db_in_memory().unwrap();

        let client = db_clients::create_client(
            &conn,
            CreateClient {
                name: "Acme Corp".to_string(),
                email: Some("billing@acme.com".to_string()),
                company: Some("Acme Corporation".to_string()),
                address: Some("123 Main St".to_string()),
                phone: None,
                notes: None,
                hourly_rate: None,
            },
        )
        .unwrap();

        let invoice = db_invoices::create_invoice(
            &conn,
            &client.id,
            "2025-01-15T00:00:00Z",
            "2025-02-15T00:00:00Z",
            Some("Thank you for your business!"),
            Some(10.0),
        )
        .unwrap();

        db_invoices::add_line_item(&conn, &invoice.id, "Web Development", 20.0, 150.0, 0).unwrap();
        db_invoices::add_line_item(&conn, &invoice.id, "Design Work", 8.0, 120.0, 1).unwrap();

        let html = render_invoice_html(
            &conn,
            &invoice.id,
            "My Business",
            "me@business.com",
            "456 Oak Ave",
        )
        .unwrap();

        assert!(html.contains("My Business"));
        assert!(html.contains("Acme Corp"));
        assert!(html.contains("Web Development"));
        assert!(html.contains("Design Work"));
        assert!(html.contains("Thank you for your business!"));
        assert!(html.contains("3960.00")); // subtotal: 3000 + 960
    }
}
