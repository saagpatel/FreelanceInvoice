use serde_json::Value;

use crate::error::{AppError, AppResult};

pub struct CreateStripePaymentLinkRequest {
    pub api_key: String,
    pub amount_cents: i64,
    pub invoice_number: String,
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
}

fn validate_payment_link_request(
    input: &CreateStripePaymentLinkRequest,
) -> AppResult<(String, String)> {
    if input.api_key.trim().is_empty() {
        return Err(AppError::Validation(
            "Stripe API key is required".to_string(),
        ));
    }
    if input.amount_cents <= 0 {
        return Err(AppError::Validation(
            "Invoice total must be greater than zero to create a Stripe payment link".to_string(),
        ));
    }

    let success_url = input
        .success_url
        .clone()
        .ok_or_else(|| AppError::Validation("Stripe success URL is required".to_string()))?;
    let cancel_url = input
        .cancel_url
        .clone()
        .ok_or_else(|| AppError::Validation("Stripe cancel URL is required".to_string()))?;
    validate_return_url(&success_url, "success")?;
    validate_return_url(&cancel_url, "cancel")?;

    Ok((success_url, cancel_url))
}

pub async fn create_payment_link(input: CreateStripePaymentLinkRequest) -> AppResult<String> {
    let (success_url, cancel_url) = validate_payment_link_request(&input)?;
    let amount_cents = input.amount_cents.to_string();
    let product_name = format!("Invoice {}", input.invoice_number);

    let form = vec![
        ("mode".to_string(), "payment".to_string()),
        (
            "line_items[0][price_data][currency]".to_string(),
            "usd".to_string(),
        ),
        (
            "line_items[0][price_data][product_data][name]".to_string(),
            product_name,
        ),
        (
            "line_items[0][price_data][unit_amount]".to_string(),
            amount_cents,
        ),
        ("line_items[0][quantity]".to_string(), "1".to_string()),
        ("success_url".to_string(), success_url),
        ("cancel_url".to_string(), cancel_url),
    ];

    let response = reqwest::Client::new()
        .post("https://api.stripe.com/v1/checkout/sessions")
        .bearer_auth(input.api_key.trim())
        .form(&form)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        let message = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|v| {
                v.get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| format!("Stripe request failed with status {}", status.as_u16()));
        return Err(AppError::Validation(message));
    }

    let parsed: Value = serde_json::from_str(&body)?;
    let url = parsed
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Stripe did not return a payment URL".to_string()))?;

    Ok(url.to_string())
}

fn validate_return_url(url: &str, label: &str) -> AppResult<()> {
    let parsed = reqwest::Url::parse(url).map_err(|_| {
        AppError::Validation(format!("Stripe {label} URL must be a valid absolute URL"))
    })?;

    if parsed.scheme() != "https" {
        return Err(AppError::Validation(format!(
            "Stripe {label} URL must use https"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_payment_link_request, validate_return_url, CreateStripePaymentLinkRequest};

    #[test]
    fn rejects_non_https_return_url() {
        let result = validate_return_url("http://example.com/callback", "success");
        assert!(result.is_err());
    }

    #[test]
    fn accepts_https_return_url() {
        let result = validate_return_url("https://example.com/callback", "success");
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_non_positive_invoice_totals_before_requesting_stripe() {
        let result = validate_payment_link_request(&CreateStripePaymentLinkRequest {
            api_key: "sk_test_123".to_string(),
            amount_cents: 0,
            invoice_number: "INV-1001".to_string(),
            success_url: Some("https://example.com/success".to_string()),
            cancel_url: Some("https://example.com/cancel".to_string()),
        });

        let message = result.expect_err("zero-total invoices should be rejected");
        assert!(
            message
                .to_string()
                .contains("Invoice total must be greater than zero"),
            "payment-link creation should fail with a clear validation message"
        );
    }
}
