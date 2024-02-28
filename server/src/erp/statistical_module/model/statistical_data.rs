use ahash::HashSet;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::erp::order_module::model::{GetOrdersQuery, OrderCurrency};

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow, Clone, PartialOrd)]
pub struct PopularSKU {
    pub id: i64,
    pub currency: OrderCurrency,
    pub order_count: i64,
    pub average_price: f64,
    pub total_out: i64,
}
impl PartialEq for PopularSKU {
    fn eq(&self, other: &Self) -> bool {
        self.total_out == other.total_out
    }
}
impl Eq for PopularSKU {}

impl Ord for PopularSKU {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.total_out {
            val if val < other.total_out => std::cmp::Ordering::Less,
            val if val > other.total_out => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow, Clone, PartialOrd)]
pub struct SalesAmountWithCurrency {
    pub any: f64,
    pub settled: f64,
    pub unsettled: f64,
    pub partial_settled: f64,
    pub currency: OrderCurrency,
}

impl PartialEq for SalesAmountWithCurrency {
    fn eq(&self, other: &Self) -> bool {
        self.any == other.any
    }
}
impl Eq for SalesAmountWithCurrency {}

impl Ord for SalesAmountWithCurrency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.any {
            val if val < other.any => std::cmp::Ordering::Less,
            val if val > other.any => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow, Clone)]
pub struct StatisticalOrderData {
    pub total_count: StatisticalOrderCountData,
    pub total_amount: Vec<SalesAmountWithCurrency>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow, Clone)]
pub struct StatisticalOrderCountData {
    pub any_count: i64,
    pub stock_in_count: i64,
    pub stock_out_count: i64,
    pub exchange_count: i64,
    pub return_count: i64,
    pub calibration_count: i64,
    pub calibration_strict_count: i64,
    pub verification_count: i64,
    pub verification_strict_count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, FromRow, Clone)]
pub struct StatisticalData {
    pub area_count: i64,
    pub person_count: i64,
    pub warehouse_count: i64,
    pub sku_count: i64,
    pub sku_category_count: i64,
    pub order: StatisticalOrderData,
    pub order_status_count: i64,

    pub most_popular_skus: Vec<PopularSKU>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams, Clone, PartialEq, Eq)]
pub struct GetStatisticalDataQuery {
    pub date_start: Option<i64>,
    pub date_end: Option<i64>,
    pub order_status_id: Option<i64>,
    pub warehouse_ids: Option<HashSet<i64>>,
    pub person_related_id: Option<i64>,
    pub person_in_charge_id: Option<i64>,
    pub currency: Option<OrderCurrency>,
    pub created_by_user_id: Option<i64>,
}

impl GetStatisticalDataQuery {
    pub fn get_order_query(&self) -> GetOrdersQuery {
        let mut q = GetOrdersQuery::empty();
        q.date_start = self.date_start;
        q.date_end = self.date_end;
        q.order_status_id = self.order_status_id;
        q.warehouse_ids = self.warehouse_ids.clone();
        q.person_in_charge_id = self.person_in_charge_id;
        q.person_related_id = self.person_related_id;
        q.currency = self.currency;
        q.created_by_user_id = self.created_by_user_id;
        q
    }
}
