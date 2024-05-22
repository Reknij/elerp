use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::sql::{get_sort_col_str, get_sorter_str};

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow, IntoParams, Clone)]
pub struct InventoryProduct {
    pub warehouse_id: i64,
    pub sku_id: i64,
    pub sku_category_id: i64,
    pub quantity: i64,
}

#[derive(Debug, Deserialize, ToSchema, FromRow, IntoParams)]
pub struct GetInventoryQuery {
    pub warehouse_id: Option<i64>,
    pub sku_id: Option<i64>,
    pub sku_category_id: Option<i64>,
    pub quantity_start: Option<i64>,
    pub quantity_end: Option<i64>,
    pub sorters: Option<Vec<String>>,
}

impl GetInventoryQuery {
    pub fn get_where_condition(&self) -> String {
        let mut conditions = Vec::with_capacity(4);
        if let Some(v) = self.warehouse_id {
            conditions.push(format!("inventory.warehouse_id={v}"));
        }
        if let Some(v) = &self.sku_id {
            conditions.push(format!("inventory.sku_id={v}"));
        }
        if let Some(v) = &self.sku_category_id {
            conditions.push(format!("inventory.sku_category_id={v}"));
        }
        if let Some(v) = &self.quantity_start {
            conditions.push(format!("inventory.quantity>={v}"));
        }
        if let Some(v) = &self.quantity_end {
            conditions.push(format!("inventory.quantity<={v}"));
        }
        if !conditions.is_empty() {
            let c = conditions.join(" AND ");
            format!("WHERE {c}").into()
        } else {
            "".into()
        }
    }

    pub fn get_order_condition(&self) -> String {
        if self.sorters.is_none() {
            return "".into()
        }
        let mut conditions = vec![];
        for sorter in self.sorters.as_ref().unwrap() {
            let mut col = get_sort_col_str(sorter);
            let sort = get_sorter_str(sorter);
            if col == "sku_id" {
                col = format!("sku_name {sort}").into();
            } else if col == "sku_category_id" {
                col = format!("sku_category_name {sort}").into();
            } else if col == "warehouse_id" {
                col = format!("warehouse_name {sort}").into();
            } else {
                col = format!("inventory.{col} {sort}").into();
            }
            conditions.push(col)
        }
        if !conditions.is_empty() {
            let c = conditions.join(", ");
            format!("ORDER BY {c}").into()
        } else {
            "".into()
        }
    }
}
