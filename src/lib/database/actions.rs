use crate::{
    error::ApiError,
    models::{Amount, Box, Listing, LogData, Order, Product, ProductIdent, ResponseUser, User, AddressData, Category},
    web::{ImageData, ReqId, SignIn},
};
use chrono::Utc;
use futures::TryFutureExt;
use rand::Rng;
use sqlx::pool;
use std::sync::Arc;
use tower_cookies::Cookies;
use uuid::Uuid;
pub type Pool = sqlx::Pool<sqlx::postgres::Postgres>;
use crate::database::models::{
    Box as DBox, Listing as DListing, Product as DProduct, User as DBUser,
};

const BASE_URL: &str = "http://localhost:3000";

/// This struct handles all the database queries.
pub struct DatabaseHand;

/// Alias for `Result` which guarantees that `Error` will always be `ApiError`
type DResult<T> = Result<T, ApiError>;

impl DatabaseHand {
    pub async fn check_listing_tty(pool: &Pool, id: &Uuid) -> DResult<String> {
        let pool = pool.clone();
        let listing = sqlx::query!("SELECT tty FROM listing WHERE id = $1", id)
            .fetch_one(&pool)
            .await?;
        Ok(listing.tty)
    }
    pub async fn get_user_from_private_key(
        pool: &Pool,
        private_key: &Uuid,
    ) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let user = sqlx::query_as!(
            DBUser,
            "SELECT username, email, id, created_at, points, is_superuser, address  FROM users WHERE private_key = $1",
            private_key
        )
        .fetch_one(&pool)
        .await?;
        let mut user: ResponseUser = user.into();
        user.orders = DatabaseHand::get_orders(&pool, &user.id).await?;
        user.owned_products = DatabaseHand::get_owned_products(&pool, &user.id).await?;
        Ok(user)
    }
    pub async fn create_user(pool: &Pool, user: &User) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let user = user.clone();
        sqlx::query!(
            "INSERT INTO users(username, email, password, id, created_at, points, is_superuser, private_key)
        VALUES($1, $2, $3, $4, $5, $6, $7, $8)",
            user.username,
            user.email,
            user.password,
            user.id,
            user.created_at,
            user.points as i32,
            user.is_superuser,
            user.private_key
        )
        .execute(&pool)
        .await?;
        Ok(user.into())
    }

    pub async fn get_private_key(pool: &Pool, id: &Uuid) -> DResult<Uuid> {
        let pool = pool.clone();
        let private_key = sqlx::query!("SELECT private_key FROM users WHERE id = $1", id)
            .fetch_one(&pool)
            .await?
            .private_key;

        Ok(private_key)
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

    // Get all user's owned product's ids from owned_products table
    pub async fn get_owned_products(pool: &Pool, id: &Uuid) -> DResult<Vec<Uuid>> {
        let pool = pool.clone();
        let owned_products = sqlx::query!(
            "SELECT product_id FROM products_owned WHERE user_id = $1",
            id
        )
        .fetch_all(&pool)
        .await?;
        let mut owned_products_ids: Vec<Uuid> = vec![];
        for product in owned_products {
            owned_products_ids.push(product.product_id);
        }
        Ok(owned_products_ids)
    }

    pub async fn get_users(pool: &Pool) -> DResult<Vec<ResponseUser>> {
        let pool = pool.clone();
        let users = sqlx::query_as!(
            DBUser,
            "SELECT username, email, id, created_at, points, is_superuser, address from users"
        )
        .fetch_all(&pool)
        .await?;
        let mut final_users: Vec<ResponseUser> = vec![];
        for user in users {
            let points = DatabaseHand::get_user_points(&pool, &user.id).await?;
            let mut user: ResponseUser = user.into();
            user.points = points;
            user.owned_products = DatabaseHand::get_owned_products(&pool, &user.id).await?;
            final_users.push(user);
        }
        Ok(final_users)
    }

    pub async fn get_user_email(pool: &Pool, email: &str) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let mut user: ResponseUser = sqlx::query_as!(
            DBUser,
            "SELECT username, email, id, created_at, points, is_superuser, address from users WHERE email = $1",
            email.clone()
        )
        .fetch_one(&pool)
        .await?
        .into();

        let points = DatabaseHand::get_user_points(&pool, &user.id).await?;
        user.points = points;
        user.owned_products = DatabaseHand::get_owned_products(&pool, &user.id).await?;
        user.orders = DatabaseHand::get_orders(&pool, &user.id).await?;
        Ok(user)
    }
    pub async fn get_user(pool: &Pool, id: Uuid) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let mut user: ResponseUser = sqlx::query_as!(
            DBUser,
            "SELECT username, email, id, created_at, points, is_superuser, address from users WHERE id = $1",
            id.clone()
        )
        .fetch_one(&pool)
        .await?
        .into();

        let points = DatabaseHand::get_user_points(&pool, &user.id).await?;
        user.points = points;
        user.orders = DatabaseHand::get_orders(&pool, &user.id).await?;
        user.owned_products = DatabaseHand::get_owned_products(&pool, &user.id).await?;
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
        let image = sqlx::query!("SELECT for_id FROM images WHERE for_id = $1", id.clone())
            .fetch_one(&pool)
            .await?;
        Ok(image.for_id.to_string())
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
            let ed_img = listing_image.split('/').collect::<Vec<_>>();
            listing.image = BASE_URL.to_owned() + "/get/image/" + ed_img.last().unwrap();
            // listing.image = listing_image;
            let bxs = DatabaseHand::get_boxes_of_listing(&pool, &listing.id).await?;
            listing.box_count = bxs.len() as u32;
            listing.boxes = bxs;
            final_listings.push(listing);
        }
        Ok(final_listings)
    }

    pub async fn get_listing_ich(pool: &Pool) -> DResult<Vec<Listing>> {
        let mut final_listings: Vec<Listing> = vec![];
        let pool = pool.clone();
        let listings = sqlx::query_as!(DListing, "SELECT * FROM listing WHERE tty = 'ICH'")
            .fetch_all(&pool)
            .await?;
        for listing in listings {
            let mut listing: Listing = listing.into();
            let listing_image = DatabaseHand::get_image(&pool, &listing.id).await?;
            let ed_img = listing_image.split('/').collect::<Vec<_>>();
            listing.image = BASE_URL.to_owned() + "/get/image/" + ed_img.last().unwrap();
            let bxs = DatabaseHand::get_boxes_of_listing(&pool, &listing.id).await?;
            listing.box_count = bxs.len() as u32;
            listing.boxes = bxs;
            final_listings.push(listing);
        }
        Ok(final_listings)
    }

    pub async fn get_listing_hex(pool: &Pool) -> DResult<Vec<Listing>> {
        let mut final_listings: Vec<Listing> = vec![];
        let pool = pool.clone();
        let listings = sqlx::query_as!(DListing, "SELECT * FROM listing WHERE tty = 'HEX'")
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
                    "INSERT INTO listing (title, created_at, id, tty, description, category_id) VALUES($1, $2, $3, $4, $5, $6)",
                    &data.0.title,
                    &data.0.created_at,
                    &data.0.id,
                    &data.0.tty,
                    &data.0.description,
                    &data.0.category_id
                )
                .execute(&pool)
                .await?;
                sqlx::query!(
                    "INSERT INTO images (path, for_id, extension) VALUES($1, $2, $3)",
                    data.2.path,
                    data.2.id,
                    data.2.ext
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
                    "INSERT INTO box (id, price, listing_id, created_at, original_price) VALUES ($1, $2, $3, $4, $5) ",
                    bx.id,
                    bx.price as i32,
                    bx.listing_id,
                    bx.created_at,
                    bx.original_price as i32
                )
                .execute(&pool)
                .await?;
                for prod in prods {
                    sqlx::query!(
                        "INSERT INTO products
                    (box_id, title, id, description, level, status, created_at, amount, image, ini_amount)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                        // Remember that prod.box_id is a temporary id so we have
                        // to use `bx.id`
                        bx.id,
                        prod.title,
                        prod.id,
                        prod.description,
                        prod.level as i32,
                        prod.status,
                        prod.created_at,
                        prod.amount,
                        prod.image,
                        prod.ini_amount
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
                DatabaseHand::add_log(
                    &pool,
                    LogData {
                        user_id: data.clone().1.id,
                        id: Uuid::new_v4(),
                        created_at: Utc::now().naive_utc(),
                        action: "Box deleted".to_owned(),
                    },
                )
                .await?;
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
                // sqlx::query!("DELETE FROM listi WHERE listing_id = $1", listing_id)
                //     .execute(&pool)
                //     .await?;
                for box_id in box_ids {
                    DatabaseHand::delete_box(&pool, (box_id.id, req_id.clone())).await?;

                    // sqlx::query!("DELETE FROM products WHERE box_id = $1", box_id.id)
                    //     .execute(&pool)
                    //     .await?;
                }

                sqlx::query!("DELETE FROM listing WHERE id = $1 ", listing_id)
                    .execute(&pool)
                    .await?;
                let listings = DatabaseHand::get_listing(&pool).await?;
                DatabaseHand::add_log(
                    &pool,
                    LogData {
                        user_id: req_id.id.clone(),
                        id: Uuid::new_v4(),
                        created_at: Utc::now().naive_utc(),
                        action: "Listing deleted".to_owned(),
                    },
                )
                .await?;
                Ok(listings)
            }
            Ok(false) | Err(_) => Err(ApiError::NotSuperuser),
        }
    }

    // delete_single_product and return all the listings
    pub async fn delete_product(pool: &Pool, data: (Uuid, ReqId)) -> DResult<Vec<Listing>> {
        let (product_id, req_id) = data;
        let pool = pool.clone();
        match DatabaseHand::confirm_user_privilege(&pool, &req_id).await {
            Ok(true) => {
                let box_id = sqlx::query!("SELECT box_id FROM products WHERE id = $1", product_id)
                    .fetch_one(&pool)
                    .await?;
                sqlx::query!("DELETE FROM products WHERE id = $1", product_id)
                    .execute(&pool)
                    .await?;
                let listing_id =
                    sqlx::query!("SELECT listing_id FROM box WHERE id = $1", box_id.box_id)
                        .fetch_one(&pool)
                        .await?;
                let listings = DatabaseHand::get_listing(&pool).await?;
                DatabaseHand::add_log(
                    &pool,
                    LogData {
                        user_id: req_id.id.clone(),
                        id: Uuid::new_v4(),
                        created_at: Utc::now().naive_utc(),
                        action: "Deleted Product".to_owned(),
                    },
                )
                .await?;
                Ok(listings)
            }

            Ok(false) | Err(_) => Err(ApiError::NotSuperuser),
        }
    }

    pub async fn get_box_cost(pool: &Pool, box_id: &Uuid) -> DResult<i32> {
        let pool = pool.clone();
        let cost = sqlx::query!("SELECT price FROM box WHERE id = $1", box_id)
            .fetch_one(&pool)
            .await?;

        Ok(cost.price)
    }

    pub async fn deduct_points_from_user(pool: &Pool, data: (u32, Uuid)) -> DResult<()> {
        let (points, user_id) = data;
        let user_points = DatabaseHand::get_user_points(&pool, &user_id).await?;
        let p = user_points - points;
        let pool = pool.clone();
        sqlx::query!(
            "UPDATE users SET points = $1 WHERE id = $2",
            p as i32,
            user_id
        )
        .execute(&pool)
        .await?;
        Ok(())
    }
    // Confirm user privilege also
    pub async fn add_product_to_box(
        pool: &Pool,
        data: (ReqId, Uuid, Vec<Product>),
    ) -> DResult<Vec<Listing>> {
        let (req_id, box_id, products) = data;
        let pool = pool.clone();
        match DatabaseHand::confirm_user_privilege(&pool, &req_id).await {
            Ok(true) => {
                for product in products {
                    sqlx::query!(
                        "INSERT INTO products
                (box_id, title, id, description, level, status, created_at, amount, image, ini_amount)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                        box_id,
                        product.title,
                        product.id,
                        product.description,
                        product.level as i32,
                        product.status,
                        product.created_at,
                        product.amount,
                        product.image,
                        product.ini_amount
                    )
                    .execute(&pool)
                    .await?;
                }

                let listing = DatabaseHand::get_listing(&pool).await?;
                DatabaseHand::add_log(
                    &pool,
                    LogData {
                        user_id: req_id.id.clone(),
                        id: Uuid::new_v4(),
                        created_at: Utc::now().naive_utc(),
                        action: "Product Added to a box".to_owned(),
                    },
                )
                .await?;
                Ok(listing)
            }
            Ok(false) | Err(_) => Err(ApiError::NotSuperuser),
        }
    }
    pub async fn get_single_product(pool: &Pool, product_id: &Uuid) -> DResult<Product> {
        let pool = pool.clone();
        let product = sqlx::query_as!(DProduct, "SELECT * FROM products WHERE id = $1", product_id)
            .fetch_one(&pool)
            .await?
            .into();
        Ok(product)
    }
    // Get product amount from product's id
    pub async fn get_product_amount(pool: &Pool, product_id: &Uuid) -> DResult<i32> {
        let pool = pool.clone();
        let amount = sqlx::query!("SELECT amount FROM products WHERE id = $1", product_id)
            .fetch_one(&pool)
            .await?;
        Ok(amount.amount)
    }
    pub async fn buy_box(pool: &Pool, data: (Uuid, ReqId)) -> DResult<Product> {
        let (box_id, req_id) = data;
        let pool = pool.clone();
        let points = DatabaseHand::get_user_points(&pool, &req_id.id).await?;
        let cost = DatabaseHand::get_box_cost(&pool, &box_id).await?;

        // Checking if user has enough points
        if points == 0 {
            return Err(ApiError::InsufficientPoints);
        } else {
            if points < cost as u32 {
                return Err(ApiError::InsufficientPoints);
            }
        }

        // Deducting points from user
        DatabaseHand::deduct_points_from_user(&pool, (cost as u32, req_id.id)).await?;

        // Selecting a random product
        let mut products_idents = Vec::new();
        let products = DatabaseHand::get_products(&pool, &box_id)
            .await?
            .into_iter()
            .filter(|prod| prod.status == false)
            .collect::<Vec<Product>>();

        for product in &products {
            let product: ProductIdent = product.clone().into();
            products_idents.push(product);
        }

        let prod = DatabaseHand::select_weighted_random_product(&products_idents);
        match prod {
            Some(prod) => {
                sqlx::query!(
                    "UPDATE products SET amount = amount - 1 WHERE id = $1",
                    &prod.id
                )
                .execute(&pool)
                .await?;
                
                if DatabaseHand::get_product_amount(&pool, &prod.id).await? == 0 {
                    sqlx::query!(
                        "UPDATE products SET status = true WHERE id = $1",
                        &prod.id
                    )
                    .execute(&pool)
                    .await?;
                }

                // Adding the product purchase to products_owned
                let t = Utc::now().naive_utc();
                sqlx::query!(
                    "INSERT INTO products_owned(user_id, product_id, bought_at, id) 
                VALUES($1, $2, $3, $4)",
                    &req_id.id,
                    &prod.id,
                    t,
                    Uuid::new_v4()
                )
                .execute(&pool)
                .await?;

                
                let product = DatabaseHand::get_single_product(&pool, &prod.id).await?;
                let order = Order {
                    id: Uuid::new_v4(),
                    user_id: req_id.id,
                    product_id: prod.id,
                    created_at: t,
                    status: "Pending".to_owned(),
                    product_name: product.clone().title,
                };
                DatabaseHand::add_order(order, &pool).await?;

                Ok(product)
            }
            None => Err(ApiError::SelectionError),
        }
    }

    fn select_weighted_random_product(products: &Vec<ProductIdent>) -> Option<ProductIdent> {
        let total_sum = products.iter().map(|p| p.total).sum();
        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(0..total_sum);
        let mut running_total = 0;
        for product in products {
            running_total += product.total;
            if running_total >= random_number {
                return Some(product.clone());
            }
        }
        None
    }

    // Get image path and extension from id and return as tuple
    pub async fn get_image_p_and_ext(pool: &Pool, id: &Uuid) -> DResult<(String, String)> {
        let pool = pool.clone();
        let image = sqlx::query!(
            "SELECT path, extension FROM images WHERE for_id = $1",
            id
        )
        .fetch_one(&pool)
        .await?;
        Ok((image.path, image.extension))
    }

   


    pub async fn save_image(pool: &Pool, data: ImageData) -> DResult<String> {
        let pool = pool.clone();
        let ImageData { id, path, ext } = data;
        let image = sqlx::query!(
            "INSERT INTO images(path, for_id, extension) VALUES($1, $2, $3) RETURNING path",
            path,
            id,
            ext
        )
        .fetch_one(&pool)
        .await?;

        Ok(image.path.clone())
    }

    // Logging
    pub async fn add_log(pool: &Pool, data: LogData) -> DResult<()> {
        let pool = pool.clone();
        let LogData {
            user_id,
            id,
            created_at,
            action,
        } = data;
        sqlx::query!(
            "INSERT INTO logs(user_id, id, created_at, action) VALUES($1, $2, $3, $4)",
            user_id,
            id,
            created_at,
            action
        )
        .execute(&pool)
        .await?;
        Ok(())
    }

    pub async fn get_logs(pool: &Pool) -> DResult<Vec<LogData>> {
        let pool = pool.clone();
        let logs = sqlx::query_as!(LogData, "SELECT * FROM logs")
            .fetch_all(&pool)
            .await?;
        Ok(logs)
    }

    pub async fn get_listing_from_id(pool: &Pool, id: &Uuid) -> DResult<Listing> {
        let pool = pool.clone();
        let listing = sqlx::query_as!(DListing, "SELECT * FROM listing WHERE id = $1", id)
            .fetch_one(&pool)
            .await?;
        let mut listing: Listing = listing.into();
        let listing_image = DatabaseHand::get_image(&pool, &listing.id).await?;
        let ed_img = listing_image.split('/').collect::<Vec<_>>();
        listing.image = BASE_URL.to_owned() + "/get/image/" + ed_img.last().unwrap();
        let bxs = DatabaseHand::get_boxes_of_listing(&pool, &listing.id).await?;
        listing.box_count = bxs.len() as u32;
        listing.boxes = bxs;
        Ok(listing)
    }

    pub async fn add_order(order: Order, pool: &Pool) -> DResult<()> {
        let pool = pool.clone();
        let Order {
            id,
            user_id,
            created_at,
            status,
            product_id,
            product_name
        } = order;
        sqlx::query!(
            "INSERT INTO order_tracking(id, user_id, product_id, created_at, status, product_name) VALUES($1, $2, $3, $4, $5, $6)",
            id,
            user_id,
            product_id,
            created_at,
            status,
            product_name
        )
        .execute(&pool)
        .await?;
        Ok(())
    }

    // Get user's orders from user_id
    pub async fn get_orders(pool: &Pool, user_id: &Uuid) -> DResult<Vec<Order>> {
        let pool = pool.clone();
        let orders = sqlx::query_as!(
            Order,
            "SELECT * FROM order_tracking WHERE user_id = $1",
            user_id
        )
        .fetch_all(&pool)
        .await?;
        Ok(orders)
    }

    // Get all the orders
    pub async fn get_all_orders(pool: &Pool) -> DResult<Vec<Order>> {
        let pool = pool.clone();
        let orders = sqlx::query_as!(Order, "SELECT * FROM order_tracking")
            .fetch_all(&pool)
            .await?;
        Ok(orders)
    }

    pub async fn get_categories(pool: &Pool) -> DResult<Vec<Category>> {
        let pool = pool.clone();
        let categories = sqlx::query_as!(Category, "SELECT * FROM category")
            .fetch_all(&pool)
            .await?;
        Ok(categories)
    }


    pub async fn create_category(pool: &Pool, data: &Category) -> DResult<Category> {
        let pool = pool.clone();
        let Category { name, created_at, id } = data;
        let category = sqlx::query_as!(
            Category,
            "INSERT INTO category(name, created_at, id) VALUES($1, $2, $3) RETURNING *",
            name,
            created_at,
            id
        )
        .fetch_one(&pool)
        .await?;
        Ok(category)
    }

    // Update address by user's id and return the user
    pub async fn update_address(pool: &Pool, data: AddressData) -> DResult<ResponseUser> {
        let pool = pool.clone();
        let AddressData { address, user_id } = data;
        sqlx::query!("UPDATE users SET address = $1 WHERE id = $2", address, user_id)
            .execute(&pool)
            .await?;
        let user = DatabaseHand::get_user(&pool, user_id).await?;
        Ok(user)
        
    }

    // Get image extension
    pub async fn get_image_ext(pool: &Pool, id: &Uuid) -> DResult<String> {
        let pool = pool.clone();
        let image = sqlx::query!(
            "SELECT extension FROM images WHERE for_id = $1",
            id
        )
        .fetch_one(&pool)
        .await?;
     
        Ok(image.extension)
    }
    // Get product from id 
    
}
