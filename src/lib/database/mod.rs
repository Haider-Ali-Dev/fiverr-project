pub mod actions;
pub(in crate::database) mod models;

use sqlx::{Postgres, Pool, postgres::PgPoolOptions};

#[derive(Clone, Debug)]
pub struct Database {
    pub pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(uri: &str) -> Self {
        Self {
            pool: PgPoolOptions::new().connect(uri).await.unwrap(),
        }
    }
}