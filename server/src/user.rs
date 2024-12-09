use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub struct User {
    id: String,
    username: Option<String>,
}

impl User {
    pub fn new(username: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            username,
        }
    }
}
