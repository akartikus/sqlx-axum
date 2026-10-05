use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,

    #[error("{0}")]
    Validation(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "resource not found".to_string()),

            Self::Validation(message) => (StatusCode::BAD_REQUEST, message),

            Self::Database(error) => {
                tracing::error!(
                    error = ?error,
                    "database error"
                );

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}
