use database::Database;
pub mod web;
pub mod database;
pub mod models;
pub mod error;


#[derive(Debug, Clone)]
pub struct State {
    pub database: Database
}
