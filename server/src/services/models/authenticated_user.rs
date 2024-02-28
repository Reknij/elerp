use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, MatchedPath},
    http::request::Parts,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::{IntoParams, ToSchema};

use crate::{
    custom_error::{AppError, CustomErrorCode},
    services::AppState,
    user_system::models::{
        user_info::{UserInfo, UserType},
        user_permission::{
            ADD_ORDER, ADD_ORDER_PAYMENT, MANAGE_AREA, MANAGE_ORDER_STATUS, MANAGE_PERSON,
            MANAGE_SKU, MANAGE_SKU_CATEGORY, MANAGE_WAREHOUSE, UPDATE_REMOVE_ORDER,
            UPDATE_REMOVE_ORDER_PAYMENT,
        },
    },
};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetTokenQuery {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct AuthenticatedUser {
    pub user: UserInfo,
    pub token: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, s: &S) -> Result<Self, Self::Rejection> {
        let app = AppState::from_ref(s);

        // Use the MatchedPath extractor inside the from_request_parts method
        let path = MatchedPath::from_request_parts(parts, s)
            .await
            .map(|path| path.as_str().to_string())
            .map_err(|err| anyhow::Error::from(err))?;

        let auth = match parts.headers.get("X-authorization") {
            Some(v) => {
                if let Ok(auth) = v.to_str() {
                    Some(auth)
                } else {
                    None
                }
            }
            None => {
                let mut r = None;
                if let Some(a) = parts.headers.get("Sec-Websocket-Protocol") {
                    if let Ok(v) = a.to_str() {
                        let arr: Vec<&str> = v.split(", ").collect();
                        if arr.len() > 1 {
                            r = Some(arr[1]);
                        }
                    }
                }
                r
            }
        };
        if let Some(auth) = auth {
            let mut tx = app.ps.begin_tx(false).await?;
            let user = app
                .us
                .token_to_user(auth, crate::erp::ActionType::System, tx.as_mut())
                .await?;
            if let Some(user) = user {
                if path.starts_with("/api/erp")
                    && !app.us.is_socket_connected(user.id, tx.as_mut()).await?
                {
                    return AppError::custom(
                        CustomErrorCode::NoPermission,
                        "You must connect web socket first before access the api of erp.",
                    )
                    .into_err();
                }
                tx.commit().await?;
                return Ok(AuthenticatedUser {
                    user,
                    token: auth.to_string(),
                });
            }
            return AppError::custom(
                CustomErrorCode::UserNotFound,
                "The user is not found with given token!",
            )
            .into_err();
        }
        info!("Some connection no login.");
        AppError::custom(CustomErrorCode::NoPermission, "Please login to continue...").into_err()
    }
}

impl AuthenticatedUser {
    pub fn fail_if_not_admin(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin {
            AppError::custom(CustomErrorCode::NoPermission, "Only admin can access.").into_err()?;
        }
        Ok(())
    }

    pub fn is_manage_area(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin
            && self.user.permission & MANAGE_AREA != MANAGE_AREA
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to manage area.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_manage_person(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin
            && self.user.permission & MANAGE_PERSON != MANAGE_PERSON
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to manage person.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_manage_warehouse(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin
            && self.user.permission & MANAGE_WAREHOUSE != MANAGE_WAREHOUSE
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to manage warehouse.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_manage_sku(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin && self.user.permission & MANAGE_SKU != MANAGE_SKU
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to manage sku.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_manage_order_status(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin
            && self.user.permission & MANAGE_ORDER_STATUS != MANAGE_ORDER_STATUS
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to manage order status.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_add_order_payment(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin
            && self.user.permission & ADD_ORDER_PAYMENT != ADD_ORDER_PAYMENT
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to add order payment.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_update_remove_order_payment(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin
            && self.user.permission & UPDATE_REMOVE_ORDER_PAYMENT != UPDATE_REMOVE_ORDER_PAYMENT
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to remove order payment.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_manage_sku_category(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin
            && self.user.permission & MANAGE_SKU_CATEGORY != MANAGE_SKU_CATEGORY
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to manage sku category.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_add_order(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin && self.user.permission & ADD_ORDER != ADD_ORDER {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to add order.",
            )
            .into_err();
        }
        Ok(())
    }

    pub fn is_update_remove_order(&self) -> Result<(), AppError> {
        if self.user.user_type != UserType::Admin
            && self.user.permission & UPDATE_REMOVE_ORDER != UPDATE_REMOVE_ORDER
        {
            return AppError::custom(
                CustomErrorCode::NoPermission,
                "You don't have permission to update/remove order.",
            )
            .into_err();
        }
        Ok(())
    }
}
