use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use strum::AsRefStr;
use tracing::error;

#[derive(Debug, AsRefStr)]
pub enum CustomErrorCode {
    PersonNotFound,
    AreaNotFound,
    WarehouseNotFound,
    SKUCategoryNotFound,
    SKUNotFound,
    OrderNotFound,
    OrderStatusNotFound,
    OrderPaymentSettled,
    OrderPaymentNotFound,
    OrderPaymentIsNone,
    OrderItemsIsEmpty,
    UserNotFound,
    WrongPassword,
    NoPermission,
    SameObject,
    CheckFailed,
    NotAllowed,
    TotalAmountUnexpected,
    SomeoneIsDepentIt,
    Linked,
    Unlinked,
    NotLinked,
    GuestOrderConfirmed,
    GuestOrderExpired,
    FromGuestOrder,
    UserLimitExceeded,
    WarehouseLimitExceeded,
    AreaLimitExceeded,
    PersonLimitExceeded,
    SKUCategoryLimitExceeded,
    SKULimitExceeded,
    OrderLimitExceeded,
    OrderStatusLimitExceeded,
    OrderPaymentLimitExceeded,
    GuestOrderLimitExceeded,
}

pub enum AppErrorType {
    Internal(anyhow::Error),
    InternalSQL(sqlx::Error),
    Custom(CustomErrorCode, &'static str),
}
pub struct AppError {
    err: AppErrorType,
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        AppError {
            err: AppErrorType::Internal(value),
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError {
            err: AppErrorType::InternalSQL(value),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self.err {
            AppErrorType::Internal(err) => {
                let msg = err.to_string();
                error!(msg);
                let body = json!({
                    "error_code": "E0001",
                    "error_type": "Internal",
                    "msg": "Internal server got some error, please refer to server logs.",
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }
            AppErrorType::InternalSQL(err) => {
                let msg = err.to_string();
                error!(msg);
                let body = json!({
                    "error_code": "E0001",
                    "error_type": "Internal",
                    "msg": "Internal server got some error, please refer to server logs.",
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
            }
            AppErrorType::Custom(code, msg) => {
                let body = json!({
                    "error_code": "E0002",
                    "error_type": code.as_ref(),
                    "msg": msg,
                });
                (StatusCode::EXPECTATION_FAILED, Json(body)).into_response()
            }
        }
    }
}

impl AppError {
    pub fn into_err<T>(self) -> Result<T, Self> {
        Err(self)
    }

    pub fn custom(code: CustomErrorCode, msg: &'static str) -> Self {
        Self {
            err: AppErrorType::Custom(code, msg),
        }
    }
}
