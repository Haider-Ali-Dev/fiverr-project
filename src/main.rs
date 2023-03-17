use std::sync::Arc;

use api::{
    database::Database,
    web::routes::{create_box, create_listing, register_user, sign_in_user, auth, get_all_users, get_listings, delete_listing, send_server_status},
    State,
};
use axum::{
    http::{header::CONTENT_TYPE, Method},
    routing::{post, get},
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
        .route("/admin/server_status", get(send_server_status))
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
