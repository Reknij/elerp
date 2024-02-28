use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow)]
pub struct UserConfigure {
    pub user_id: i64,
    pub language: String,
}
