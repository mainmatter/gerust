use serde::Serialize;
use sqlx::Postgres;
use uuid::Uuid;

/// A user record.
#[derive(Serialize, Debug, Clone)]
pub struct User {
    /// The id of the record.
    pub id: Uuid,
    /// The user's name.
    pub name: String,
}

/// Loads a user based on the passed token.
///
/// If no user exists for the token, [`Option::None`] is returned, otherwise `Option::Some(User)` is returned.
///
/// # Errors
/// Returns [`crate::Error::DbError`] if a general database error occurs.
pub async fn load_with_token(
    token: &str,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Option<User>, crate::Error> {
    Ok(
        sqlx::query_as!(User, "SELECT id, name FROM users WHERE token = $1", token)
            .fetch_optional(executor)
            .await?,
    )
}
