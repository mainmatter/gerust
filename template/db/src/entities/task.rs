use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub description: String,
}
