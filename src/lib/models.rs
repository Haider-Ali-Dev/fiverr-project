
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Box {
    pub id: Uuid,
    pub price: u32,
    pub listing_id: Uuid,
    pub created_at: NaiveDateTime,
    pub products: Vec<Product>,
    pub total: u32,
    pub available_products: u32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Listing {
    pub image: String,
    pub boxes: Vec<Box>,
    pub id: Uuid,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub box_count: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub box_id: Uuid,
    pub title: String,
    pub description: String,
    pub level: u32,
    pub status: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub email: String,
    pub id: Uuid,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub owned_products: Vec<Product>,
    pub points: u32,
    pub is_superuser: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseUser {
    pub is_superuser: bool,
    pub username: String,
    pub email: String,
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub owned_products: Vec<Product>,
    pub points: u32,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amount {
    pub user_id: Uuid,
    pub points: u32,
}



impl From<User> for ResponseUser {
    fn from(value: User) -> Self {
        ResponseUser {
            is_superuser: value.is_superuser,
            username: value.username,
            email: value.email,
            id: value.id,
            created_at: value.created_at,
            owned_products: value.owned_products,
            points: value.points
        }
    }
}
