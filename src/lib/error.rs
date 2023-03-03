use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;
use bcrypt::BcryptError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("An error occurred in the database.")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Incorrect Password")]
    IncorrectPassword(#[from] BcryptError),
    #[error("User is not a superuser")]
    NotSuperuser,
    #[error("Error has been occured while parsing image.")]
    ImageError(#[from]  axum::extract::multipart::MultipartError) 
}


#[derive(Serialize)]
pub struct ErrorBody {
    error: String,
    status_code: u16,
}

impl IntoResponse for ErrorBody {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_msg) = match self {
            ApiError::DatabaseError(a) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong in the server. {a}"),
            ),
            ApiError::IncorrectPassword(_) => (
                StatusCode::BAD_REQUEST,
                "Incorrct passsword".to_string()
            ),
            ApiError::NotSuperuser => (
                StatusCode::BAD_REQUEST,
                "User is not a superuser.".to_string()
            ),
            ApiError::ImageError(_) => (
                StatusCode::BAD_REQUEST,
                "Error has been occured while parsing the image.".to_string()
            )
        };

        let body = ErrorBody {
            error: error_msg,
            status_code: status.as_u16(),
        };

        (status, body).into_response()
    }
}

