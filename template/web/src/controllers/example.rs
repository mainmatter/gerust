use axum::response::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub hello: String,
}

pub async fn hello() -> Json<Message> {
    Json(Message {
        hello: String::from("world"),
    })
}
