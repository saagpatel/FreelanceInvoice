use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub company: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub hourly_rate: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClient {
    pub name: String,
    pub email: Option<String>,
    pub company: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub hourly_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClient {
    pub name: Option<String>,
    pub email: Option<String>,
    pub company: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
    pub hourly_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Active,
    Completed,
    Archived,
    OnHold,
}

impl ProjectStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Archived => "archived",
            ProjectStatus::OnHold => "on_hold",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(ProjectStatus::Active),
            "completed" => Some(ProjectStatus::Completed),
            "archived" => Some(ProjectStatus::Archived),
            "on_hold" => Some(ProjectStatus::OnHold),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub client_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub hourly_rate: Option<f64>,
    pub budget_hours: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProject {
    pub client_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<ProjectStatus>,
    pub hourly_rate: Option<f64>,
    pub budget_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<ProjectStatus>,
    pub hourly_rate: Option<f64>,
    pub budget_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: String,
    pub project_id: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_secs: i64,
    pub is_billable: bool,
    pub is_manual: bool,
    pub invoice_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Sent,
    Paid,
    Overdue,
    Cancelled,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InvoiceStatus::Draft => "draft",
            InvoiceStatus::Sent => "sent",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Overdue => "overdue",
            InvoiceStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(InvoiceStatus::Draft),
            "sent" => Some(InvoiceStatus::Sent),
            "paid" => Some(InvoiceStatus::Paid),
            "overdue" => Some(InvoiceStatus::Overdue),
            "cancelled" => Some(InvoiceStatus::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub invoice_number: String,
    pub client_id: String,
    pub status: InvoiceStatus,
    pub issue_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub subtotal: f64,
    pub tax_rate: Option<f64>,
    pub tax_amount: f64,
    pub total: f64,
    pub notes: Option<String>,
    pub payment_link: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub id: String,
    pub invoice_id: String,
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub amount: f64,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    pub id: String,
    pub project_description: String,
    pub conservative_hours: f64,
    pub realistic_hours: f64,
    pub optimistic_hours: f64,
    pub confidence_score: f64,
    pub risk_flags: serde_json::Value,
    pub similar_projects: serde_json::Value,
    pub reasoning: Option<String>,
    pub raw_response: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTimer {
    pub id: i32,
    pub project_id: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub accumulated_secs: i64,
    pub is_paused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerState {
    pub is_running: bool,
    pub is_paused: bool,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub description: Option<String>,
    pub elapsed_secs: i64,
    pub start_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
}
