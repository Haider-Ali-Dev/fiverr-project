use crate::models;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub id: Uuid,
    pub points: i32,
}

#[derive(Debug, Clone)]
pub struct Listing {
    pub id: Uuid,
    pub title: String,
    pub created_at: NaiveDateTime,
}


#[derive(Debug, Clone)]
pub struct Box {
    pub id: Uuid,
    pub price: i32,
    pub listing_id: Uuid,
    pub created_at:  NaiveDateTime
}

impl From<Listing> for models::Listing {
    fn from(value: Listing) -> Self {
        Self {
            boxes: vec![],
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            box_count: 0,
        }
    }
}

impl From<User> for models::ResponseUser {
    fn from(value: User) -> Self {
        Self {
            username: value.username,
            email: value.email,
            id: value.id,
            created_at: value.created_at,
            owned_products: vec![],
            points: value.points as u32,
        }
    }
}
