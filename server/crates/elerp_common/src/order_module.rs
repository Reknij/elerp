use crate::{inventory_module, sql};
use ahash::{HashMap, HashMapExt};
use anyhow::{bail, Result};
use futures::TryStreamExt;
use sqlx::{QueryBuilder, Row, SqliteConnection};

use self::model::{
    check_order_result::{CheckOrderResult, ItemNotAvailable},
    order::{Order, OrderItem, OrderPaymentStatus, OrderType},
};

pub mod model;

pub async fn check(order: &Order, fast_check: bool, tx: &mut SqliteConnection) -> Result<CheckOrderResult> {
    let mut items_not_available = Vec::new();
    if order.is_record {
        return Ok(CheckOrderResult { items_not_available });
    }
    let items = if let Some(items) = order.items.as_ref() {
        items
    } else {
        return Ok(CheckOrderResult { items_not_available });
    };
    let mut item_map = HashMap::with_capacity(items.len());
    for item in items {
        if !item.exchanged {
            item_map.entry(item.sku_id).and_modify(|q| *q += item.quantity).or_insert(item.quantity);
        }
    }
    let mut inventory = { inventory_module::get_virtual(items.len()) };
    match order.order_type {
        OrderType::Exchange | OrderType::StockOut => {
            for (sku_id, require_quantity) in item_map {
                let (latest_quantity, actual_quantity) = inventory
                    .get_mut(order.warehouse_id, sku_id, tx)
                    .await?
                    .map(|p| (p.change(p.latest_quantity() - require_quantity), p.quantity()))
                    .unwrap_or((0 - require_quantity, 0));
                if latest_quantity < 0 {
                    items_not_available.push(ItemNotAvailable {
                        sku_id,
                        require_quantity,
                        actual_quantity,
                    });
                    if fast_check {
                        return Ok(CheckOrderResult { items_not_available });
                    }
                }
            }
        }
        OrderType::StockIn | OrderType::Return | OrderType::Calibration | OrderType::CalibrationStrict => (),
        OrderType::Verification => {
            for (sku_id, require_quantity) in item_map {
                let actual_quantity = inventory.get_mut(order.warehouse_id, sku_id, tx).await?.map(|p| p.quantity()).unwrap_or(0);
                if actual_quantity != require_quantity {
                    items_not_available.push(ItemNotAvailable {
                        sku_id,
                        require_quantity,
                        actual_quantity,
                    });
                    if fast_check {
                        return Ok(CheckOrderResult { items_not_available });
                    }
                }
            }
        }
        OrderType::VerificationStrict => {
            let mut item_ids = Vec::with_capacity(item_map.len());
            for (sku_id, require_quantity) in item_map {
                item_ids.push(sku_id);
                let actual_quantity = inventory.get_mut(order.warehouse_id, sku_id, tx).await?.map(|p| p.quantity()).unwrap_or(0);
                if actual_quantity != require_quantity {
                    items_not_available.push(ItemNotAvailable {
                        sku_id,
                        require_quantity,
                        actual_quantity,
                    });
                    if fast_check {
                        return Ok(CheckOrderResult { items_not_available });
                    }
                }
            }
            let ids = item_ids.into_iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",");

            if fast_check {
                if let Some(row) = sqlx::query(&format!(
                    "SELECT sku_id, quantity FROM inventory WHERE warehouse_id=? AND sku_id NOT IN ({ids}) AND quantity <> 0 LIMIT 1"
                ))
                .bind(order.warehouse_id)
                .fetch(&mut *tx)
                .try_next()
                .await?
                {
                    items_not_available.push(ItemNotAvailable {
                        sku_id: row.get("sku_id"),
                        require_quantity: 0,
                        actual_quantity: row.get("quantity"),
                    });
                    return Ok(CheckOrderResult { items_not_available });
                }
            } else {
                let q = format!("SELECT sku_id, quantity FROM inventory WHERE warehouse_id=? AND sku_id NOT IN ({ids}) AND quantity <> 0");
                let mut r = sqlx::query(&q).bind(order.warehouse_id).fetch(&mut *tx);
                while let Some(row) = r.try_next().await? {
                    items_not_available.push(ItemNotAvailable {
                        sku_id: row.get("sku_id"),
                        require_quantity: 0,
                        actual_quantity: row.get("quantity"),
                    });
                }
            }
        }
    }
    Ok(CheckOrderResult { items_not_available })
}

fn calc_total_amount(items: &Vec<OrderItem>) -> f64 {
    let mut total: f64 = 0.0;
    for item in items.iter() {
        if item.exchanged {
            continue;
        }
        total += (item.quantity as f64) * item.price;
    }
    total
}

pub async fn add(mut order: Order, tx: &mut SqliteConnection) -> Result<Order> {
    let items = order.items.as_ref();
    let total_amount = if let Some(items) = items {
        if !order.is_record {
            inventory_module::change(order.warehouse_id, items, order.order_type, tx).await?;
        }
        calc_total_amount(items)
    } else {
        0.0
    };
    let order_payment_status = if order.order_type == OrderType::StockOut && total_amount > 0.0 {
        OrderPaymentStatus::Unsettled
    } else {
        OrderPaymentStatus::None
    };
    let r = sqlx::query("INSERT INTO orders (from_guest_order_id, created_by_user_id, updated_by_user_id, warehouse_id, currency, total_amount, person_related_id, person_in_charge_id, date, last_updated_date, description, order_type, is_record, order_category_id, total_amount_settled, order_payment_status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(order.from_guest_order_id)
        .bind(order.created_by_user_id)
        .bind(order.updated_by_user_id)
            .bind(order.warehouse_id)
            .bind(&order.currency)
            .bind(total_amount)
            .bind(order.person_related_id)
            .bind(order.person_in_charge_id)
            .bind(order.date)
            .bind(order.last_updated_date)
            .bind(&order.description)
            .bind(&order.order_type)
            .bind(order.is_record)
            .bind(order.order_category_id)
            .bind(0)
            .bind(order_payment_status)
            .execute(&mut *tx)
            .await?;
    if r.rows_affected() != 1 {
        bail!("Can't insert the order to history!");
    }

    order.id = sql::try_set_standard_id(r.last_insert_rowid(), "orders", tx).await?;
    order.total_amount = total_amount;
    order.total_amount_settled = 0.0;
    order.order_payment_status = order_payment_status;

    add_order_items(&order, tx).await?;

    Ok(order)
}

async fn add_order_items(order: &Order, tx: &mut SqliteConnection) -> Result<()> {
    let items = order.items.as_ref();
    if items.is_none() || items.as_ref().unwrap().len() == 0 {
        return Ok(());
    }
    let mut query_builder = QueryBuilder::new("INSERT INTO order_items (order_id, sku_id, sku_category_id, quantity, price, exchanged, amount) ");
    query_builder.push_values(items.unwrap(), |mut b, item| {
        b.push_bind(order.id)
            .push_bind(item.sku_id)
            .push(format!("(SELECT sku_category_id FROM sku_list WHERE id = {})", item.sku_id))
            .push_bind(item.quantity)
            .push_bind(item.price)
            .push_bind(item.exchanged)
            .push_bind(item.quantity as f64 * item.price);
    });
    let query = query_builder.build();
    query.execute(&mut *tx).await?;
    Ok(())
}
