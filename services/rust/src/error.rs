use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

// Errors produced while importing a vote, including Stortinget client failures.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("resource not found: {message}")]
    NotFound { message: String },

    #[error("invalid source data: {message}")]
    InvalidSourceData { message: String },

    // Includes connection failures, timeouts, body-read failures,
    // and HTTP status errors from error_for_status().
    #[error("Stortinget request failed")]
    Request(#[from] reqwest::Error),

    #[error("failed to deserialize Stortinget response")]
    Decode(#[from] serde_json::Error),

    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
}

// Errors exposed through the HTTP layer.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("resource not found: {message}")]
    NotFound { message: String },

    #[error("import failed")]
    Import(#[from] ImportError),

    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl ApiError {
    /// The status and message the caller is allowed to see.
    fn public(&self) -> (StatusCode, &str) {
        match self {
            Self::NotFound { message } | Self::Import(ImportError::NotFound { message }) => {
                (StatusCode::NOT_FOUND, message)
            }

            Self::Import(
                ImportError::InvalidSourceData { .. }
                | ImportError::Request(_)
                | ImportError::Decode(_),
            ) => (StatusCode::BAD_GATEWAY, "upstream request failed"),

            Self::Database(_) | Self::Import(ImportError::Database(_)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = self.public();

        if status.is_server_error() {
            tracing::error!(error = ?self, "request failed");
        }

        (status, Json(ErrorBody { error })).into_response()
    }
}
