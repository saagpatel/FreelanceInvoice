use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::Estimate;

pub fn save_estimate(
    conn: &Connection,
    project_description: &str,
    conservative_hours: f64,
    realistic_hours: f64,
    optimistic_hours: f64,
    confidence_score: f64,
    risk_flags: &serde_json::Value,
    similar_projects: &serde_json::Value,
    reasoning: Option<&str>,
    raw_response: Option<&str>,
) -> AppResult<Estimate> {
    let id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO estimates (id, project_description, conservative_hours, realistic_hours, optimistic_hours, confidence_score, risk_flags, similar_projects, reasoning, raw_response)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            project_description,
            conservative_hours,
            realistic_hours,
            optimistic_hours,
            confidence_score,
            serde_json::to_string(risk_flags).unwrap_or_default(),
            serde_json::to_string(similar_projects).unwrap_or_default(),
            reasoning,
            raw_response,
        ],
    )?;

    get_estimate(conn, &id)
}

pub fn get_estimate(conn: &Connection, id: &str) -> AppResult<Estimate> {
    conn.query_row(
        "SELECT * FROM estimates WHERE id = ?1",
        params![id],
        |row| {
            let risk_flags_str: String = row.get("risk_flags")?;
            let similar_projects_str: String = row.get("similar_projects")?;
            Ok(Estimate {
                id: row.get("id")?,
                project_description: row.get("project_description")?,
                conservative_hours: row.get("conservative_hours")?,
                realistic_hours: row.get("realistic_hours")?,
                optimistic_hours: row.get("optimistic_hours")?,
                confidence_score: row.get("confidence_score")?,
                risk_flags: serde_json::from_str(&risk_flags_str).unwrap_or_default(),
                similar_projects: serde_json::from_str(&similar_projects_str).unwrap_or_default(),
                reasoning: row.get("reasoning")?,
                raw_response: row.get("raw_response")?,
                created_at: row.get("created_at")?,
            })
        },
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Estimate not found: {id}"))
        }
        _ => AppError::Database(e),
    })
}

pub fn list_estimates(conn: &Connection) -> AppResult<Vec<Estimate>> {
    let mut stmt = conn.prepare("SELECT * FROM estimates ORDER BY created_at DESC")?;
    let estimates = stmt
        .query_map([], |row| {
            let risk_flags_str: String = row.get("risk_flags")?;
            let similar_projects_str: String = row.get("similar_projects")?;
            Ok(Estimate {
                id: row.get("id")?,
                project_description: row.get("project_description")?,
                conservative_hours: row.get("conservative_hours")?,
                realistic_hours: row.get("realistic_hours")?,
                optimistic_hours: row.get("optimistic_hours")?,
                confidence_score: row.get("confidence_score")?,
                risk_flags: serde_json::from_str(&risk_flags_str).unwrap_or_default(),
                similar_projects: serde_json::from_str(&similar_projects_str).unwrap_or_default(),
                reasoning: row.get("reasoning")?,
                raw_response: row.get("raw_response")?,
                created_at: row.get("created_at")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(estimates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db_in_memory;

    #[test]
    fn test_save_and_get_estimate() {
        let conn = init_db_in_memory().unwrap();
        let risk_flags = serde_json::json!(["scope creep", "unclear requirements"]);
        let similar = serde_json::json!([]);

        let estimate = save_estimate(
            &conn,
            "Build a landing page",
            20.0,
            15.0,
            10.0,
            0.75,
            &risk_flags,
            &similar,
            Some("Based on similar web projects"),
            None,
        )
        .unwrap();

        assert_eq!(estimate.realistic_hours, 15.0);
        assert_eq!(estimate.confidence_score, 0.75);

        let fetched = get_estimate(&conn, &estimate.id).unwrap();
        assert_eq!(fetched.project_description, "Build a landing page");
    }

    #[test]
    fn test_list_estimates() {
        let conn = init_db_in_memory().unwrap();
        let empty = serde_json::json!([]);

        save_estimate(
            &conn,
            "Project A",
            10.0,
            8.0,
            5.0,
            0.8,
            &empty,
            &empty,
            None,
            None,
        )
        .unwrap();
        save_estimate(
            &conn,
            "Project B",
            20.0,
            15.0,
            10.0,
            0.6,
            &empty,
            &empty,
            None,
            None,
        )
        .unwrap();

        let estimates = list_estimates(&conn).unwrap();
        assert_eq!(estimates.len(), 2);
    }
}
