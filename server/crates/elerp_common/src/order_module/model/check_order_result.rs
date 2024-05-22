use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ItemNotAvailable {
    pub sku_id: i64,
    pub require_quantity: i64,
    pub actual_quantity: i64,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CheckOrderResult {
    pub items_not_available: Vec<ItemNotAvailable>,
}
