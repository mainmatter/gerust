use axum::response::Json;
use payloads::*;

pub const OPENAPI_TAG: &str = "Greeting";

/// Responds with a [`HelloResponse`], encoded as JSON.
#[axum::debug_handler]
#[utoipa::path(get, path = "/tasks", tag = OPENAPI_TAG, responses(
    (status = OK, description = "Hello there!", body = HelloResponse)
))]
pub async fn hello() -> Json<HelloResponse> {
    Json(HelloResponse {
        hello: String::from("world"),
    })
}

mod payloads {   
    /// A greeting to respond with to the requesting client
    #[derive(serde::Serialize)]
    #[derive(utoipa::ToSchema)]
    #[derive(Debug)]
    pub struct HelloResponse {
        /// Who do we say hello to?
        pub hello: String,
    }
}
