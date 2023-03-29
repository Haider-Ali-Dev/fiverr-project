use std::{io, str::FromStr, sync::Arc};

use axum::{
    body::StreamBody,
    extract::{Multipart, Path},
    http::{header::COOKIE, HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json, TypedHeader,
};
use headers::ContentType;
use tower_cookies::{Cookie, Cookies};
use tungstenite::{Message, WebSocket};
use uuid::Uuid;

use crate::{
    database::{actions::DatabaseHand, Database},
    error::ApiError,
    models::{self, ImageLink, Listing, Product, ResponseUser, ServerStatus, User, Amount, LogData, Category},
    web::{ImageData, ReqId},
    State,
};
use tokio::fs::File as AsyncFile;
use tokio::io::BufWriter as AsyncBufWriter;

use axum::body::Bytes;
use axum::BoxError;
use futures::{Stream, TryStreamExt};

use tokio_util::io::{ReaderStream, StreamReader};

use super::{
    AddressDataReq, BoxCreation, DeleteListing, Id, IdAndReqId, IdReq, ProductCreation, Register,
    ReqListing, SignIn, CategoryData,
};

pub async fn register_user(
    Extension(data): Extension<Arc<State>>,
    user: Json<Register>,
    cookies: Cookies,
) -> Result<Json<ResponseUser>, ApiError> {
    let pool = data.database.pool.clone();
    let user_data: User = user.0.clone().try_into()?;
    let response = DatabaseHand::create_user(&pool, &user_data).await?;
    let private_key = DatabaseHand::get_private_key(&pool, &user_data.id)
        .await?
        .to_string();
    cookies.add(Cookie::new("session_id", private_key));
    Ok(Json(response))
}

pub async fn sign_in_user(
    Extension(data): Extension<Arc<State>>,
    user: Json<SignIn>,
    cookies: Cookies,
) -> Result<Json<ResponseUser>, ApiError> {
    let pool = data.database.pool.clone();
    let user_data = user.0.clone();
    let response = DatabaseHand::sign_in(&pool, &user_data).await?;
    let private_key = DatabaseHand::get_private_key(&pool, &response.id)
        .await?
        .to_string();
    cookies.add(Cookie::new("session_id", private_key));
    Ok(Json(response))
}
pub async fn get_all_users(
    Extension(data): Extension<Arc<State>>,
) -> Result<Json<Vec<ResponseUser>>, ApiError> {
    let pool = data.database.pool.clone();
    let users = DatabaseHand::get_users(&pool).await?;
    Ok(Json(users))
}
pub async fn auth(
    Extension(data): Extension<Arc<State>>,
    headers: HeaderMap,
) -> Result<Json<ResponseUser>, ApiError> {
    let pool = data.database.pool.clone();
    match headers.get(COOKIE) {
        Some(data) => {
            let cookie = data.to_str().unwrap().split('=').collect::<Vec<_>>()[1];
            let key: String = cookie.to_string();
            let user =
                DatabaseHand::get_user_from_private_key(&pool, &Uuid::from_str(&key).unwrap())
                    .await?;
            Ok(Json(user))
        }
        None => Err(ApiError::NoSessionCookieFound),
    }
}

pub async fn get_listings(
    Extension(data): Extension<Arc<State>>,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let listings = DatabaseHand::get_listing(&pool).await?;
    Ok(Json(listings))
}

pub async fn create_listing(
    Extension(data): Extension<Arc<State>>,
    mut form: Multipart,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let mut req_list = ReqListing {
        tty: String::new(),
        title: String::new(),
        image: String::new(),
        req_id: String::new(),
        description: String::new(),
        category_id: String::new(),
    };
    let mut file_name = String::from("database/images/");
    let mut ext = String::new();
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
                "tty" => {
                    let value = f.text().await?;
                    req_list.tty = value;
                },
                "description" => {
                    let value = f.text().await?;
                    req_list.description = value;
                },
                "category_id" => {
                    let value = f.text().await?;
                    req_list.category_id = value;
                },
                "file" => match f.content_type() {
                    Some("image/png") => {
                        let id = uuid::Uuid::new_v4().to_string();
                        file_name.push_str(&id);
                        ext.push_str(&"PNG".to_string());

                        stream_to_file(&format!("{id}.png"), f).await.unwrap();
                    }
                    Some("image/jpeg") => {
                        let id = uuid::Uuid::new_v4().to_string();
                        file_name.push_str(&id);
                        ext.push_str(&"JPG".to_string());
                        println!("{}", id);
                        stream_to_file(&format!("{id}.jpg"), f).await.unwrap();
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    let listing: Listing = req_list.clone().into();
    let req_id: ReqId = req_list.into();
    let image_data = ImageData {
        path: file_name,
        ext,
        id: listing.clone().id,
    };

    let listings = DatabaseHand::create_listing(&pool, (listing, req_id, image_data)).await?;
    Ok(Json(listings))
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

pub async fn generate_link(
    Extension(data): Extension<Arc<State>>,
    mut form: Multipart,
) -> Result<Json<ImageLink>, ApiError> {
    let id = uuid::Uuid::new_v4();
    let mut img = ImageData {
        path: String::new(),
        id: id.clone(),
        ext: String::new(),
    };

    let mut file_name = String::from("database/images/");
    while let Some(f) = form.next_field().await.unwrap() {
        if let Some(name) = f.name() {
            match name {
                "file" => match f.content_type() {
                    Some("image/png") => {
                        file_name.push_str(&id.to_string());
                        img.ext = "PNG".to_string();
                        stream_to_file(&format!("{id}.png"), f).await.unwrap();
                    }
                    Some("image/jpeg") => {
                        file_name.push_str(&id.to_string());
                        img.ext = "JPG".to_string();
                        stream_to_file(&format!("{id}.jpg"), f).await.unwrap();
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
    let img_clone = img.clone();
    DatabaseHand::save_image(&data.database.pool.clone(), img).await?;
    Ok(Json(ImageLink {
        link: img_clone.id.to_string(),
    }))
}
pub async fn create_box(
    Extension(data): Extension<Arc<State>>,
    box_data: Json<BoxCreation>,
) -> Result<Json<Vec<models::Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let box_data = box_data.0.into();
    let bx = DatabaseHand::create_box(&pool, box_data).await?;
    match DatabaseHand::check_listing_tty(&pool, &bx[0].listing_id)
        .await?
        .as_str()
    {
        "ICH" => {
            let lis = DatabaseHand::get_listing_ich(&pool).await?;
            Ok(Json(lis))
        }
        "HEX" => {
            let lis = DatabaseHand::get_listing_hex(&pool).await?;
            Ok(Json(lis))
        }
        _ => Err(ApiError::NotSuperuser),
    }
}

pub async fn delete_listing(
    Extension(data): Extension<Arc<State>>,
    listing_data: Json<DeleteListing>,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let listings = DatabaseHand::delete_listing(&pool, listing_data.0.clone().into()).await?;
    Ok(Json(listings))
}

pub async fn delete_single_product(
    Extension(data): Extension<Arc<State>>,
    product_data: Json<IdAndReqId>,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let products = DatabaseHand::delete_product(&pool, product_data.0.clone().into()).await?;
    Ok(Json(products))
}

pub async fn get_listing_hex(
    Extension(data): Extension<Arc<State>>,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let listings = DatabaseHand::get_listing_hex(&pool).await?;
    Ok(Json(listings))
}

pub async fn get_listing_ich(
    Extension(data): Extension<Arc<State>>,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let listings = DatabaseHand::get_listing_ich(&pool).await?;
    Ok(Json(listings))
}

pub async fn send_server_status() -> Result<Json<ServerStatus>, ApiError> {
    let status = ServerStatus {
        status: true,
        message: "Server is up and running".to_string(),
    };
    Ok(Json(status))
}

pub async fn add_product_to_box(
    Extension(data): Extension<Arc<State>>,
    product_data: Json<ProductCreation>,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let product_data: (Vec<Product>, ReqId, Uuid) = product_data.0.into();
    let listing =
        DatabaseHand::add_product_to_box(&pool, (product_data.1, product_data.2, product_data.0))
            .await?;
    Ok(Json(listing))
}

pub async fn delete_box(
    Extension(data): Extension<Arc<State>>,
    box_data: Json<IdAndReqId>,
) -> Result<Json<Vec<Listing>>, ApiError> {
    let pool = data.database.pool.clone();
    let _ = DatabaseHand::delete_box(&pool, box_data.0.clone().into()).await?;
    Ok(Json(DatabaseHand::get_listing(&pool).await?))
}
pub async fn get_image(
    Path(id): Path<String>,
    Extension(data): Extension<Arc<State>>,
) -> Result<impl IntoResponse, ApiError> {
    let pool = data.database.pool.clone();
    println!("{id}");
    println!(
        "{:?}",
        DatabaseHand::get_image_ext(&pool, &Uuid::from_str(&id).unwrap()).await
    );
    let mut rp = "database/images/".to_string();
    let (file, raw_path) =
        match DatabaseHand::get_image_ext(&pool, &Uuid::from_str(&id).unwrap()).await {
            Ok(img) => {
                let file = match tokio::fs::File::open(format!(
                    "database/images/{id}.{ext}",
                    id = id,
                    ext = img.to_lowercase()
                ))
                .await

                {
                    Ok(file) => {
                        rp = format!("database/images/{id}.{ext}", id = id, ext = img.to_lowercase());
                        file
                    },
                    Err(_) => {
                        let (path, ext) = DatabaseHand::get_image_p_and_ext(&pool, &Uuid::from_str(&id).unwrap()).await?;
                        let e = path;
                        rp = format!("{}.{}", e, ext.to_lowercase());
                       // Now create raw_path and file from e

                        let file = match tokio::fs::File::open(rp.clone()).await {
                            Ok(file) => file,
                            Err(_) => {
                                dbg!("ERROR HIT 1");
                                return Err(ApiError::ImageNotFound);
                            }
                        };
                        
                        file
                    },

                };

                (
                    file,
                   rp
                )
            }
            Err(_) => {
                dbg!("ERROR HIT 2");
                return Err(ApiError::ImageNotFound);
            }
        };

    // let (file, raw_path) = match tokio::fs::File::open(format!("database/images/{id}.png")).await {
    //     Ok(file) => (file, format!("database/images/{id}.png")),
    //     Err(_) => return Err(ApiError::ImageNotFound),
    // };
    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    println!("{:?}", raw_path);
    match raw_path.split('.').collect::<Vec<_>>()[1] {
        "png" => {
            let he = TypedHeader(ContentType::from(mime::IMAGE_PNG));
            return Ok((he, body));
        }
        "jpg" => {
            let he = TypedHeader(ContentType::from(mime::IMAGE_JPEG));
            return Ok((he, body));
        }
        _ => {
            dbg!("ERROR HIT");
            Err(ApiError::ImageNotFound)
        }
    }
}

pub async fn get_listing_from_id(
    Extension(data): Extension<Arc<State>>,
    id: Json<Id>,
) -> Result<Json<Listing>, ApiError> {
    match Uuid::from_str(&id.0.id) {
        Ok(i) => {
            let pool = data.database.pool.clone();
            let listing = DatabaseHand::get_listing_from_id(&pool, &i).await?;
            Ok(Json(listing))
        }
        Err(_) => return Err(ApiError::InvalidId),
    }
}

pub async fn get_product(
    Extension(data): Extension<Arc<State>>,
    product_data: Json<IdReq>,
) -> Result<Json<Product>, ApiError> {
    let pool = data.database.pool.clone();
    Ok(Json(
        DatabaseHand::get_single_product(&pool, &Uuid::from_str(&product_data.id).unwrap()).await?,
    ))
}

pub async fn buy_box(
    Extension(data): Extension<Arc<State>>,
    box_data: Json<IdAndReqId>,
) -> Result<Json<Product>, ApiError> {
    let pool = data.database.pool.clone();
    Ok(Json(
        DatabaseHand::buy_box(&pool, box_data.0.clone().into()).await?,
    ))
}

// update address
pub async fn update_address(
    Extension(data): Extension<Arc<State>>,
    address_data: Json<AddressDataReq>,
) -> Result<Json<ResponseUser>, ApiError> {
    let pool = data.database.pool.clone();
    let u = DatabaseHand::update_address(&pool, address_data.0.clone().into()).await?;
    Ok(Json(u))
}

pub async fn logout(
    Extension(_): Extension<Arc<State>>,
    cookies: Cookies,
) -> Result<Json<ServerStatus>, ApiError> {
    cookies.remove(Cookie::named("session"));
    Ok(ServerStatus {
        status: true,
        message: "Logged out".to_string(),
    }
    .into())
}

pub async fn add_points(
    Extension(data): Extension<Arc<State>>,
    points_data: Json<Amount>,
) -> Result<Json<ResponseUser>, ApiError> {
    let pool = data.database.pool.clone();
    let u = DatabaseHand::add_coins(&pool, &points_data).await?;
    Ok(Json(u))
}

pub async fn get_logs(
    Extension(data): Extension<Arc<State>>,
) -> Result<Json<Vec<LogData>>, ApiError> {
    let pool = data.database.pool.clone();
    let logs = DatabaseHand::get_logs(&pool).await?;
    Ok(Json(logs))
}

pub async fn create_category(
    Extension(data): Extension<Arc<State>>,
    category_data: Json<CategoryData>,
) -> Result<Json<Category>, ApiError> {
    let pool = data.database.pool.clone();
    let category_data: Category = category_data.0.clone().into();
    let category = DatabaseHand::create_category(&pool, &category_data).await?;
    Ok(Json(category))
}

pub async fn get_categories(
    Extension(data): Extension<Arc<State>>,
) -> Result<Json<Vec<Category>>, ApiError> {
    let pool = data.database.pool.clone();
    let categories = DatabaseHand::get_categories(&pool).await?;
    Ok(Json(categories))
}


// Websocket route which shows realtime logs axum can be used to create websocket routes as well.
// This route will be used to send logs to the client and make it compatible with the axum
// websocket route.

// pub async fn ws_route(
//     Extension(data): Extension<Arc<State>>,
//     // The `ws` extractor extracts the `WebSocket` from the request.
//     // import the `ws` extractor from `axum::extract::ws`
//     ws: WebSocket,
// ) -> Result<impl IntoResponse, ApiError> {
//     let (mut sender, mut receiver) = ws.split();
//     let mut stream = tokio::stream::StreamExt::fuse(tokio::stream::iter(vec![
//         Ok(Message::text("Hello")),
//         Ok(Message::text("World")),
//     ]));
//     while let Some(message) = stream.next().await {
//         sender.send(message?).await?;
//     }
//     while let Some(message) = receiver.next().await {
//         let message = message?;
//         if message.is_close() {
//             break;
//         }
//     }
//     Ok("Hello")
// }
