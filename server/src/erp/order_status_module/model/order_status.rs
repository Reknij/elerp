use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::erp::util::{get_sort_col_str, get_sorter_str, get_search_where_condition};

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow, Clone)]
pub struct OrderStatus {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub color: Option<String>,
    pub text_color: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams, PartialEq, Eq)]
pub struct GetOrderStatusQuery {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub sorters: Option<Vec<String>>,
}

impl GetOrderStatusQuery {
    pub fn get_where_condition(&self) -> String {
        let mut conditions = Vec::with_capacity(1);
        if let Some(v) = &self.id {
            conditions.push(format!("order_status_list.id = {v}"));
        }
        if let Some(v) = &self.name {
            conditions.push(get_search_where_condition("order_status_list.name", v));
        }
        if !conditions.is_empty() {
            let c = conditions.join(" AND ");
            format!("WHERE {c}").into()
        } else {
            "".into()
        }
    }

    pub fn get_order_condition(&self) -> String {
        let mut conditions = vec![];
        if self.name.is_some() {
            conditions.push("length(order_status_list.name) ASC".into());
        }
        if let Some(sorters) = self.sorters.as_ref() {
            for sorter in sorters {
                let col = get_sort_col_str(sorter);
                let sort = get_sorter_str(sorter);
                conditions.push(format!("order_status_list.{col} {sort}"))
            }
        }

        if !conditions.is_empty() {
            let c = conditions.join(", ");
            format!("ORDER BY {c}").into()
        } else {
            "".into()
        }
    }
}
