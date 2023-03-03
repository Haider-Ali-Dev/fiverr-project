use std::sync::Arc;

use axum::{Extension, Json};

use crate::{
    database::actions::DatabaseHand,
    error::ApiError,
    models::{ResponseUser, User},
    State,
};

use super::{Register, SignIn};

pub async fn register_user(
    Extension(data): Extension<Arc<State>>,
    user: Json<Register>,
) -> Result<Json<ResponseUser>, ApiError> {
    let pool = data.database.pool.clone();
    let user_data: User = user.0.clone().try_into()?;
    let response = DatabaseHand::create_user(&pool, &user_data).await?;
    Ok(Json(response))
}

pub async fn sign_in_user(
    Extension(data): Extension<Arc<State>>,
    user: Json<SignIn>,
) -> Result<Json<ResponseUser>, ApiError> {
    let pool = data.database.pool.clone();
    let user_data = user.0.clone();
    let response = DatabaseHand::sign_in(&pool, &user_data).await?;
    Ok(Json(response))
}


