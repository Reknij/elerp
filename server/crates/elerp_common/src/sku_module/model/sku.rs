use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::sql::{get_search_where_condition, get_sort_col_str, get_sorter_str};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow)]
pub struct SKU {
    /// Id will generated by the system.
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub sku_category_id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub color: Option<String>,
    pub text_color: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetSKUsQuery {
    pub id: Option<i64>,
    pub sku_category_id: Option<i64>,
    pub name: Option<String>,
    pub sorters: Option<Vec<String>>,
}

impl GetSKUsQuery {
    pub fn get_where_condition(&self) -> String {
        let mut conditions = Vec::with_capacity(2);
        if let Some(v) = &self.id {
            conditions.push(format!("sku_list.id = {v}").into());
        }
        if let Some(v) = self.sku_category_id {
            conditions.push(format!("sku_list.sku_category_id={v}").into());
        }
        if let Some(v) = &self.name {
            let v = v.trim();
            conditions.push(get_search_where_condition("sku_list.name", v));
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
            conditions.push("length(sku_list.name) ASC".into());
        }
        if let Some(sorters) = self.sorters.as_ref() {
            for sorter in sorters {
                let mut col = get_sort_col_str(&sorter);
                let sort = get_sorter_str(sorter);
                if col == "sku_category_id" {
                    col = format!("sku_category_name {sort}").into();
                } else {
                    col = format!("sku_list.{col} {sort}").into();
                }
                conditions.push(col)
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
