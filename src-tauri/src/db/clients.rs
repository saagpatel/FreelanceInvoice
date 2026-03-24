use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Client, CreateClient, UpdateClient};

pub fn create_client(conn: &Connection, input: CreateClient) -> AppResult<Client> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    conn.execute(
        "INSERT INTO clients (id, name, email, company, address, phone, notes, hourly_rate, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            input.name,
            input.email,
            input.company,
            input.address,
            input.phone,
            input.notes,
            input.hourly_rate,
            now.to_rfc3339(),
            now.to_rfc3339(),
        ],
    )?;

    get_client(conn, &id)
}

pub fn get_client(conn: &Connection, id: &str) -> AppResult<Client> {
    conn.query_row("SELECT * FROM clients WHERE id = ?1", params![id], |row| {
        Ok(Client {
            id: row.get("id")?,
            name: row.get("name")?,
            email: row.get("email")?,
            company: row.get("company")?,
            address: row.get("address")?,
            phone: row.get("phone")?,
            notes: row.get("notes")?,
            hourly_rate: row.get("hourly_rate")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    })
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => {
            AppError::NotFound(format!("Client not found: {id}"))
        }
        _ => AppError::Database(e),
    })
}

pub fn list_clients(conn: &Connection) -> AppResult<Vec<Client>> {
    let mut stmt = conn.prepare("SELECT * FROM clients ORDER BY name ASC")?;
    let clients = stmt
        .query_map([], |row| {
            Ok(Client {
                id: row.get("id")?,
                name: row.get("name")?,
                email: row.get("email")?,
                company: row.get("company")?,
                address: row.get("address")?,
                phone: row.get("phone")?,
                notes: row.get("notes")?,
                hourly_rate: row.get("hourly_rate")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(clients)
}

pub fn update_client(conn: &Connection, id: &str, input: UpdateClient) -> AppResult<Client> {
    // Verify client exists
    get_client(conn, id)?;

    let now = Utc::now();

    if let Some(name) = &input.name {
        conn.execute(
            "UPDATE clients SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name, now.to_rfc3339(), id],
        )?;
    }
    if let Some(email) = &input.email {
        conn.execute(
            "UPDATE clients SET email = ?1, updated_at = ?2 WHERE id = ?3",
            params![email, now.to_rfc3339(), id],
        )?;
    }
    if let Some(company) = &input.company {
        conn.execute(
            "UPDATE clients SET company = ?1, updated_at = ?2 WHERE id = ?3",
            params![company, now.to_rfc3339(), id],
        )?;
    }
    if let Some(address) = &input.address {
        conn.execute(
            "UPDATE clients SET address = ?1, updated_at = ?2 WHERE id = ?3",
            params![address, now.to_rfc3339(), id],
        )?;
    }
    if let Some(phone) = &input.phone {
        conn.execute(
            "UPDATE clients SET phone = ?1, updated_at = ?2 WHERE id = ?3",
            params![phone, now.to_rfc3339(), id],
        )?;
    }
    if let Some(notes) = &input.notes {
        conn.execute(
            "UPDATE clients SET notes = ?1, updated_at = ?2 WHERE id = ?3",
            params![notes, now.to_rfc3339(), id],
        )?;
    }
    if let Some(hourly_rate) = &input.hourly_rate {
        conn.execute(
            "UPDATE clients SET hourly_rate = ?1, updated_at = ?2 WHERE id = ?3",
            params![hourly_rate, now.to_rfc3339(), id],
        )?;
    }

    get_client(conn, id)
}

pub fn delete_client(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM clients WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(AppError::NotFound(format!("Client not found: {id}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db_in_memory;

    fn setup() -> Connection {
        init_db_in_memory().expect("Failed to init test DB")
    }

    #[test]
    fn test_create_and_get_client() {
        let conn = setup();
        let client = create_client(
            &conn,
            CreateClient {
                name: "Acme Corp".to_string(),
                email: Some("hello@acme.com".to_string()),
                company: Some("Acme".to_string()),
                address: None,
                phone: None,
                notes: None,
                hourly_rate: Some(150.0),
            },
        )
        .unwrap();

        assert_eq!(client.name, "Acme Corp");
        assert_eq!(client.email, Some("hello@acme.com".to_string()));
        assert_eq!(client.hourly_rate, Some(150.0));

        let fetched = get_client(&conn, &client.id).unwrap();
        assert_eq!(fetched.name, "Acme Corp");
    }

    #[test]
    fn test_list_clients() {
        let conn = setup();
        create_client(
            &conn,
            CreateClient {
                name: "Beta Inc".to_string(),
                email: None,
                company: None,
                address: None,
                phone: None,
                notes: None,
                hourly_rate: None,
            },
        )
        .unwrap();

        create_client(
            &conn,
            CreateClient {
                name: "Alpha LLC".to_string(),
                email: None,
                company: None,
                address: None,
                phone: None,
                notes: None,
                hourly_rate: None,
            },
        )
        .unwrap();

        let clients = list_clients(&conn).unwrap();
        assert_eq!(clients.len(), 2);
        assert_eq!(clients[0].name, "Alpha LLC"); // sorted by name
    }

    #[test]
    fn test_update_client() {
        let conn = setup();
        let client = create_client(
            &conn,
            CreateClient {
                name: "Old Name".to_string(),
                email: None,
                company: None,
                address: None,
                phone: None,
                notes: None,
                hourly_rate: None,
            },
        )
        .unwrap();

        let updated = update_client(
            &conn,
            &client.id,
            UpdateClient {
                name: Some("New Name".to_string()),
                email: Some(Some("new@email.com".to_string())),
                company: None,
                address: None,
                phone: None,
                notes: None,
                hourly_rate: Some(Some(200.0)),
            },
        )
        .unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.email, Some("new@email.com".to_string()));
        assert_eq!(updated.hourly_rate, Some(200.0));
    }

    #[test]
    fn test_delete_client() {
        let conn = setup();
        let client = create_client(
            &conn,
            CreateClient {
                name: "To Delete".to_string(),
                email: None,
                company: None,
                address: None,
                phone: None,
                notes: None,
                hourly_rate: None,
            },
        )
        .unwrap();

        delete_client(&conn, &client.id).unwrap();
        assert!(get_client(&conn, &client.id).is_err());
    }

    #[test]
    fn test_get_nonexistent_client() {
        let conn = setup();
        let result = get_client(&conn, "nonexistent-id");
        assert!(result.is_err());
    }
}
