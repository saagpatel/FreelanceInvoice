use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{
    ActiveTimer, CreateManualTimeEntryInput, TimeEntry, TimerState, UpdateManualTimeEntryInput,
};

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

#[derive(Debug, Clone)]
struct ActiveTimerRecord {
    id: i32,
    project_id: String,
    description: Option<String>,
    started_at: DateTime<Utc>,
    segment_started_at: DateTime<Utc>,
    accumulated_secs: i64,
    is_paused: bool,
}

fn row_to_active_timer_record(row: &rusqlite::Row) -> rusqlite::Result<ActiveTimerRecord> {
    Ok(ActiveTimerRecord {
        id: row.get("id")?,
        project_id: row.get("project_id")?,
        description: row.get("description")?,
        started_at: row.get("started_at")?,
        segment_started_at: row.get("segment_started_at")?,
        accumulated_secs: row.get("accumulated_secs")?,
        is_paused: row.get("is_paused")?,
    })
}

fn get_time_entry(conn: &Connection, id: &str) -> AppResult<TimeEntry> {
    conn.query_row(
        "SELECT * FROM time_entries WHERE id = ?1",
        params![id],
        row_to_time_entry,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Time entry not found: {id}"))
        }
        _ => AppError::Database(e),
    })
}

pub fn list_time_entries_by_project(
    conn: &Connection,
    project_id: &str,
) -> AppResult<Vec<TimeEntry>> {
    let mut stmt =
        conn.prepare("SELECT * FROM time_entries WHERE project_id = ?1 ORDER BY start_time DESC")?;
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

pub fn create_manual_time_entry(
    conn: &Connection,
    input: CreateManualTimeEntryInput,
) -> AppResult<TimeEntry> {
    if input.end_time <= input.start_time {
        return Err(AppError::Validation(
            "Manual entry end time must be after start time".to_string(),
        ));
    }

    let duration_secs = (input.end_time - input.start_time).num_seconds();
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO time_entries (id, project_id, description, start_time, end_time, duration_secs, is_billable, is_manual)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
        params![
            id,
            input.project_id,
            input.description,
            input.start_time.to_rfc3339(),
            input.end_time.to_rfc3339(),
            duration_secs,
            input.is_billable,
        ],
    )?;

    get_time_entry(conn, &id)
}

pub fn update_manual_time_entry(
    conn: &Connection,
    id: &str,
    input: UpdateManualTimeEntryInput,
) -> AppResult<TimeEntry> {
    let existing = get_time_entry(conn, id)?;

    if !existing.is_manual {
        return Err(AppError::Validation(
            "Only manual time entries can be edited".to_string(),
        ));
    }

    if existing.invoice_id.is_some() {
        return Err(AppError::Validation(
            "Invoiced time entries cannot be edited".to_string(),
        ));
    }

    let description = input.description.unwrap_or(existing.description);
    let start_time = input.start_time.unwrap_or(existing.start_time);
    let end_time = input.end_time.unwrap_or(existing.end_time);
    if end_time <= start_time {
        return Err(AppError::Validation(
            "Manual entry end time must be after start time".to_string(),
        ));
    }
    let duration_secs = (end_time - start_time).num_seconds();
    let is_billable = input.is_billable.unwrap_or(existing.is_billable);

    conn.execute(
        "UPDATE time_entries
         SET description = ?1, start_time = ?2, end_time = ?3, duration_secs = ?4, is_billable = ?5
         WHERE id = ?6",
        params![
            description,
            start_time.to_rfc3339(),
            end_time.to_rfc3339(),
            duration_secs,
            is_billable,
            id,
        ],
    )?;

    get_time_entry(conn, id)
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
        "INSERT OR REPLACE INTO active_timer
         (id, project_id, description, start_time, started_at, segment_started_at, accumulated_secs, is_paused)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, 0, 0)",
        params![
            project_id,
            description,
            now.to_rfc3339(),
            now.to_rfc3339(),
            now.to_rfc3339()
        ],
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
    let timer = get_active_timer_record(conn)?
        .ok_or_else(|| AppError::Timer("No active timer to stop".to_string()))?;

    let now = Utc::now();
    let elapsed = if timer.is_paused {
        timer.accumulated_secs
    } else {
        timer.accumulated_secs + (now - timer.segment_started_at).num_seconds()
    };

    let entry = create_time_entry_from_timer(
        conn,
        &timer.project_id,
        timer.description.as_deref(),
        timer.started_at,
        now,
        elapsed,
    )?;

    conn.execute("DELETE FROM active_timer WHERE id = 1", [])?;

    Ok(entry)
}

pub fn pause_timer(conn: &Connection) -> AppResult<ActiveTimer> {
    let timer = get_active_timer_record(conn)?
        .ok_or_else(|| AppError::Timer("No active timer to pause".to_string()))?;

    if timer.is_paused {
        return Err(AppError::Timer("Timer is already paused".to_string()));
    }

    let now = Utc::now();
    let elapsed = timer.accumulated_secs + (now - timer.segment_started_at).num_seconds();

    conn.execute(
        "UPDATE active_timer SET accumulated_secs = ?1, is_paused = 1 WHERE id = 1",
        params![elapsed],
    )?;

    Ok(ActiveTimer {
        id: timer.id,
        project_id: timer.project_id,
        description: timer.description,
        start_time: timer.started_at,
        accumulated_secs: elapsed,
        is_paused: true,
    })
}

