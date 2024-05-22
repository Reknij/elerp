use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ListSlice<T> {
    pub count: i64,
    pub items: Vec<T>,
}
