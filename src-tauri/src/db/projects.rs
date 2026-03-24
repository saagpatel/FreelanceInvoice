use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{CreateProject, Project, ProjectStatus, UpdateProject};

pub fn create_project(conn: &Connection, input: CreateProject) -> AppResult<Project> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let status = input.status.unwrap_or(ProjectStatus::Active);

    conn.execute(
        "INSERT INTO projects (id, client_id, name, description, status, hourly_rate, budget_hours, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            input.client_id,
            input.name,
            input.description,
            status.as_str(),
            input.hourly_rate,
            input.budget_hours,
            now.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )?;

    get_project(conn, &id)
}

fn row_to_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
    let status_str: String = row.get("status")?;
    Ok(Project {
        id: row.get("id")?,
        client_id: row.get("client_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        status: ProjectStatus::from_str(&status_str).unwrap_or(ProjectStatus::Active),
        hourly_rate: row.get("hourly_rate")?,
        budget_hours: row.get("budget_hours")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn get_project(conn: &Connection, id: &str) -> AppResult<Project> {
    conn.query_row("SELECT * FROM projects WHERE id = ?1", params![id], |row| {
        row_to_project(row)
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Project not found: {id}"))
        }
        _ => AppError::Database(e),
    })
}

pub fn list_projects(conn: &Connection, status: Option<&str>) -> AppResult<Vec<Project>> {
    if let Some(status) = status {
        let mut stmt =
            conn.prepare("SELECT * FROM projects WHERE status = ?1 ORDER BY name ASC")?;
        let projects = stmt
            .query_map(params![status], |row| row_to_project(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(projects)
    } else {
        let mut stmt = conn.prepare("SELECT * FROM projects ORDER BY name ASC")?;
        let projects = stmt
            .query_map([], |row| row_to_project(row))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(projects)
    }
}

pub fn list_projects_by_client(conn: &Connection, client_id: &str) -> AppResult<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT * FROM projects WHERE client_id = ?1 ORDER BY name ASC")?;
    let projects = stmt
        .query_map(params![client_id], |row| row_to_project(row))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(projects)
}

pub fn update_project(conn: &Connection, id: &str, input: UpdateProject) -> AppResult<Project> {
    get_project(conn, id)?;
    let now = Utc::now();

    if let Some(name) = &input.name {
        conn.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now.to_rfc3339(), id],
        )?;
    }
    if let Some(desc) = &input.description {
        conn.execute(
            "UPDATE projects SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![desc, now.to_rfc3339(), id],
        )?;
    }
    if let Some(status) = &input.status {
        conn.execute(
            "UPDATE projects SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now.to_rfc3339(), id],
        )?;
    }
    if let Some(rate) = &input.hourly_rate {
        conn.execute(
            "UPDATE projects SET hourly_rate = ?1, updated_at = ?2 WHERE id = ?3",
            params![rate, now.to_rfc3339(), id],
        )?;
    }
    if let Some(hours) = &input.budget_hours {
        conn.execute(
            "UPDATE projects SET budget_hours = ?1, updated_at = ?2 WHERE id = ?3",
            params![hours, now.to_rfc3339(), id],
        )?;
    }

    get_project(conn, id)
}

pub fn delete_project(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("Project not found: {id}")));
    }
    Ok(())
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
                name: "Test Client".to_string(),
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
    fn test_create_and_get_project() {
        let (conn, client_id) = setup();
        let project = create_project(
            &conn,
            CreateProject {
                client_id,
                name: "Website Redesign".to_string(),
                description: Some("Full redesign".to_string()),
                status: None,
                hourly_rate: Some(175.0),
                budget_hours: Some(40.0),
            },
        )
        .unwrap();

        assert_eq!(project.name, "Website Redesign");
        assert_eq!(project.status, ProjectStatus::Active);
        assert_eq!(project.hourly_rate, Some(175.0));
    }

    #[test]
    fn test_list_projects_with_status_filter() {
        let (conn, client_id) = setup();
        create_project(
            &conn,
            CreateProject {
                client_id: client_id.clone(),
                name: "Active Project".to_string(),
                description: None,
                status: Some(ProjectStatus::Active),
                hourly_rate: None,
                budget_hours: None,
            },
        )
        .unwrap();

        create_project(
            &conn,
            CreateProject {
                client_id,
                name: "Completed Project".to_string(),
                description: None,
                status: Some(ProjectStatus::Completed),
                hourly_rate: None,
                budget_hours: None,
            },
        )
        .unwrap();

        let active = list_projects(&conn, Some("active")).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Active Project");

        let all = list_projects(&conn, None).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_update_project() {
        let (conn, client_id) = setup();
        let project = create_project(
            &conn,
            CreateProject {
                client_id,
                name: "Old Name".to_string(),
                description: None,
                status: None,
                hourly_rate: None,
                budget_hours: None,
            },
        )
        .unwrap();

        let updated = update_project(
            &conn,
            &project.id,
            UpdateProject {
                name: Some("New Name".to_string()),
                description: None,
                status: Some(ProjectStatus::Completed),
                hourly_rate: Some(None),
                budget_hours: Some(None),
            },
        )
        .unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.status, ProjectStatus::Completed);
    }

    #[test]
    fn test_delete_project() {
        let (conn, client_id) = setup();
        let project = create_project(
            &conn,
            CreateProject {
                client_id,
                name: "To Delete".to_string(),
                description: None,
                status: None,
                hourly_rate: None,
                budget_hours: None,
            },
        )
        .unwrap();

        delete_project(&conn, &project.id).unwrap();
        assert!(get_project(&conn, &project.id).is_err());
    }
}
