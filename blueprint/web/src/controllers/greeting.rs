use axum::response::Json;
use serde::{Deserialize, Serialize};

/// A greeting to respond with to the requesting client
#[derive(Deserialize, Serialize)]
pub struct Greeting {
    /// Who do we say hello to?
    pub hello: String,
}

/// Responds with a [`Greeting`], encoded as JSON.
pub async fn hello() -> Json<Greeting> {
    Json(Greeting {
        hello: String::from("world"),
    })
}
