use crate::{
    error::ApiError,
    models::{Listing, User},
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod routes;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Register {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignIn {
    pub email: String,
    pub password: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReqId {
    pub id: Uuid
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReqListing {
    pub title: String,
}


impl From<ReqListing> for Listing {
    fn from(list: ReqListing) -> Self {
        Self {
            boxes: vec![],
            id: Uuid::new_v4(),
            title: list.title,
            created_at: Utc::now().naive_utc(),
            box_count: 0,
        }
    }
}
impl TryFrom<Register> for User {
    type Error = ApiError;
    fn try_from(user: Register) -> Result<Self, Self::Error> {
        let hash_pass = hash(user.password, DEFAULT_COST)?;
        let created_at = Utc::now().naive_utc();
        Ok(Self {
            username: user.username,
            email: user.email,
            id: Uuid::new_v4(),
            password: hash_pass,
            created_at: created_at,
            owned_products: vec![],
            points: 0,
            is_superuser: false,
        })
    }
}
