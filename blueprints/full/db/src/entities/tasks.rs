use serde::Deserialize;
use serde::Serialize;
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Serialize, Debug, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
}

pub async fn load_all(db: &PgPool) -> Result<Vec<Task>, anyhow::Error> {
    let tasks = sqlx::query_as!(Task, "SELECT id, description FROM tasks")
        .fetch_all(db)
        .await?;
    Ok(tasks)
}

pub async fn load(id: Uuid, db: &PgPool) -> Result<Task, anyhow::Error> {
    let task = sqlx::query_as!(Task, "SELECT id, description FROM tasks WHERE id = $1", id)
        .fetch_one(db)
        .await?;
    Ok(task)
}

pub async fn create(description: String, db: &PgPool) -> Result<Task, anyhow::Error> {
    let record = sqlx::query!(
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        description
    )
    .fetch_one(db)
    .await?;

    Ok(Task {
        id: record.id,
        description,
    })
}
