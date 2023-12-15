use crate::entities::users::User;
use sqlx::postgres::PgPool;

pub async fn create(name: String, token: String, db: &PgPool) -> Result<User, anyhow::Error> {
    let record = sqlx::query!(
        "INSERT INTO users (name, token) VALUES ($1, $2) RETURNING id",
        name,
        token,
    )
    .fetch_one(db)
    .await?;

    Ok(User {
        id: record.id,
        name,
    })
}
