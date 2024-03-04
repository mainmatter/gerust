use axum::http::StatusCode;
use std::fmt::{Debug, Display};

/// Helper function to create an internal error response while
/// taking care to log the error itself.
///
/// This is useful to avoid duplication in web endpoints – in the case of unrecoverable errors,
/// there's only really two things to do anyway, which you'll want to do in every such case:
///
/// 1. create an error-level tracing event
/// 2. respond with an axum::http::StatusCode::INTERNAL_SERVER_ERROR status code
///
/// Example
/// ```rust
/// use pacesetter::web::internal_error;
///
/// pub async fn read_all(
///     State(app_state): State<AppState>,
/// ) -> Result<Json<Vec<Task>>, StatusCode> {
///     let tasks = tasks::load_all(&app_state.db_pool)
///         .await
///         .map_err(internal_error)?;
///
///     Ok(Json(tasks))
/// }
/// ```
pub fn internal_error<E>(e: E) -> StatusCode
where
    // Some "error-like" types (e.g. `anyhow::Error`) don't implement the error trait, therefore
    // we "downgrade" to simply requiring `Debug` and `Display`, the traits
    // we actually need for logging purposes.
    E: Debug + Display,
{
    tracing::error!(err.msg = %e, err.details = ?e, "Internal server error");
    // We don't want to leak internal implementation details to the client
    // via the error response, so we just return an opaque internal server.
    StatusCode::INTERNAL_SERVER_ERROR
}
