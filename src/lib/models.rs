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
    pub available_products: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Level {
    ALevel,
    BLevel,
    CLevel,
    DLevel,
    ELevel,
    FLevel,
    GLevel,
    HLevel,
    ILevel,
    JLevel,
    KLevel,
    LLevel,
    MLevel,
    NLevel,
    OLevel,
    PLevel,
    QLevel,
    RLevel,
    SLevel,
    TLevel,
    ULevel,
    VLevel,
    WLevel,
    XLevel,
    YLevel,
    ZLevel,
    LastLevel,
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductIdent {
    pub id: Uuid,
    pub level: Level,
    pub total: u32,
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Listing {
    pub image: String,
    pub boxes: Vec<Box>,
    pub id: Uuid,
    pub title: String,
    pub created_at: NaiveDateTime,
    pub box_count: u32,
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
    pub amount: i32,
    pub available: i32,
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
    pub is_superuser: bool,
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
            points: value.points,
        }
    }
}

impl From<Product> for ProductIdent {
    fn from(value: Product) -> Self {
        ProductIdent {
            id: value.id,
            level: Level::from(value.level),
            total: value.amount as u32,
            
        }
    }
}
impl From<u32> for Level {
    fn from(value: u32) -> Self {
        match value {
            0 => Level::ALevel,
            1 => Level::BLevel,
            2 => Level::CLevel,
            3 => Level::DLevel,
            4 => Level::ELevel,
            5 => Level::FLevel,
            6 => Level::GLevel,
            7 => Level::HLevel,
            8 => Level::ILevel,
            9 => Level::JLevel,
            10 => Level::KLevel,
            11 => Level::LLevel,
            12 => Level::MLevel,
            13 => Level::NLevel,
            14 => Level::OLevel,
            15 => Level::PLevel,
            16 => Level::QLevel,
            17 => Level::RLevel,
            18 => Level::SLevel,
            19 => Level::TLevel,
            20 => Level::ULevel,
            21 => Level::VLevel,
            22 => Level::WLevel,
            23 => Level::XLevel,
            24 => Level::YLevel,
            25 => Level::ZLevel,
            _ => Level::LastLevel,
        }
    }
}