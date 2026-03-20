use salvo::prelude::*;
use salvo::http::{ParseError, StatusCode, StatusError};
use salvo::oapi::{oapi, EndpointOutRegister};

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

    /// Hardcoded fallback for when StatusCode constants are missing from scope
    fn get_code(n: u16) -> StatusCode {
        StatusCode::from_u16(n).unwrap_or_else(|_| {
            // If even 500 fails to parse, we use the raw internal value
            // This is a last-resort safety measure
            StatusCode::from_u16(500).unwrap()
        })
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        let (status, err) = match self {
            Self::Public(msg) => (
                Self::get_code(400),
                StatusError::bad_request().brief(msg),
            ),

            Self::Validation(e) => (
                Self::get_code(400),
                StatusError::bad_request().brief(e.to_string()),
            ),

            Self::HttpParse(e) => (
                Self::get_code(400),
                StatusError::bad_request().brief(e.to_string()),
            ),

            Self::HttpStatus(e) => (e.code, e),

            Self::SqlxError(e) => {
                tracing::error!(error = ?e, "database error");
                (
                    Self::get_code(500),
                    StatusError::internal_server_error().brief("Database error"),
                )
            }

            Self::Salvo(e) => {
                tracing::error!(error = ?e, "salvo error");
                (
                    Self::get_code(500),
                    StatusError::internal_server_error().brief("Internal server error"),
                )
            }

            Self::Internal(msg) => {
                tracing::error!(msg = msg, "internal error");
                (
                    Self::get_code(500),
                    StatusError::internal_server_error(),
                )
            }

            Self::Anyhow(e) => {
                tracing::error!(error = ?e, "anyhow error");
                (
                    Self::get_code(500),
                    StatusError::internal_server_error(),
                )
            }
        };

        res.status_code(status);
        res.render(err);
    }
}

impl EndpointOutRegister for AppError {
    fn register(
        components: &mut salvo::oapi::Components,
        operation: &mut salvo::oapi::Operation,
    ) {
        let schema = StatusError::to_schema(components);

        // closure using u16 directly
        let mut add = |status_num: u16, desc: &str| {
            if let Ok(code) = StatusCode::try_from(status_num) {
                operation.responses.insert(
                    code.as_str(),
                    oapi::Response::new(desc)
                        .add_content("application/json", schema.clone()),
                );
            }
        };

        add(400, "Bad request / validation error");
        add(401, "Unauthorized");
        add(403, "Forbidden");
        add(404, "Resource not found");
        add(409, "Conflict");
        add(422, "Unprocessable entity");
        add(500, "Internal server error");
    }
}