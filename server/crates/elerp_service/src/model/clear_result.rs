use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct ClearResult {
    pub success: i64,
    pub failed: i64,
}