pub fn resume_timer(conn: &Connection) -> AppResult<ActiveTimer> {
    let timer = get_active_timer_record(conn)?
        .ok_or_else(|| AppError::Timer("No active timer to resume".to_string()))?;

    if !timer.is_paused {
        return Err(AppError::Timer("Timer is not paused".to_string()));
    }

    let now = Utc::now();
    conn.execute(
        "UPDATE active_timer
         SET segment_started_at = ?1, start_time = ?2, is_paused = 0
         WHERE id = 1",
        params![now.to_rfc3339(), now.to_rfc3339()],
    )?;

    Ok(ActiveTimer {
        id: timer.id,
        project_id: timer.project_id,
        description: timer.description,
        start_time: timer.started_at,
        accumulated_secs: timer.accumulated_secs,
        is_paused: false,
    })
}

fn get_active_timer_record(conn: &Connection) -> AppResult<Option<ActiveTimerRecord>> {
    let result = conn.query_row(
        "SELECT
            id,
            project_id,
            description,
            COALESCE(started_at, start_time) AS started_at,
            COALESCE(segment_started_at, start_time) AS segment_started_at,
            accumulated_secs,
            is_paused
         FROM active_timer
         WHERE id = 1",
        [],
        row_to_active_timer_record,
    );

    match result {
        Ok(timer) => Ok(Some(timer)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn get_active_timer(conn: &Connection) -> AppResult<Option<ActiveTimer>> {
    Ok(get_active_timer_record(conn)?.map(|timer| ActiveTimer {
        id: timer.id,
        project_id: timer.project_id,
        description: timer.description,
        start_time: timer.started_at,
        accumulated_secs: timer.accumulated_secs,
        is_paused: timer.is_paused,
    }))
}

pub fn get_timer_state(conn: &Connection) -> AppResult<TimerState> {
    match get_active_timer_record(conn)? {
        Some(timer) => {
            let elapsed = if timer.is_paused {
                timer.accumulated_secs
            } else {
                timer.accumulated_secs + (Utc::now() - timer.segment_started_at).num_seconds()
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
                start_time: Some(timer.started_at),
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
    use crate::models::{
        CreateClient, CreateManualTimeEntryInput, CreateProject, UpdateManualTimeEntryInput,
    };

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

        let entry = stop_timer(&conn).unwrap();
        assert!(entry.duration_secs >= 0);
    }

    #[test]
    fn test_timer_keeps_original_start_after_resume() {
        let (conn, project_id) = setup();

        let started = start_timer(&conn, &project_id, Some("Deep work")).unwrap();
        let initial_start = started.start_time;

        pause_timer(&conn).unwrap();
        resume_timer(&conn).unwrap();
        let entry = stop_timer(&conn).unwrap();

        assert_eq!(entry.start_time, initial_start);
        assert!(entry.end_time >= entry.start_time);
    }

    #[test]
    fn test_cannot_start_two_timers() {
        let (conn, project_id) = setup();

        start_timer(&conn, &project_id, None).unwrap();
        let result = start_timer(&conn, &project_id, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_and_update_manual_time_entry() {
        let (conn, project_id) = setup();
        let start = Utc::now() - chrono::Duration::hours(2);
        let end = Utc::now() - chrono::Duration::hours(1);

        let entry = create_manual_time_entry(
            &conn,
            CreateManualTimeEntryInput {
                project_id: project_id.clone(),
                description: Some("Manual planning".to_string()),
                start_time: start,
                end_time: end,
                is_billable: true,
            },
        )
        .unwrap();
        assert!(entry.is_manual);
        assert_eq!(entry.project_id, project_id);
        assert_eq!(entry.duration_secs, 3600);

        let updated = update_manual_time_entry(
            &conn,
            &entry.id,
            UpdateManualTimeEntryInput {
                description: Some(Some("Updated description".to_string())),
                start_time: None,
                end_time: Some(end + chrono::Duration::minutes(30)),
                is_billable: Some(false),
            },
        )
        .unwrap();
        assert_eq!(updated.description, Some("Updated description".to_string()));
        assert_eq!(updated.duration_secs, 5400);
        assert!(!updated.is_billable);
    }

    #[test]
    fn test_manual_entry_rejects_invalid_time_range() {
        let (conn, project_id) = setup();
        let start = Utc::now();
        let end = start - chrono::Duration::minutes(5);

        let result = create_manual_time_entry(
            &conn,
            CreateManualTimeEntryInput {
                project_id,
                description: None,
                start_time: start,
                end_time: end,
                is_billable: true,
            },
        );
        assert!(result.is_err());
    }
}
