use std::sync::Arc;

use api::{
    database::Database,
    web::routes::{
        add_product_to_box, auth, create_box, create_listing, delete_box, delete_listing,
        delete_single_product, generate_link, get_all_users, get_image, get_listing_from_id,
        get_listing_hex, get_listing_ich, get_listings, register_user, send_server_status,
        sign_in_user, buy_box, get_product, update_address
    },
    State,
};
use axum::{
    http::{header::CONTENT_TYPE, Method},
    routing::{get, post},
    Extension, Router, Server,
};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{CorsLayer, Origin};

#[tokio::main]
async fn main() {
    let database = Database::new("postgres://haider:@localhost:5432/ichinbankuji").await;
    let state = State { database };
    let router = Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/signin", post(sign_in_user))
        .route("/admin/create/listing", post(create_listing))
        .route("/admin/create/box", post(create_box))
        .route("/auth/verify", get(auth))
        .route("/get/users", get(get_all_users))
        .route("/get/listings", get(get_listings))
        .route("/admin/delete/listing", post(delete_listing))
        .route("/admin/delete/product", post(delete_single_product))
        .route("/admin/server_status", get(send_server_status))
        .route("/admin/add/product", post(add_product_to_box))
        .route("/admin/delete/box", post(delete_box))
        .route("/get/image/:id", get(get_image))
        .route("/admin/generate/image_link", post(generate_link))
        .route("/buy/box", post(buy_box))
        .route("/get/listings/ich", get(get_listing_ich))
        .route("/get/listings/hex", get(get_listing_hex))
        .route("/get/listing", post(get_listing_from_id))
        .route("/get/product", post(get_product))
        .route("/update/address", post(update_address))
        .layer(Extension(Arc::new(state)))
        .layer(CookieManagerLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::exact("http://localhost:4200".parse().unwrap()))
                .allow_methods(vec![Method::GET, Method::POST])
                .allow_credentials(true)
                .allow_headers(vec![CONTENT_TYPE]),
        );

    match Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
    {
        Ok(_) => println!("Server started"),
        Err(e) => println!("Server error: {}", e),
    }
}
