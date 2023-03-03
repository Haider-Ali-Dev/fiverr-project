use std::sync::Arc;

use api::{
    database::Database,
    web::routes::{register_user, sign_in_user},
    State,
};
use axum::{routing::post, Extension, Router, Server};

#[tokio::main]
async fn main() {
    let database = Database::new("postgres://haider:@localhost:5432/ichinbankuji").await;
    let state = State { database };
    let router = Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/signin", post(sign_in_user))
        .layer(Extension(Arc::new(state)));

    Server::bind(&"0.0.0.0:3200".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}
