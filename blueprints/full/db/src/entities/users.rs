use serde::Serialize;
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

pub async fn load_with_token(token: &str, db: &PgPool) -> Result<Option<User>, anyhow::Error> {
    match sqlx::query_as!(User, "SELECT id, name FROM users WHERE token = $1", token)
        .fetch_one(db)
        .await
    {
        Ok(user) => Ok(Some(user)),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Ok(None),
            _ => Err(error.into()),
        },
    }
}
