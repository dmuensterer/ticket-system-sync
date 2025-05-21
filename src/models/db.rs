use serde::Serialize;
use sqlx::{Connection, Pool, Row, Sqlite, SqliteConnection, SqlitePool, migrate::MigrateDatabase};
use tracing::{debug, info};

use super::assignment::Assignment;

pub struct DB {
    conn: Pool<Sqlite>,
    path: &'static str,
}

impl DB {
    pub async fn new() -> anyhow::Result<Self> {
        let db_path = "sqlite://database.db";
        Self::create_db(db_path).await.unwrap();
        let conn = SqlitePool::connect(db_path).await.unwrap();

        let db = Self {
            conn,
            path: db_path,
        };

        db.create_table().await?;
        Ok(db)
    }

    async fn create_db(path: &str) -> anyhow::Result<(), String> {
        if !Sqlite::database_exists(path).await.unwrap_or(false) {
            println!("Creating database {}", path);
            match Sqlite::create_database(path).await {
                Ok(_) => println!("Create db success"),
                Err(error) => panic!("error: {}", error),
            }
        } else {
            println!("Database already exists");
        }
        Ok(())
    }

    async fn create_table(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS assignments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                zammad_id INTEGER,
                jira_id INTEGER
            )",
        )
        .execute(&self.conn)
        .await?;
        self.show_all_assignments().await?;
        Ok(())
    }

    pub async fn create_assignment_from_zammad(&self, zammad_id: &i32) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO assignments (zammad_id) VALUES (?)")
            .bind(zammad_id)
            .execute(&self.conn)
            .await?;
        info!("Created assignment with zammad_id: {}", zammad_id);
        Ok(())
    }

    pub async fn add_jira_id_to_assignment(
        &self,
        jira_id: &i32,
        zammad_id: &i32,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE assignments SET jira_id = (?) WHERE zammad_id = (?)")
            .bind(jira_id)
            .bind(zammad_id)
            .execute(&self.conn)
            .await?;
        Ok(())
    }

    pub async fn get_jira_id_by_zammad_id(&self, zammad_id: &i32) -> anyhow::Result<i32> {
        let jira_id = sqlx::query("SELECT * FROM assignments WHERE zammad_id = ?")
            .bind(zammad_id)
            .fetch_one(&self.conn)
            .await?
            .try_get("jira_id")
            .map_err(|e| anyhow::anyhow!("Failed to get jira_id from row: {}", e))?;

        Ok(jira_id)
    }

    pub async fn show_all_assignments(&self) -> anyhow::Result<()> {
        let assignments = match sqlx::query("SELECT * FROM assignments")
            .fetch_all(&self.conn)
            .await
        {
            Ok(rows) => rows,
            Err(e) => return Err(anyhow::anyhow!("Failed to fetch assignments: {}", e)),
        };

        println!("Found {} assignments:", assignments.len());
        for row in assignments {
            // Extract fields from the row using get
            let id: i32 = match row.try_get("id") {
                Ok(id) => id,
                Err(_) => 0,
            };

            let zammad_id: Option<String> = match row.try_get("zammad_id") {
                Ok(id) => id,
                Err(_) => None,
            };

            let jira_id: Option<String> = match row.try_get("jira_id") {
                Ok(id) => id,
                Err(_) => None,
            };

            println!(
                "DB entries: id={}, zammad_id={}, jira_id={}",
                id,
                zammad_id.unwrap_or_else(|| "None".to_string()),
                jira_id.unwrap_or_else(|| "None".to_string())
            );
        }
        Ok(())
    }
}
