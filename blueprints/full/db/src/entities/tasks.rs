#[cfg(feature = "test-helpers")]
use fake::{faker::lorem::en::*, Dummy};
use serde::Deserialize;
use serde::Serialize;
use sqlx::Postgres;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Debug, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
}

#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct TaskChangeset {
    #[cfg_attr(feature = "test-helpers", dummy(faker = "Sentence(3..8)"))]
    #[validate(length(min = 1))]
    pub description: String,
}

pub async fn load_all(
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Vec<Task>, anyhow::Error> {
    let tasks = sqlx::query_as!(Task, "SELECT id, description FROM tasks")
        .fetch_all(executor)
        .await?;
    Ok(tasks)
}

pub async fn load(
    id: Uuid,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, anyhow::Error> {
    let task = sqlx::query_as!(Task, "SELECT id, description FROM tasks WHERE id = $1", id)
        .fetch_one(executor)
        .await?;
    Ok(task)
}

pub async fn create(
    task: TaskChangeset,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Task, crate::Error> {
    task.validate().map_err(crate::Error::ValidationError)?;

    let record = sqlx::query!(
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        task.description
    )
    .fetch_one(executor)
    .await
    .map_err(|e| crate::Error::DbError(e.into()))?;

    Ok(Task {
        id: record.id,
        description: task.description,
    })
}
