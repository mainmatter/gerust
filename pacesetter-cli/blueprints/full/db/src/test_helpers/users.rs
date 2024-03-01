use crate::entities::users::User;
use fake::{faker::name::en::*, Dummy};
use sqlx::postgres::PgPool;

#[derive(Debug, Clone, Dummy)]
pub struct UserChangeset {
    #[dummy(faker = "Name()")]
    pub name: String,
    #[dummy(faker = "100..101")]
    pub token: String,
}

pub async fn create(user: UserChangeset, db: &PgPool) -> Result<User, anyhow::Error> {
    let record = sqlx::query!(
        "INSERT INTO users (name, token) VALUES ($1, $2) RETURNING id",
        user.name,
        user.token,
    )
    .fetch_one(db)
    .await?;

    Ok(User {
        id: record.id,
        name: user.name,
    })
}
