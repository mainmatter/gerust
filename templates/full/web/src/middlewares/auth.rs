use crate::state::AppState;
use axum::body::Body;
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use {{crate_name}}_db::entities::User;
use tracing::Span;

#[tracing::instrument(skip_all, fields(rejection_reason = tracing::field::Empty))]
pub async fn auth(
    State(app_state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        log_rejection_reason("Missing authorization header");
        return Err(StatusCode::UNAUTHORIZED);
    };

    match sqlx::query_as!(
        User,
        "SELECT id, name FROM users WHERE token = $1",
        auth_header
    )
    .fetch_one(&app_state.db_pool)
    .await
    {
        Ok(current_user) => {
            req.extensions_mut().insert(current_user);
            Ok(next.run(req).await)
        }
        Err(sqlx::Error::RowNotFound) => {
            log_rejection_reason("Unknown user token");
            Err(StatusCode::UNAUTHORIZED)
        }
        Err(e) => {
            tracing::error!(err.msg = %e, error.details = ?e, "Database error");
            log_rejection_reason("Database error");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn log_rejection_reason(msg: &str) {
    Span::current().record("rejection_reason", msg);
}
