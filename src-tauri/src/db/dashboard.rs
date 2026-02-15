use rusqlite::{params, Connection};
use serde::Serialize;

use crate::error::AppResult;

#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub total_revenue: f64,
    pub outstanding_amount: f64,
    pub hours_this_week: f64,
    pub hours_this_month: f64,
    pub active_projects: i32,
    pub pending_invoices: i32,
}

#[derive(Debug, Serialize)]
pub struct RevenueByClient {
    pub client_name: String,
    pub total_revenue: f64,
    pub total_hours: f64,
    pub effective_rate: f64,
}

#[derive(Debug, Serialize)]
pub struct HoursByProject {
    pub project_name: String,
    pub total_hours: f64,
    pub billable_hours: f64,
}

#[derive(Debug, Serialize)]
pub struct MonthlyRevenue {
    pub month: String,
    pub revenue: f64,
}

pub fn get_dashboard_summary(conn: &Connection) -> AppResult<DashboardSummary> {
    let total_revenue: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status = 'paid'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let outstanding_amount: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total), 0) FROM invoices WHERE status IN ('sent', 'overdue')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let hours_this_week: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM time_entries
             WHERE strftime('%W', start_time) = strftime('%W', 'now')
             AND strftime('%Y', start_time) = strftime('%Y', 'now')",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as f64
        / 3600.0;

    let hours_this_month: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_secs), 0) FROM time_entries
             WHERE strftime('%Y-%m', start_time) = strftime('%Y-%m', 'now')",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0) as f64
        / 3600.0;

    let active_projects: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM projects WHERE status = 'active'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let pending_invoices: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM invoices WHERE status IN ('sent', 'overdue')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(DashboardSummary {
        total_revenue,
        outstanding_amount,
        hours_this_week,
        hours_this_month,
        active_projects,
        pending_invoices,
    })
}

pub fn get_revenue_by_client(conn: &Connection) -> AppResult<Vec<RevenueByClient>> {
    let mut stmt = conn.prepare(
        "SELECT c.name,
                COALESCE(SUM(i.total), 0) as revenue,
                COALESCE(SUM(te.total_hours), 0) as hours
         FROM clients c
         LEFT JOIN invoices i ON i.client_id = c.id AND i.status = 'paid'
         LEFT JOIN (
             SELECT p.client_id, SUM(te.duration_secs) / 3600.0 as total_hours
             FROM time_entries te
             JOIN projects p ON p.id = te.project_id
             GROUP BY p.client_id
         ) te ON te.client_id = c.id
         GROUP BY c.id, c.name
         HAVING revenue > 0 OR hours > 0
         ORDER BY revenue DESC",
    )?;

    let results = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let revenue: f64 = row.get(1)?;
            let hours: f64 = row.get(2)?;
            let effective_rate = if hours > 0.0 { revenue / hours } else { 0.0 };
            Ok(RevenueByClient {
                client_name: name,
                total_revenue: revenue,
                total_hours: hours,
                effective_rate,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

pub fn get_hours_by_project(conn: &Connection, days: Option<i32>) -> AppResult<Vec<HoursByProject>> {
    let days = days.unwrap_or(30);
    let mut stmt = conn.prepare(
        "SELECT p.name,
                COALESCE(SUM(te.duration_secs), 0) / 3600.0 as total_hours,
                COALESCE(SUM(CASE WHEN te.is_billable = 1 THEN te.duration_secs ELSE 0 END), 0) / 3600.0 as billable_hours
         FROM projects p
         LEFT JOIN time_entries te ON te.project_id = p.id
             AND te.start_time >= datetime('now', ?1)
         GROUP BY p.id, p.name
         HAVING total_hours > 0
         ORDER BY total_hours DESC",
    )?;

    let modifier = format!("-{days} days");
    let results = stmt
        .query_map(params![modifier], |row| {
            Ok(HoursByProject {
                project_name: row.get(0)?,
                total_hours: row.get(1)?,
                billable_hours: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

pub fn get_monthly_revenue(conn: &Connection, months: Option<i32>) -> AppResult<Vec<MonthlyRevenue>> {
    let months = months.unwrap_or(12);
    let modifier = format!("-{months} months");
    let mut stmt = conn.prepare(
        "SELECT strftime('%Y-%m', issue_date) as month,
                COALESCE(SUM(total), 0) as revenue
         FROM invoices
         WHERE status = 'paid'
             AND issue_date >= datetime('now', ?1)
         GROUP BY month
         ORDER BY month ASC",
    )?;

    let results = stmt
        .query_map(params![modifier], |row| {
            Ok(MonthlyRevenue {
                month: row.get(0)?,
                revenue: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db_in_memory;

    #[test]
    fn test_empty_dashboard_summary() {
        let conn = init_db_in_memory().expect("init db");
        let summary = get_dashboard_summary(&conn).unwrap();
        assert_eq!(summary.total_revenue, 0.0);
        assert_eq!(summary.outstanding_amount, 0.0);
        assert_eq!(summary.active_projects, 0);
        assert_eq!(summary.pending_invoices, 0);
    }

    #[test]
    fn test_dashboard_with_data() {
        let conn = init_db_in_memory().expect("init db");

        // Create a client and project
        conn.execute(
            "INSERT INTO clients (id, name) VALUES ('c1', 'Client A')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO projects (id, client_id, name, status) VALUES ('p1', 'c1', 'Project A', 'active')",
            [],
        )
        .unwrap();

        // Create paid and sent invoices
        conn.execute(
            "INSERT INTO invoices (id, invoice_number, client_id, status, issue_date, due_date, total)
             VALUES ('i1', 'INV-001', 'c1', 'paid', datetime('now'), datetime('now', '+30 days'), 1500.0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO invoices (id, invoice_number, client_id, status, issue_date, due_date, total)
             VALUES ('i2', 'INV-002', 'c1', 'sent', datetime('now'), datetime('now', '+30 days'), 500.0)",
            [],
        )
        .unwrap();

        let summary = get_dashboard_summary(&conn).unwrap();
        assert_eq!(summary.total_revenue, 1500.0);
        assert_eq!(summary.outstanding_amount, 500.0);
        assert_eq!(summary.active_projects, 1);
        assert_eq!(summary.pending_invoices, 1);
    }

    #[test]
    fn test_revenue_by_client() {
        let conn = init_db_in_memory().expect("init db");
        conn.execute(
            "INSERT INTO clients (id, name) VALUES ('c1', 'Client A')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO invoices (id, invoice_number, client_id, status, issue_date, due_date, total)
             VALUES ('i1', 'INV-001', 'c1', 'paid', datetime('now'), datetime('now', '+30 days'), 2000.0)",
            [],
        )
        .unwrap();

        let revenue = get_revenue_by_client(&conn).unwrap();
        assert_eq!(revenue.len(), 1);
        assert_eq!(revenue[0].client_name, "Client A");
        assert_eq!(revenue[0].total_revenue, 2000.0);
    }
}
