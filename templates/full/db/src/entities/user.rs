use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}
