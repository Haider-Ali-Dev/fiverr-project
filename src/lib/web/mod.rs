use std::{str::FromStr};

use crate::{
    error::ApiError,
    models::{self, Listing, Product, User},
};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Value;
use uuid::Uuid;

pub mod routes;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IdAndReqId {
    pub id: String,
    pub req_id: ReqIdStr,
}

impl From<IdAndReqId> for (Uuid, ReqId) {
    fn from(value: IdAndReqId) -> Self {
        (
            Uuid::from_str(&value.id).unwrap(),
            value.req_id.into()
        )
    }
}

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
    pub id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReqIdStr {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteListing {
    pub listing_id: String,
    pub req_id: ReqIdStr
}


impl From<DeleteListing> for (Uuid, ReqId) {
    fn from(value: DeleteListing) -> Self {
        (
            Uuid::from_str(&value.listing_id).unwrap(),
            value.req_id.into()
        )
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReqListing {
    pub req_id: String,
    pub image: String,
    pub title: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductData {
    pub title: String,
    pub description: String,
    pub level: u32,
    pub amount: i32,
    pub image: String
}

impl From<ProductData> for Product {
    fn from(p: ProductData) -> Self {
        Self {
            available: p.amount,
            amount: p.amount,
            id: Uuid::new_v4(),
            // Temporary Id
            box_id: Uuid::new_v4(),
            title: p.title,
            description: p.description,
            level: p.level,
            status: false,
            created_at: Utc::now().naive_utc(),
            image: p.image,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductCreation {
    pub req_id: ReqIdStr,
    pub product_data: Vec<ProductData>,
    pub box_id: String
}

impl From<ProductCreation> for (Vec<Product>, ReqId, Uuid) {
    fn from(data: ProductCreation) -> Self {
        let mut p_vec = vec![];
        for prod in &data.product_data {
            let prod: Product = prod.clone().into();
            p_vec.push(prod);
        }

        (
            p_vec,
            data.req_id.into(),
            Uuid::from_str(&data.box_id).unwrap()
        )
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoxData {
    pub price: u32,
    pub listing_id: String,
    pub products: Vec<ProductData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxCreation {
    req_id: ReqIdStr,
    box_data: BoxData,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub path: String,
    pub id: Uuid
}

impl From<BoxCreation> for (models::Box, Vec<Product>, ReqId) {
    fn from(data: BoxCreation) -> Self {
        let mut p_vec = vec![];
        let bx = models::Box {
            id: Uuid::new_v4(),
            price: data.box_data.price,
            listing_id: Uuid::from_str(&data.box_data.listing_id).unwrap(),
            created_at: Utc::now().naive_utc(),
            products: vec![],
            total: 0,
            available_products: 0,
        };

        for prod in &data.box_data.products {
            let prod: Product = prod.clone().into();
            p_vec.push(prod);
        }

        (bx, p_vec, data.req_id.into())
    }
}
impl From<ReqIdStr> for ReqId {
    fn from(value: ReqIdStr) -> Self {
        Self {
            id: Uuid::from_str(&value.id).unwrap()
        }
    }
}
impl From<ReqListing> for Listing {
    fn from(list: ReqListing) -> Self {
        Self {
            image: String::new(),
            boxes: vec![],
            id: Uuid::new_v4(),
            title: list.title,
            created_at: Utc::now().naive_utc(),
            box_count: 0,
        }
    }
}

impl From<ReqListing> for ReqId {
    fn from(value: ReqListing) -> Self {
        Self {
            id: Uuid::from_str(&value.req_id).unwrap()
        }
    }
}
impl TryFrom<Register> for User {
    type Error = ApiError;
    fn try_from(user: Register) -> Result<Self, Self::Error> {
        let hash_pass = hash(user.password, DEFAULT_COST)?;
        let created_at = Utc::now().naive_utc();
        Ok(Self {
            private_key: Uuid::new_v4(),
            username: user.username,
            email: user.email,
            id: Uuid::new_v4(),
            password: hash_pass,
            created_at,
            owned_products: vec![],
            points: 0,
            is_superuser: false,
        })
    }
}
