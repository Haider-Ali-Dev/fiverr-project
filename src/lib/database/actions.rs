use crate::{
    error::ApiError,
    models::{Amount, Box, Listing, ResponseUser, User},
};
use sqlx::pool;
use uuid::Uuid;
pub type Pool = sqlx::Pool<sqlx::postgres::Postgres>;
use crate::database::models::{Box as DBox, Listing as DListing, User as DBUser};
pub struct DatabaseHand;

type DResult<T> = Result<T, ApiError>;

impl DatabaseHand {
    pub async fn create_user(pool: &Pool, user: &User) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let user = user.clone();
        sqlx::query!("INSERT INTO users(username, email, password, id, created_at) VALUES($1, $2, $3, $4, $5)",
         user.username, user.email, user.password, user.id, user.created_at ).execute(&pool).await?;
        Ok(user.into())
    }

    pub async fn get_user(pool: &Pool, id: Uuid) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let mut user: ResponseUser = sqlx::query_as!(
            DBUser,
            "SELECT username, email, id, created_at, points from users WHERE id = $1",
            id.clone()
        )
        .fetch_one(&pool)
        .await?
        .into();

        let points = DatabaseHand::get_user_points(&pool, &user.id).await?;
        user.points = points;
        Ok(user)
    }

    pub async fn get_user_points(pool: &Pool, id: &Uuid) -> DResult<u32> {
        let pool = pool.clone();
        let points = sqlx::query!("SELECT points FROM users WHERE id = $1", id)
            .fetch_one(&pool)
            .await?
            .points;

        Ok(points as u32)
    }

    pub async fn add_coins(pool: &Pool, amount: &Amount) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let existing_coins = DatabaseHand::get_user_points(&pool, &amount.user_id).await?;
        let total = existing_coins + amount.points;
        sqlx::query!(
            "UPDATE users SET points = $1 WHERE id = $2",
            total as i32,
            amount.user_id
        )
        .execute(&pool)
        .await?;

        let user = DatabaseHand::get_user(&pool, amount.user_id).await?;
        Ok(user)
    }

    pub async fn get_listing(pool: &Pool) -> DResult<Vec<Listing>> {
        let mut final_listings: Vec<Listing> = vec![];
        let pool = pool.clone();
        let listings = sqlx::query_as!(DListing, "SELECT * FROM listing")
            .fetch_all(&pool)
            .await?;
        for listing in listings {
            let mut listing: Listing = listing.into();
        }
        Ok(final_listings)
    }

    pub async fn get_boxes_of_listing(pool: &Pool, listing_id: &Uuid) -> DResult<Vec<Box>> {
        let final_boxes = vec![];
        let pool = pool.clone();
        let boxes = sqlx::query_as!(DBox, "SELECT * FROM box WHERE listing_id = $1", listing_id)
            .fetch_all(&pool).await?;

        for b in boxes {
            todo!()
        }
        Ok(final_boxes)
    }
}
