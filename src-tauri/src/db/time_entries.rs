use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{ActiveTimer, TimeEntry, TimerState};

fn row_to_time_entry(row: &rusqlite::Row) -> rusqlite::Result<TimeEntry> {
    Ok(TimeEntry {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        description: row.get("description")?,
        start_time: row.get("start_time")?,
        end_time: row.get("end_time")?,
        duration_secs: row.get("duration_secs")?,
        is_billable: row.get("is_billable")?,
        is_manual: row.get("is_manual")?,
        invoice_id: row.get("invoice_id")?,
        created_at: row.get("created_at")?,
    })
}

pub fn list_time_entries_by_project(
    conn: &Connection,
    project_id: &str,
) -> AppResult<Vec<TimeEntry>> {
    let mut stmt = conn
        .prepare("SELECT * FROM time_entries WHERE project_id = ?1 ORDER BY start_time DESC")?;
    let entries = stmt
        .query_map(params![project_id], |row| row_to_time_entry(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn list_uninvoiced_entries_by_client(
    conn: &Connection,
    client_id: &str,
) -> AppResult<Vec<TimeEntry>> {
    let mut stmt = conn.prepare(
        "SELECT te.* FROM time_entries te
         JOIN projects p ON te.project_id = p.id
         WHERE p.client_id = ?1 AND te.invoice_id IS NULL AND te.is_billable = 1
         ORDER BY te.start_time ASC",
    )?;
    let entries = stmt
        .query_map(params![client_id], |row| row_to_time_entry(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(entries)
}

pub fn create_time_entry_from_timer(
    conn: &Connection,
    project_id: &str,
    description: Option<&str>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration_secs: i64,
) -> AppResult<TimeEntry> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO time_entries (id, project_id, description, start_time, end_time, duration_secs, is_billable, is_manual)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, 0)",
        params![
            id,
            project_id,
            description,
            start_time.to_rfc3339(),
            end_time.to_rfc3339(),
            duration_secs,
        ],
    )?;

    conn.query_row(
        "SELECT * FROM time_entries WHERE id = ?1",
        params![id],
        |row| row_to_time_entry(row),
    )
    .map_err(AppError::Database)
}

pub fn delete_time_entry(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM time_entries WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("Time entry not found: {id}")));
    }
    Ok(())
}

// Timer operations (active_timer singleton)
pub fn start_timer(
    conn: &Connection,
    project_id: &str,
    description: Option<&str>,
) -> AppResult<ActiveTimer> {
    // Check if timer already running
    let existing = get_active_timer(conn)?;
    if existing.is_some() {
        return Err(AppError::Timer(
            "A timer is already running. Stop it first.".to_string(),
        ));
    }

    let now = Utc::now();
    conn.execute(
        "INSERT OR REPLACE INTO active_timer (id, project_id, description, start_time, accumulated_secs, is_paused)
         VALUES (1, ?1, ?2, ?3, 0, 0)",
        params![project_id, description, now.to_rfc3339()],
    )?;

    Ok(ActiveTimer {
        id: 1,
        project_id: project_id.to_string(),
        description: description.map(String::from),
        start_time: now,
        accumulated_secs: 0,
        is_paused: false,
    })
}

pub fn stop_timer(conn: &Connection) -> AppResult<TimeEntry> {
    let timer = get_active_timer(conn)?
        .ok_or_else(|| AppError::Timer("No active timer to stop".to_string()))?;

    let now = Utc::now();
    let elapsed = if timer.is_paused {
        timer.accumulated_secs
    } else {
        timer.accumulated_secs + (now - timer.start_time).num_seconds()
    };

    let entry = create_time_entry_from_timer(
        conn,
        &timer.project_id,
        timer.description.as_deref(),
        timer.start_time,
        now,
        elapsed,
    )?;

    conn.execute("DELETE FROM active_timer WHERE id = 1", [])?;

    Ok(entry)
}

