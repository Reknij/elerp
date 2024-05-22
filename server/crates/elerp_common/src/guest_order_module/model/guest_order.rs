use crate::{
    set_to_string,
    sql::{eq_or_not, get_sort_col_str, get_sorter_str, in_or_not, like_or_not},
};
use ahash::HashSet;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum::AsRefStr;
use utoipa::{IntoParams, ToSchema};

use crate::order_module::model::{
    check_order_result::CheckOrderResult,
    order::{Order, OrderCurrency, OrderItem, OrderPaymentStatus, OrderType},
};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, AsRefStr, Type, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuestOrderStatus {
    Confirmed,
    Pending,
    Expired,
}

impl Default for GuestOrderStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct GuestOrder {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub created_by_user_id: i64,
    #[serde(default)]
    pub person_in_charge_id: i64,
    #[serde(default)]
    pub date: i64,
    #[serde(default)]
    pub confirmed_date: i64,
    #[serde(default)]
    pub sub_token: String,
    #[serde(default)]
    pub currency: OrderCurrency,
    #[serde(default)]
    pub warehouse_id: i64,
    #[serde(default)]
    pub person_related_id: i64,
    #[serde(default)]
    pub description: String,

    pub order_type: OrderType,

    #[serde(default)]
    pub guest_order_status: GuestOrderStatus,
    #[serde(default)]
    pub order_id: i64,
    
    pub order_category_id: i64,
    #[serde(default)]
    pub items: Option<Vec<OrderItem>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GuestOrderConfirm {
    pub check_result: CheckOrderResult,
    pub order: Option<GuestOrder>,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams, Default)]
pub struct GetGuestOrdersQuery {
    pub id: Option<i64>,
    pub created_by_user_id: Option<i64>,
    pub fuzzy: Option<String>,
    pub warehouse_ids: Option<HashSet<i64>>,
    pub person_related_id: Option<i64>,
    pub person_in_charge_id: Option<i64>,
    pub order_type: Option<OrderType>,
    pub currency: Option<OrderCurrency>,
    pub date_start: Option<i64>,
    pub date_end: Option<i64>,
    pub confirmed_date_start: Option<i64>,
    pub confirmed_date_end: Option<i64>,
    pub guest_order_status: Option<GuestOrderStatus>,
    pub sorters: Option<Vec<String>>,
    pub reverse: Option<HashSet<String>>,
}

impl From<GuestOrder> for Order {
    fn from(value: GuestOrder) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Order {
            id: 0,
            from_guest_order_id: value.id,
            created_by_user_id: value.created_by_user_id,
            updated_by_user_id: value.created_by_user_id,
            date: now,
            last_updated_date: now,
            person_in_charge_id: value.person_in_charge_id,
            order_category_id: value.order_category_id,
            currency: value.currency,
            items: value.items,
            total_amount: 0.0,
            total_amount_settled: 0.0,
            order_payment_status: OrderPaymentStatus::None,
            warehouse_id: value.warehouse_id,
            person_related_id: value.person_related_id,
            description: value.description,
            order_type: value.order_type,
        }
    }
}

impl From<Order> for GuestOrder {
    fn from(value: Order) -> Self {
        GuestOrder {
            id: 0,
            sub_token: "".into(),
            created_by_user_id: value.created_by_user_id,
            person_in_charge_id: value.person_in_charge_id,
            currency: value.currency,
            items: value.items,
            warehouse_id: value.warehouse_id,
            person_related_id: value.person_related_id,
            description: value.description,
            order_type: value.order_type,
            guest_order_status: GuestOrderStatus::Pending,
            order_id: value.id,
            date: value.date,
            confirmed_date: value.date,
            order_category_id: value.order_category_id,
        }
    }
}

impl GetGuestOrdersQuery {
    pub fn get_where_condition(&self) -> String {
        let mut conditions = Vec::with_capacity(5);
        let reverse = self.reverse.as_ref();

        if let Some(v) = &self.id {
            let eq = eq_or_not(reverse, "id");
            conditions.push(format!("guest_orders.id{eq}{v}"));
        }
        if let Some(v) = &self.created_by_user_id {
            let eq = eq_or_not(reverse, "created_by_user_id");
            conditions.push(format!("guest_orders.created_by_user_id{eq}{v}"));
        }
        if let Some(v) = &self.fuzzy {
            let eq = like_or_not(reverse, "fuzzy");
            conditions.push(format!("CAST(guest_orders.id AS TEXT) {eq} '%{v}%' OR persons_related.name {eq} '%{v}%' OR persons_in_charge.name {eq} '%{v}%' OR order_status_list.name {eq} '%{v}%' OR warehouses.name {eq} '%{v}%'"));
        }
        if let Some(v) = &self.warehouse_ids {
            let eq = in_or_not(reverse, "warehouse_ids");
            let v = set_to_string(&v, ",");
            conditions.push(format!("guest_orders.warehouse_id{eq}({v})"));
        }
        if let Some(v) = &self.person_related_id {
            let eq = eq_or_not(reverse, "person_related_id");
            conditions.push(format!("guest_orders.person_related_id{eq}{v}"));
        }
        if let Some(v) = &self.person_in_charge_id {
            let eq = eq_or_not(reverse, "person_in_charge_id");
            conditions.push(format!("guest_orders.person_in_charge_id{eq}{v}"));
        }
        if let Some(v) = &self.order_type {
            let eq = eq_or_not(reverse, "order_type");
            conditions.push(format!("guest_orders.order_type{eq}'{}'", v.as_ref()));
        }
        if let Some(v) = &self.currency {
            let eq = eq_or_not(reverse, "currency");
            conditions.push(format!("guest_orders.currency{eq}'{}'", v.as_ref()));
        }
        if let Some(v) = &self.guest_order_status {
            let eq = eq_or_not(reverse, "guest_order_status");
            conditions.push(format!(
                "guest_orders.guest_order_status{eq}'{}'",
                v.as_ref()
            ));
        }
        if let Some(v) = &self.date_start {
            conditions.push(format!("guest_orders.date>={v}"));
        }
        if let Some(v) = &self.date_end {
            conditions.push(format!("guest_orders.date<={v}"));
        }
        if let Some(v) = &self.confirmed_date_start {
            conditions.push(format!("guest_orders.confirmed_date>={v}"));
        }
        if let Some(v) = &self.confirmed_date_end {
            conditions.push(format!("guest_orders.confirmed_date<={v}"));
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
            if col == "warehouse_id" {
                col = format!("warehouse_name {sort}").into();
            } else if col == "person_related_id" {
                col = format!("person_related_name {sort}").into();
            } else if col == "person_in_charge_id" {
                col = format!("person_in_charge_name {sort}").into();
            } else if col == "order_status_id" {
                col = format!("order_status_name {sort}").into();
            } else {
                col = format!("guest_orders.{col} {sort}").into();
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
