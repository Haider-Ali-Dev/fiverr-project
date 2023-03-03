use std::{io, sync::Arc};

use axum::{extract::Multipart, http::StatusCode, Extension, Json};

use crate::{
    database::actions::DatabaseHand,
    error::ApiError,
    models::{Listing, ResponseUser, User},
    State,
};
use tokio::fs::File as AsyncFile;
use tokio::io::BufWriter as AsyncBufWriter;

use axum::body::{Body, Bytes, HttpBody};
use axum::BoxError;
use futures::{Stream, TryStreamExt};
use std::io::{BufWriter, Write};
use tokio_util::io::{ReaderStream, StreamReader};

use super::{Register, ReqListing, SignIn};

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

pub async fn create_listing(
    mut form: Multipart,
    Extension(data): Extension<Arc<State>>,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let mut req_list = ReqListing {
        title: String::new(),
        image: String::new(),
        req_id: String::new(),
    };
    let mut file_name = String::from("database/images/");
    while let Some(f) = form.next_field().await.unwrap() {
        if let Some(name) = f.name() {
            match name {
                "title" => {
                    let value = f.text().await?;
                    req_list.title = value;
                }
                "req_id" => {
                    let value = f.text().await?;
                    req_list.req_id = value;
                }
                "file" => match f.content_type() {
                    Some("image/png") => {
                        let id = uuid::Uuid::new_v4().to_string();
                        file_name.push_str(&id);
                        stream_to_file(&format!("{id}.png"), f).await.unwrap();
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
    todo!();
}

pub async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    async {
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        let path = std::path::Path::new("database/images/").join(path);
        let mut file = AsyncBufWriter::new(AsyncFile::create(path).await?);

        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}
