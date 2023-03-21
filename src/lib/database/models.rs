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
    pub is_superuser: bool,
    pub address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Listing {
    pub id: Uuid,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub tty: String
}

#[derive(Debug, Clone)]
pub struct Box {
    pub id: Uuid,
    pub price: i32,
    pub listing_id: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct Product {
    pub box_id: Uuid,
    pub title: String,
    pub id: Uuid,
    pub description: String,
    pub level: i32,
    pub status: bool,
    pub created_at: NaiveDateTime,
    pub amount: i32,
    pub image: String,
}




impl From<Box> for models::Box {
    fn from(value: Box) -> Self {
        Self {
            id: value.id,
            price: value.price as u32,
            listing_id: value.listing_id,
            created_at: value.created_at,
            products: vec![],
            total: 0,
            available_products: 0,
        }
    }
}
impl From<Product> for models::Product {
    fn from(value: Product) -> Self {
        Self {
            id: value.id,
            box_id: value.box_id,
            title: value.title,
            description: value.description,
            level: value.level as u32,
            status: value.status,
            created_at: value.created_at,
            amount: value.amount,
            available: value.amount,
            image: value.image
        }
    }
}

impl From<Listing> for models::Listing {
    fn from(value: Listing) -> Self {
        Self {
            boxes: vec![],
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            box_count: 0,
            image: "".to_owned(),
            tty: value.tty
        }
    }
}

impl From<User> for models::ResponseUser {
    fn from(value: User) -> Self {
        Self {
            is_superuser: value.is_superuser,
            username: value.username,
            email: value.email,
            id: value.id,
            created_at: value.created_at,
            owned_products: vec![],
            points: value.points as u32,
            orders: vec![],
            address: value.address
        }
    }
}
