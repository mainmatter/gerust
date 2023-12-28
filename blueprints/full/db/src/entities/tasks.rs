use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Debug, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
}

#[derive(Deserialize, Validate)]
#[cfg_attr(feature = "test-helpers", derive(Serialize))]
pub struct TaskChangeset {
    #[validate(length(min = 1))]
    pub description: String,
}

pub async fn load_all(db: &crate::DbPool) -> Result<Vec<Task>, anyhow::Error> {
    let tasks = sqlx::query_as!(Task, "SELECT id, description FROM tasks")
        .fetch_all(db)
        .await?;
    Ok(tasks)
}

pub async fn load(id: Uuid, db: &crate::DbPool) -> Result<Task, anyhow::Error> {
    let task = sqlx::query_as!(Task, "SELECT id, description FROM tasks WHERE id = $1", id)
        .fetch_one(db)
        .await?;
    Ok(task)
}

pub async fn create(task: TaskChangeset, db: &crate::DbPool) -> Result<Task, crate::Error> {
    task.validate()
        .map_err(|e| crate::Error::ValidationError(e))?;

    let record = sqlx::query!(
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        task.description
    )
    .fetch_one(db)
    .await
    .map_err(|e| crate::Error::DbError(e.into()))?;

    Ok(Task {
        id: record.id,
        description: task.description,
    })
}
