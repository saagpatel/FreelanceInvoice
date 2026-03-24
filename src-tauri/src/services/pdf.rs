use handlebars::Handlebars;
use printpdf::{BuiltinFont, Mm, PdfDocument};
use rusqlite::Connection;
use serde::Serialize;
use std::io::BufWriter;

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

pub fn export_invoice_pdf_bytes(
    conn: &Connection,
    invoice_id: &str,
    business_name: &str,
    business_email: &str,
    business_address: &str,
) -> AppResult<Vec<u8>> {
    let invoice = invoices::get_invoice(conn, invoice_id)?;
    let client = clients::get_client(conn, &invoice.client_id)?;
    let line_items = invoices::get_line_items(conn, invoice_id)?;

    let (doc, page1, layer1) = PdfDocument::new(
        &format!("Invoice {}", invoice.invoice_number),
        Mm(210.0),
        Mm(297.0),
        "Invoice Layer",
    );
    let layer = doc.get_page(page1).get_layer(layer1);
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    let bold_font = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;

    let mut y = 285.0;
    layer.use_text(business_name.to_string(), 18.0, Mm(15.0), Mm(y), &bold_font);
    y -= 8.0;
    if !business_email.trim().is_empty() {
        layer.use_text(business_email.to_string(), 10.0, Mm(15.0), Mm(y), &font);
        y -= 6.0;
    }
    if !business_address.trim().is_empty() {
        for line in business_address.lines() {
            layer.use_text(line.to_string(), 10.0, Mm(15.0), Mm(y), &font);
            y -= 5.0;
        }
    }

    layer.use_text("INVOICE", 20.0, Mm(145.0), Mm(285.0), &bold_font);
    layer.use_text(
        format!("#{}", invoice.invoice_number),
        11.0,
        Mm(145.0),
        Mm(276.0),
        &font,
    );
    layer.use_text(
        format!(
            "Issue: {}",
            format_date_short(&invoice.issue_date.to_rfc3339())
        ),
        10.0,
        Mm(145.0),
        Mm(268.0),
        &font,
    );
    layer.use_text(
        format!("Due: {}", format_date_short(&invoice.due_date.to_rfc3339())),
        10.0,
        Mm(145.0),
        Mm(262.0),
        &font,
    );

    y = 248.0;
    layer.use_text("Bill To", 11.0, Mm(15.0), Mm(y), &bold_font);
    y -= 7.0;
    layer.use_text(client.name, 10.0, Mm(15.0), Mm(y), &font);
    y -= 5.0;
    if let Some(company) = client.company {
        if !company.trim().is_empty() {
            layer.use_text(company, 10.0, Mm(15.0), Mm(y), &font);
            y -= 5.0;
        }
    }
    if let Some(email) = client.email {
        if !email.trim().is_empty() {
            layer.use_text(email, 10.0, Mm(15.0), Mm(y), &font);
            y -= 5.0;
        }
    }
    if let Some(address) = client.address {
        if !address.trim().is_empty() {
            for line in address.lines() {
                layer.use_text(line.to_string(), 10.0, Mm(15.0), Mm(y), &font);
                y -= 5.0;
            }
        }
    }

    let mut row_y = 205.0;
    layer.use_text("Description", 10.5, Mm(15.0), Mm(row_y), &bold_font);
    layer.use_text("Qty", 10.5, Mm(120.0), Mm(row_y), &bold_font);
    layer.use_text("Rate", 10.5, Mm(145.0), Mm(row_y), &bold_font);
    layer.use_text("Amount", 10.5, Mm(170.0), Mm(row_y), &bold_font);
    row_y -= 7.0;

    for line_item in line_items {
        layer.use_text(line_item.description, 10.0, Mm(15.0), Mm(row_y), &font);
        layer.use_text(
            format!("{:.2}", line_item.quantity),
            10.0,
            Mm(120.0),
            Mm(row_y),
            &font,
        );
        layer.use_text(
            format_money(line_item.unit_price),
            10.0,
            Mm(145.0),
            Mm(row_y),
            &font,
        );
        layer.use_text(
            format_money(line_item.amount),
            10.0,
            Mm(170.0),
            Mm(row_y),
            &font,
        );
        row_y -= 6.0;
    }

    let summary_y = 50.0;
    layer.use_text(
        format!("Subtotal: ${}", format_money(invoice.subtotal)),
        11.0,
        Mm(130.0),
        Mm(summary_y + 15.0),
        &font,
    );
    layer.use_text(
        format!("Tax: ${}", format_money(invoice.tax_amount)),
        11.0,
        Mm(130.0),
        Mm(summary_y + 8.0),
        &font,
    );
    layer.use_text(
        format!("Total: ${}", format_money(invoice.total)),
        12.5,
        Mm(130.0),
        Mm(summary_y),
        &bold_font,
    );

    if let Some(notes) = invoice.notes {
        if !notes.trim().is_empty() {
            layer.use_text("Notes:", 10.5, Mm(15.0), Mm(38.0), &bold_font);
            let mut notes_y = 32.0;
            for line in notes.lines().take(4) {
                layer.use_text(line.to_string(), 10.0, Mm(15.0), Mm(notes_y), &font);
                notes_y -= 5.0;
            }
        }
    }

    if let Some(payment_link) = invoice.payment_link {
        if !payment_link.trim().is_empty() {
            layer.use_text(
                format!("Pay online: {}", payment_link),
                9.0,
                Mm(15.0),
                Mm(12.0),
                &font,
            );
        }
    }

    let mut bytes = Vec::new();
    {
        let mut writer = BufWriter::new(&mut bytes);
        doc.save(&mut writer)
            .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    }
    Ok(bytes)
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

    #[test]
    fn test_export_invoice_pdf_bytes() {
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

        let bytes = export_invoice_pdf_bytes(
            &conn,
            &invoice.id,
            "My Business",
            "me@business.com",
            "456 Oak Ave",
        )
        .unwrap();

        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF-"));
    }
}
