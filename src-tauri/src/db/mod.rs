pub mod clients;
pub mod dashboard;
pub mod estimates;
pub mod invoices;
pub mod projects;
pub mod time_entries;

use rusqlite::Connection;

use crate::error::AppResult;

const MIGRATION_V1: &str = r#"
CREATE TABLE IF NOT EXISTS clients (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT,
    company TEXT,
    address TEXT,
    phone TEXT,
    notes TEXT,
    hourly_rate REAL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY NOT NULL,
    client_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    hourly_rate REAL,
    budget_hours REAL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (client_id) REFERENCES clients(id)
);
CREATE INDEX IF NOT EXISTS idx_projects_client_id ON projects(client_id);
CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);

CREATE TABLE IF NOT EXISTS time_entries (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    description TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    duration_secs INTEGER NOT NULL,
    is_billable INTEGER NOT NULL DEFAULT 1,
    is_manual INTEGER NOT NULL DEFAULT 0,
    invoice_id TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (invoice_id) REFERENCES invoices(id)
);
CREATE INDEX IF NOT EXISTS idx_time_entries_project_id ON time_entries(project_id);
CREATE INDEX IF NOT EXISTS idx_time_entries_invoice_id ON time_entries(invoice_id);

CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY NOT NULL,
    invoice_number TEXT NOT NULL UNIQUE,
    client_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    issue_date TEXT NOT NULL,
    due_date TEXT NOT NULL,
    subtotal REAL NOT NULL DEFAULT 0,
    tax_rate REAL,
    tax_amount REAL NOT NULL DEFAULT 0,
    total REAL NOT NULL DEFAULT 0,
    notes TEXT,
    payment_link TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (client_id) REFERENCES clients(id)
);
CREATE INDEX IF NOT EXISTS idx_invoices_client_id ON invoices(client_id);
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);

CREATE TABLE IF NOT EXISTS invoice_line_items (
    id TEXT PRIMARY KEY NOT NULL,
    invoice_id TEXT NOT NULL,
    description TEXT NOT NULL,
    quantity REAL NOT NULL,
    unit_price REAL NOT NULL,
    amount REAL NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_invoice_line_items_invoice_id ON invoice_line_items(invoice_id);

CREATE TABLE IF NOT EXISTS estimates (
    id TEXT PRIMARY KEY NOT NULL,
    project_description TEXT NOT NULL,
    conservative_hours REAL NOT NULL,
    realistic_hours REAL NOT NULL,
    optimistic_hours REAL NOT NULL,
    confidence_score REAL NOT NULL,
    risk_flags TEXT NOT NULL DEFAULT '[]',
    similar_projects TEXT NOT NULL DEFAULT '[]',
    reasoning TEXT,
    raw_response TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE TABLE IF NOT EXISTS active_timer (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    project_id TEXT NOT NULL,
    description TEXT,
    start_time TEXT NOT NULL,
    accumulated_secs INTEGER NOT NULL DEFAULT 0,
    is_paused INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
"#;

pub fn init_db(db_path: &str) -> AppResult<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn init_db_in_memory() -> AppResult<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> AppResult<()> {
    // Current schema is a single idempotent migration; applying it directly
    // avoids version bookkeeping table maintenance.
    conn.execute_batch(MIGRATION_V1)?;
    ensure_active_timer_columns(conn)?;
    Ok(())
}

fn ensure_active_timer_columns(conn: &Connection) -> AppResult<()> {
    if !column_exists(conn, "active_timer", "started_at")? {
        conn.execute("ALTER TABLE active_timer ADD COLUMN started_at TEXT", [])?;
        conn.execute(
            "UPDATE active_timer SET started_at = start_time WHERE started_at IS NULL",
            [],
        )?;
    }

    if !column_exists(conn, "active_timer", "segment_started_at")? {
        conn.execute(
            "ALTER TABLE active_timer ADD COLUMN segment_started_at TEXT",
            [],
        )?;
        conn.execute(
            "UPDATE active_timer SET segment_started_at = start_time WHERE segment_started_at IS NULL",
            [],
        )?;
    }

    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> AppResult<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(columns.iter().any(|name| name == column))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db_in_memory() {
        let conn = init_db_in_memory().expect("Failed to init DB");

        // Verify all tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"clients".to_string()));
        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"time_entries".to_string()));
        assert!(tables.contains(&"invoices".to_string()));
        assert!(tables.contains(&"invoice_line_items".to_string()));
        assert!(tables.contains(&"estimates".to_string()));
        assert!(tables.contains(&"active_timer".to_string()));
        assert!(tables.contains(&"app_settings".to_string()));
    }

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = init_db_in_memory().expect("Failed to init DB");
        // Running migrations again should not fail
        run_migrations(&conn).expect("Second migration run should succeed");
    }

    #[test]
    fn test_foreign_keys_enabled() {
        let conn = init_db_in_memory().expect("Failed to init DB");
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }
}
