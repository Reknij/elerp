use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::erp::order_module::model::{OrderCurrency, OrderType};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserConfigureDefaults {
    pub order_type: OrderType,
    pub order_category_id: i64,
    pub warehouse_id: i64,
    pub person_related_id: i64,
    pub order_currency: OrderCurrency,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserConfigure {
    pub user_id: i64,
    pub language: String,
    pub defaults: UserConfigureDefaults,
}

impl Default for UserConfigureDefaults {
    fn default() -> Self {
        Self {
            order_type: OrderType::StockOut,
            order_category_id: 0,
            warehouse_id: 0,
            person_related_id: 0,
            order_currency: OrderCurrency::USD,
        }
    }
}
