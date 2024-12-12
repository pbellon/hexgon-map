use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    pub id: String,
    // token: String,
    pub username: String,
    pub color: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct PublicUser {
    id: String,
    username: String,
    color: String,
}

fn string_to_color(input: String) -> (u8, u8, u8) {
    // Hash the input string
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    // Extract RGB values from the hash
    let r = (hash & 0xFF) as u8; // First 8 bits
    let g = ((hash >> 8) & 0xFF) as u8; // Next 8 bits
    let b = ((hash >> 16) & 0xFF) as u8; // Next 8 bits

    (r, g, b)
}

fn color_to_hex(color: (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", color.0, color.1, color.2)
}

impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            username: username.clone(),
            color: color_to_hex(string_to_color(username)),
        }
    }

    pub fn as_public(&self) -> PublicUser {
        PublicUser {
            id: self.id.clone(),
            color: self.color.clone(),
            username: self.username.clone(),
        }
    }
}
