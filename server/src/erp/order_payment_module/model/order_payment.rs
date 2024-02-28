use ahash::HashSet;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::{
    erp::util::{get_sort_col_str, get_sorter_str},
    myhelper::set_to_string,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, IntoParams, FromRow)]
pub struct OrderPayment {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub created_by_user_id: i64,
    #[serde(default)]
    pub order_id: i64,
    #[serde(default)]
    pub warehouse_id: i64,
    #[serde(default)]
    pub person_in_charge_id: i64,
    #[serde(default)]
    pub creation_date: i64,
    #[serde(default)]
    pub actual_date: i64,
    #[serde(default)]
    pub total_amount: f64,
    #[serde(default)]
    pub remark: String,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetOrderPaymentsQuery {
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub warehouse_ids: Option<HashSet<i64>>,
    pub created_by_user_id: Option<i64>,
    pub person_in_charge_id: Option<i64>,
    pub creation_date_start: Option<i64>,
    pub creation_date_end: Option<i64>,
    pub actual_date_start: Option<i64>,
    pub actual_date_end: Option<i64>,
    pub sorters: Option<Vec<String>>,
}

impl GetOrderPaymentsQuery {
    pub fn get_where_condition(&self) -> String {
        let mut conditions = Vec::with_capacity(5);
        if let Some(v) = &self.id {
            conditions.push(format!("order_payments.id = {v}"));
        }
        if let Some(v) = &self.warehouse_ids {
            let v = set_to_string(&v, ",");
            conditions.push(format!("orders.warehouse_id IN ({v})"));
        }
        if let Some(v) = &self.person_in_charge_id {
            conditions.push(format!("order_payments.person_in_charge_id={v}"));
        }
        if let Some(v) = &self.created_by_user_id {
            conditions.push(format!("order_payments.created_by_user_id={v}"));
        }
        if let Some(v) = &self.order_id {
            conditions.push(format!("order_payments.order_id={v}"));
        }
        if let Some(v) = &self.creation_date_start {
            conditions.push(format!("order_payments.creation_date>={v}"));
        }
        if let Some(v) = &self.creation_date_end {
            conditions.push(format!("order_payments.creation_date<={v}"));
        }
        if let Some(v) = &self.actual_date_start {
            conditions.push(format!("order_payments.actual_date>={v}"));
        }
        if let Some(v) = &self.actual_date_end {
            conditions.push(format!("order_payments.actual_date<={v}"));
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
            return "".into();
        }
        let mut conditions = vec![];
        for sorter in self.sorters.as_ref().unwrap() {
            let mut col = get_sort_col_str(sorter);
            let sort = get_sorter_str(sorter);
            if col == "person_id" {
                col = format!("person_name {sort}").into();
            } else {
                col = format!("order_payments.{col} {sort}").into();
            }
            conditions.push(col);
        }
        if !conditions.is_empty() {
            let c = conditions.join(", ");
            format!("ORDER BY {c}").into()
        } else {
            "".into()
        }
    }
}
