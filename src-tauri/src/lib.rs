mod commands;
mod db;
mod error;
mod models;
mod services;

use std::sync::Mutex;

use rusqlite::Connection;

pub struct DbState(pub Mutex<Connection>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_dir = dirs_next().unwrap_or_else(|| std::path::PathBuf::from("."));
    std::fs::create_dir_all(&app_dir).ok();

    let db_path = app_dir.join("freelanceinvoice.db");
    let conn = db::init_db(db_path.to_str().unwrap_or("freelanceinvoice.db"))
        .expect("Failed to initialize database");

    tauri::Builder::default()
        .manage(DbState(Mutex::new(conn)))
        .invoke_handler(tauri::generate_handler![
            // Clients
            commands::clients::create_client,
            commands::clients::get_client,
            commands::clients::list_clients,
            commands::clients::update_client,
            commands::clients::delete_client,
            // Projects
            commands::projects::create_project,
            commands::projects::list_projects,
            commands::projects::list_projects_by_client,
            commands::projects::update_project,
            commands::projects::delete_project,
            // Timer
            commands::timer::start_timer,
            commands::timer::stop_timer,
            commands::timer::pause_timer,
            commands::timer::resume_timer,
            commands::timer::get_timer_state,
            commands::timer::list_time_entries,
            commands::timer::delete_time_entry,
            commands::timer::create_manual_time_entry,
            commands::timer::update_manual_time_entry,
            // Invoices
            commands::invoices::create_invoice,
            commands::invoices::create_invoice_draft_atomic,
            commands::invoices::list_invoices,
            commands::invoices::update_invoice_status,
            commands::invoices::add_line_item,
            commands::invoices::get_uninvoiced_entries,
            commands::invoices::set_invoice_payment_link,
            commands::invoices::create_stripe_payment_link,
            // Estimates
            commands::estimates::list_estimates,
            commands::estimates::run_ai_estimate,
            // PDF
            commands::pdf::render_invoice_html,
            commands::pdf::export_invoice_pdf,
            // Dashboard
            commands::dashboard::get_dashboard_summary,
            commands::dashboard::get_revenue_by_client,
            commands::dashboard::get_hours_by_project,
            commands::dashboard::get_monthly_revenue,
            // Settings
            commands::settings::set_setting,
            commands::settings::get_all_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn dirs_next() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|base| {
        let current = base.join("com.freelanceinvoice.desktop");
        let legacy = base.join("com.freelanceinvoice.app");
        if current.exists() || !legacy.exists() {
            current
        } else {
            legacy
        }
    })
}
