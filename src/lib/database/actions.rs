use std::sync::Arc;

use crate::{
    error::ApiError,
    models::{Amount, Box, Listing, Product, ResponseUser, User},
    web::{ImageData, ReqId, SignIn},
};
use uuid::Uuid;
pub type Pool = sqlx::Pool<sqlx::postgres::Postgres>;
use crate::database::models::{
    Box as DBox, Listing as DListing, Product as DProduct, User as DBUser,
};

/// This struct handles all the database queries.
pub struct DatabaseHand;

/// Alias for `Result` which guarantees that `Error` will always be `ApiError`
type DResult<T> = Result<T, ApiError>;

impl DatabaseHand {
    pub async fn create_user(pool: &Pool, user: &User) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let user = user.clone();
        sqlx::query!(
            "INSERT INTO users(username, email, password, id, created_at, points, is_superuser)
        VALUES($1, $2, $3, $4, $5, $6, $7)",
            user.username,
            user.email,
            user.password,
            user.id,
            user.created_at,
            user.points as i32,
            user.is_superuser
        )
        .execute(&pool)
        .await?;
        Ok(user.into())
    }

    pub async fn sign_in(pool: &Pool, signin: &SignIn) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let password = sqlx::query!("SELECT password from users WHERE email = $1", signin.email)
            .fetch_one(&pool)
            .await?
            .password;

        match bcrypt::verify(signin.password.clone(), &password) {
            Ok(true) => DatabaseHand::get_user_email(&pool, &signin.email).await,
            Err(_) | Ok(false) => Err(ApiError::IncorrectPassword(
                bcrypt::BcryptError::InvalidHash("Invalid Password".to_owned()),
            )),
        }
    }

    pub async fn get_user_email(pool: &Pool, email: &str) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let mut user: ResponseUser = sqlx::query_as!(
            DBUser,
            "SELECT username, email, id, created_at, points, is_superuser from users WHERE email = $1",
            email.clone()
        )
        .fetch_one(&pool)
        .await?
        .into();

        let points = DatabaseHand::get_user_points(&pool, &user.id).await?;
        user.points = points;
        Ok(user)
    }
    pub async fn get_user(pool: &Pool, id: Uuid) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let mut user: ResponseUser = sqlx::query_as!(
            DBUser,
            "SELECT username, email, id, created_at, points, is_superuser from users WHERE id = $1",
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

    pub async fn get_image(pool: &Pool, id: &Uuid) -> DResult<String> {
        let pool = pool.clone();
        let image = sqlx::query!("SELECT path FROM images WHERE for_id = $1", id.clone())
            .fetch_one(&pool)
            .await?;
        Ok(image.path)
    }
    pub async fn get_listing(pool: &Pool) -> DResult<Vec<Listing>> {
        let mut final_listings: Vec<Listing> = vec![];
        let pool = pool.clone();
        let listings = sqlx::query_as!(DListing, "SELECT * FROM listing")
            .fetch_all(&pool)
            .await?;
        for listing in listings {
            let mut listing: Listing = listing.into();
            let listing_image = DatabaseHand::get_image(&pool, &listing.id).await?;
            listing.image = listing_image;
            let bxs = DatabaseHand::get_boxes_of_listing(&pool, &listing.id).await?;
            listing.box_count = bxs.len() as u32;
            listing.boxes = bxs;
            final_listings.push(listing);
        }
        Ok(final_listings)
    }

    pub async fn get_boxes_of_listing(pool: &Pool, listing_id: &Uuid) -> DResult<Vec<Box>> {
        let mut final_boxes = vec![];
        let pool = pool.clone();
        let boxes = sqlx::query_as!(DBox, "SELECT * FROM box WHERE listing_id = $1", listing_id)
            .fetch_all(&pool)
            .await?;

        for b in boxes {
            let mut b: Box = b.into();
            let products = DatabaseHand::get_products(&pool, &b.id).await?;
            b.total = products.len() as u32;
            let pro = products
                .iter()
                .filter(|p| !p.status)
                .cloned()
                .collect::<Vec<_>>();
            b.available_products = pro.len() as u32;
            b.products = pro;
            final_boxes.push(b);
        }
        Ok(final_boxes)
    }

    pub async fn get_products(pool: &Pool, box_id: &Uuid) -> DResult<Vec<Product>> {
        let mut final_products = vec![];
        let pool = pool.clone();
        let products =
            sqlx::query_as!(DProduct, "SELECT * FROM products WHERE box_id = $1", box_id)
                .fetch_all(&pool)
                .await?;

        for pro in products {
            let product: Product = pro.into();
            final_products.push(product);
        }

        Ok(final_products)
    }

    async fn confirm_user_privilege(pool: &Pool, id: &ReqId) -> DResult<bool> {
        let pool = pool.clone();
        let is_superuser_rec = sqlx::query!("SELECT is_superuser FROM users WHERE id = $1", id.id)
            .fetch_one(&pool)
            .await?;
        match is_superuser_rec.is_superuser {
            true => Ok(true),
            false => Err(ApiError::NotSuperuser),
        }
    }
    pub async fn create_listing(
        pool: &Pool,
        data: (Listing, ReqId, ImageData),
    ) -> DResult<Vec<Listing>> {
        let pool = pool.clone();
        match DatabaseHand::confirm_user_privilege(&pool, &data.1).await {
            Ok(true) => {
                sqlx::query!(
                    "INSERT INTO listing (title, created_at, id) VALUES($1, $2, $3)",
                    &data.0.title,
                    &data.0.created_at,
                    &data.0.id
                )
                .execute(&pool)
                .await?;
                sqlx::query!(
                    "INSERT INTO images (path, for_id) VALUES($1, $2)",
                    data.2.path,
                    data.2.id
                )
                .execute(&pool)
                .await?;
                let listings = DatabaseHand::get_listing(&pool).await?;
                Ok(listings)
            }
            Ok(false) | Err(_) => Err(ApiError::NotSuperuser),
        }
    }

    pub async fn create_box(pool: &Pool, data: (Box, Vec<Product>, ReqId)) -> DResult<Vec<Box>> {
        let (bx, prods, req_id) = data;
        let pool = pool.clone();
        match DatabaseHand::confirm_user_privilege(&pool, &req_id).await {
            Ok(true) => {
                sqlx::query!(
                    "INSERT INTO box (id, price, listing_id, created_at) VALUES ($1, $2, $3, $4) ",
                    bx.id,
                    bx.price as i32,
                    bx.listing_id,
                    bx.created_at
                )
                .execute(&pool)
                .await?;
                for prod in prods {
                    sqlx::query!(
                        "INSERT INTO products
                    (box_id, title, id, description, level, status, created_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7)",
                    // Remember that prod.box_id is a temporary id so we have 
                    // to use `bx.id`
                        bx.id,
                        prod.title,
                        prod.id,
                        prod.description,
                        prod.level as i32,
                        prod.status,
                        prod.created_at
                    )
                    .execute(&pool)
                    .await?;
                }

                let bxs = DatabaseHand::get_boxes_of_listing(&pool, &bx.listing_id).await?;
                Ok(bxs)
            }

            Ok(false) | Err(_) => Err(ApiError::NotSuperuser),
        }
    }

    // Deletion

    pub async fn get_single_listing(pool: &Pool, listing_id: &Uuid) -> DResult<Listing> {
        let pool = pool.clone();

        let mut listing: Listing =
            sqlx::query_as!(DListing, "SELECT * FROM listing WHERE id = $1", listing_id)
                .fetch_one(&pool)
                .await?
                .into();
        let listing_image = DatabaseHand::get_image(&pool, listing_id).await?;
        listing.image = listing_image;
        let bxs = DatabaseHand::get_boxes_of_listing(&pool, listing_id).await?;
        listing.box_count = bxs.len() as u32;
        listing.boxes = bxs;
        Ok(listing)
    }

    pub async fn delete_box(pool: &Pool, data: (Uuid, ReqId)) -> DResult<Listing> {
        let pool = pool.clone();
        match DatabaseHand::confirm_user_privilege(&pool, &data.1).await {
            Ok(true) => {
                // Delete it's products
                let id = sqlx::query!("SELECT listing_id FROM box WHERE id = $1", &data.0)
                    .fetch_one(&pool)
                    .await?;
                sqlx::query!("DELETE FROM products WHERE box_id = $1", &data.0)
                    .execute(&pool)
                    .await?;

                // Delete Box
                sqlx::query!("DELETE FROM box where id = $1", &data.0)
                    .execute(&pool)
                    .await?;

                let listing = DatabaseHand::get_single_listing(&pool, &id.listing_id).await?;
                Ok(listing)
            }
            Ok(false) | Err(_) => Err(ApiError::NotSuperuser),
        }
    }

    pub async fn delete_listing(pool: &Pool, data: (Uuid, ReqId)) -> DResult<Vec<Listing>> {
        let (listing_id, req_id) = data;
        let pool = pool.clone();
        match DatabaseHand::confirm_user_privilege(&pool, &req_id).await {
            Ok(true) => {
                let box_ids = sqlx::query!("SELECT id FROM box WHERE listing_id = $1", listing_id)
                    .fetch_all(&pool)
                    .await?;
                sqlx::query!("DELETE FROM box WHERE listing_id = $1", listing_id)
                    .execute(&pool)
                    .await?;
                for box_id in box_ids {
                    sqlx::query!("DELETE FROM products WHERE box_id = $1", box_id.id)
                        .execute(&pool)
                        .await?;
                }

                sqlx::query!("DELETE FROM listing WHERE id = $1 ", listing_id)
                    .execute(&pool)
                    .await?;
                let listings = DatabaseHand::get_listing(&pool).await?;
                Ok(listings)
            }
            Ok(false) | Err(_) => Err(ApiError::NotSuperuser),
        }
    }
}
