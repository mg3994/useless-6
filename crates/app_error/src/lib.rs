use salvo::prelude::*;
use salvo::http::{ParseError, StatusError};
use salvo::oapi::{oapi, EndpointOutRegister};
use salvo::prelude::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("public: `{0}`")]
    Public(String),
    #[error("internal: `{0}`")]
    Internal(String),
    #[error("salvo internal error: `{0}`")]
    Salvo(#[from] ::salvo::Error),
    #[error("http status error: `{0}`")]
    HttpStatus(#[from] StatusError),
    #[error("http parse error:`{0}`")]
    HttpParse(#[from] ParseError),
    #[error("anyhow error:`{0}`")]
    Anyhow(#[from] anyhow::Error),
    #[error("sqlx::Error:`{0}`")]
    SqlxError(#[from] sqlx::Error),
    #[error("validation error:`{0}`")]
    Validation(#[from] validator::ValidationErrors),
}

impl AppError {
    pub fn public<S: Into<String>>(msg: S) -> Self {
        Self::Public(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let (status, err) = match self {
            Self::Public(msg) => (
                StatusCode::BAD_REQUEST,
                StatusError::bad_request().brief(msg),
            ),

            Self::Validation(e) => (
                StatusCode::BAD_REQUEST,
                StatusError::bad_request().brief(e.to_string()),
            ),

            Self::HttpParse(e) => (
                StatusCode::BAD_REQUEST,
                StatusError::bad_request().brief(e.to_string()),
            ),

            Self::HttpStatus(e) => (e.code, e),

            Self::SqlxError(e) => {
                tracing::error!(error = ?e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    StatusError::internal_server_error().brief("Database error"),
                )
            }

            Self::Salvo(e) => {
                tracing::error!(error = ?e, "salvo error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    StatusError::internal_server_error().brief("Internal server error"),
                )
            }

            Self::Internal(msg) => {
                tracing::error!(msg = msg, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    StatusError::internal_server_error(),
                )
            }

            Self::Anyhow(e) => {
                tracing::error!(error = ?e, "anyhow error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    StatusError::internal_server_error(),
                )
            }
        };

        res.status_code(status);
        res.render(err);
    }
}

impl EndpointOutRegister for AppError {
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        let schema = StatusError::to_schema(components);

        // helper closure
        let mut add = |status_code: StatusCode, desc: &str| {
            operation.responses.insert(
                status_code.as_str(),
                oapi::Response::new(desc).add_content("application/json", schema.clone()),
            );
        };

        // use only StatusCode constants, no numbers
        add(StatusCode::BAD_REQUEST, "Bad request / validation error");
        add(StatusCode::UNAUTHORIZED, "Unauthorized");
        add(StatusCode::FORBIDDEN, "Forbidden");
        add(StatusCode::NOT_FOUND, "Resource not found");
        add(StatusCode::CONFLICT, "Conflict");
        add(StatusCode::UNPROCESSABLE_ENTITY, "Unprocessable entity");
        add(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error");
    }
}