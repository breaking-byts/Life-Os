use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Validation,
    NotFound,
    Conflict,
    Transient,
    Internal,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Validation,
            message: message.into(),
            details: None,
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::NotFound,
            message: message.into(),
            details: None,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Conflict,
            message: message.into(),
            details: None,
        }
    }

    pub fn transient(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Transient,
            message: message.into(),
            details: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Internal,
            message: message.into(),
            details: None,
        }
    }

    pub fn from_sqlx(err: sqlx::Error, message: impl Into<String>) -> Self {
        let mut base: ApiError = err.into();
        base.message = message.into();
        base
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::not_found("Resource not found"),
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() || db_err.is_foreign_key_violation() {
                    ApiError::conflict(db_err.message().to_string())
                } else {
                    ApiError::internal(db_err.message().to_string())
                }
            }
            sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_) => {
                ApiError::transient("Database temporarily unavailable")
            }
            _ => ApiError::internal(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::internal(err.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::internal(err.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for ApiError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        ApiError::internal(err.to_string())
    }
}

impl From<String> for ApiError {
    fn from(err: String) -> Self {
        ApiError::internal(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_sets_code_message() {
        let err = ApiError::validation("Invalid input");
        assert_eq!(err.code, ErrorCode::Validation);
        assert_eq!(err.message, "Invalid input");
        assert!(err.details.is_none());
    }

    #[test]
    fn error_serializes_with_snake_case_code() {
        let err = ApiError::validation("Invalid input");
        let json = serde_json::to_string(&err).expect("serialize ApiError");
        assert!(json.contains("\"code\":\"validation\""));
        assert!(json.contains("\"message\":\"Invalid input\""));
    }

    #[test]
    fn sqlx_row_not_found_maps_to_not_found() {
        let err: ApiError = sqlx::Error::RowNotFound.into();
        assert_eq!(err.code, ErrorCode::NotFound);
    }
}
