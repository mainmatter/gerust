use axum::response::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Greeting {
    pub hello: String,
}

pub async fn hello() -> Json<Greeting> {
    Json(Greeting {
        hello: String::from("world"),
    })
}