pub fn pause_timer(conn: &Connection) -> AppResult<ActiveTimer> {
    let timer = get_active_timer(conn)?
        .ok_or_else(|| AppError::Timer("No active timer to pause".to_string()))?;

    if timer.is_paused {
        return Err(AppError::Timer("Timer is already paused".to_string()));
    }

    let now = Utc::now();
    let elapsed = timer.accumulated_secs + (now - timer.start_time).num_seconds();

    conn.execute(
        "UPDATE active_timer SET accumulated_secs = ?1, is_paused = 1 WHERE id = 1",
        params![elapsed],
    )?;

    Ok(ActiveTimer {
        accumulated_secs: elapsed,
        is_paused: true,
        ..timer
    })
}

pub fn resume_timer(conn: &Connection) -> AppResult<ActiveTimer> {
    let timer = get_active_timer(conn)?
        .ok_or_else(|| AppError::Timer("No active timer to resume".to_string()))?;

    if !timer.is_paused {
        return Err(AppError::Timer("Timer is not paused".to_string()));
    }

    let now = Utc::now();
    conn.execute(
        "UPDATE active_timer SET start_time = ?1, is_paused = 0 WHERE id = 1",
        params![now.to_rfc3339()],
    )?;

    Ok(ActiveTimer {
        start_time: now,
        is_paused: false,
        ..timer
    })
}

pub fn get_active_timer(conn: &Connection) -> AppResult<Option<ActiveTimer>> {
    let result = conn.query_row("SELECT * FROM active_timer WHERE id = 1", [], |row| {
        Ok(ActiveTimer {
            id: row.get("id")?,
            project_id: row.get("project_id")?,
            description: row.get("description")?,
            start_time: row.get("start_time")?,
            accumulated_secs: row.get("accumulated_secs")?,
            is_paused: row.get("is_paused")?,
        })
    });

    match result {
        Ok(timer) => Ok(Some(timer)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn get_timer_state(conn: &Connection) -> AppResult<TimerState> {
    match get_active_timer(conn)? {
        Some(timer) => {
            let elapsed = if timer.is_paused {
                timer.accumulated_secs
            } else {
                timer.accumulated_secs + (Utc::now() - timer.start_time).num_seconds()
            };

            // Get project name
            let project_name: Option<String> = conn
                .query_row(
                    "SELECT name FROM projects WHERE id = ?1",
                    params![timer.project_id],
                    |row| row.get(0),
                )
                .ok();

            Ok(TimerState {
                is_running: true,
                is_paused: timer.is_paused,
                project_id: Some(timer.project_id),
                project_name,
                description: timer.description,
                elapsed_secs: elapsed,
                start_time: Some(timer.start_time),
            })
        }
        None => Ok(TimerState {
            is_running: false,
            is_paused: false,
            project_id: None,
            project_name: None,
            description: None,
            elapsed_secs: 0,
            start_time: None,
        }),
    }
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
        let project = create_project(
            &conn,
            CreateProject {
                client_id: client.id,
                name: "Test Project".to_string(),
                description: None,
                status: None,
                hourly_rate: None,
                budget_hours: None,
            },
        )
        .unwrap();
        (conn, project.id)
    }

    #[test]
    fn test_start_and_stop_timer() {
        let (conn, project_id) = setup();

        start_timer(&conn, &project_id, Some("Working on feature")).unwrap();

        let state = get_timer_state(&conn).unwrap();
        assert!(state.is_running);
        assert!(!state.is_paused);
        assert_eq!(state.project_name, Some("Test Project".to_string()));

        let entry = stop_timer(&conn).unwrap();
        assert_eq!(entry.project_id, project_id);
        assert!(!entry.is_manual);

        let state = get_timer_state(&conn).unwrap();
        assert!(!state.is_running);
    }

    #[test]
    fn test_pause_and_resume_timer() {
        let (conn, project_id) = setup();

        start_timer(&conn, &project_id, None).unwrap();
        let paused = pause_timer(&conn).unwrap();
        assert!(paused.is_paused);

        let state = get_timer_state(&conn).unwrap();
        assert!(state.is_paused);

        let resumed = resume_timer(&conn).unwrap();
        assert!(!resumed.is_paused);
    }

    #[test]
    fn test_cannot_start_two_timers() {
        let (conn, project_id) = setup();

        start_timer(&conn, &project_id, None).unwrap();
        let result = start_timer(&conn, &project_id, None);
        assert!(result.is_err());
    }

}
