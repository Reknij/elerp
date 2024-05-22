use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::sql::{get_search_where_condition, get_sort_col_str, get_sorter_str};

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow, Clone)]
pub struct OrderCategory {
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
pub struct GetOrderCategoryQuery {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub sorters: Option<Vec<String>>,
}

impl GetOrderCategoryQuery {
    pub fn get_where_condition(&self) -> String {
        let mut conditions = Vec::with_capacity(1);
        if let Some(v) = &self.id {
            conditions.push(format!("order_categories.id = {v}"));
        }
        if let Some(v) = &self.name {
            conditions.push(get_search_where_condition("order_categories.name", v));
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
            conditions.push("length(order_categories.name) ASC".into());
        }
        if let Some(sorters) = self.sorters.as_ref() {
            for sorter in sorters {
                let col = get_sort_col_str(sorter);
                let sort = get_sorter_str(sorter);
                conditions.push(format!("order_categories.{col} {sort}"))
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
