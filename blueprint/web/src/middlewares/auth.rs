use crate::state::AppState;
use axum::body::Body;
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use {{crate_name}}_db::entities::users;
use tracing::Span;use utoipa::openapi::security::SecurityScheme;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "User Token",
                SecurityScheme::ApiKey(utoipa::openapi::security::ApiKey::Header(
                    utoipa::openapi::security::ApiKeyValue::new(
                        http::header::AUTHORIZATION.as_str(),
                    ),
                )),
            )
        }
    }
}

/// Authenticates an incoming request based on an auth token.
///
/// This looks for a token in the `Authorization` header. If no token is present or no user exists with that token (see [`{{crate_name}}_db::entities::users::load_with_token`]), a 401 response code is returned and the request is not processed further.
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

    match users::load_with_token(auth_header, &app_state.db_pool).await {
        Ok(Some(current_user)) => {
            req.extensions_mut().insert(current_user);
            Ok(next.run(req).await)
        }
        Ok(None) => {
            log_rejection_reason("Unknown user token");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(_) => {
            log_rejection_reason("Database error");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn log_rejection_reason(msg: &str) {
    Span::current().record("rejection_reason", msg);
}
