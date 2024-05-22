use ahash::{HashSet, HashSetExt};
use anyhow::{bail, Result};
use sqlx::{Row, SqliteConnection};
use tracing::warn;

use crate::order_module::model::order::{OrderItem, OrderType};

use self::model::{inventory::InventoryProduct, virtual_inventory::VirtualInventory};

pub mod model;

pub fn calc_quantity_by_order_type(mut inventory_quantity: i64, item: &OrderItem, order_type: OrderType) -> i64 {
    match order_type {
        OrderType::Return | OrderType::StockIn => {
            if !item.exchanged {
                inventory_quantity += item.quantity
            }
        }
        OrderType::StockOut => {
            if !item.exchanged {
                inventory_quantity -= item.quantity
            }
        }
        OrderType::Exchange => {
            if item.exchanged {
                inventory_quantity += item.quantity
            } else {
                inventory_quantity -= item.quantity
            }
        }
        OrderType::Calibration | OrderType::CalibrationStrict => {
            if !item.exchanged {
                inventory_quantity = item.quantity;
            }
        }
        OrderType::Verification | OrderType::VerificationStrict => (),
    };
    inventory_quantity
}

pub fn get_virtual(capicity: usize) -> VirtualInventory {
    VirtualInventory::new(capicity)
}

async fn add(warehouse_id: i64, sku_id: i64, sku_category_id: i64, quantity: i64, tx: &mut SqliteConnection) -> Result<Option<InventoryProduct>> {
    let r = sqlx::query("INSERT INTO inventory (warehouse_id, sku_id, sku_category_id, quantity) VALUES (?, ?, ?, ?)")
        .bind(warehouse_id)
        .bind(sku_id)
        .bind(sku_category_id)
        .bind(quantity)
        .execute(&mut *tx)
        .await?;

    Ok(if r.rows_affected() != 1 {
        None
    } else {
        Some(InventoryProduct {
            warehouse_id,
            sku_id,
            sku_category_id,
            quantity,
        })
    })
}

async fn update(product: InventoryProduct, tx: &mut SqliteConnection) -> Result<Option<InventoryProduct>> {
    let r = sqlx::query("UPDATE inventory SET quantity=? WHERE warehouse_id = ? AND sku_id = ?")
        .bind(product.quantity)
        .bind(product.warehouse_id)
        .bind(product.sku_id)
        .execute(&mut *tx)
        .await?;

    Ok(if r.rows_affected() != 1 { None } else { Some(product) })
}

pub async fn change(warehouse_id: i64, items: &Vec<OrderItem>, order_type: OrderType, tx: &mut SqliteConnection) -> Result<()> {
    let mut products = HashSet::with_capacity(items.len());
    let mut inventory = get_virtual(items.len());

    if order_type == OrderType::CalibrationStrict {
        sqlx::query("UPDATE inventory SET quantity=0 WHERE warehouse_id=?").bind(warehouse_id).execute(&mut *tx).await?;
    }
    for item in items {
        if item.exchanged && order_type != OrderType::Exchange {
            continue;
        }
        if let Some(sku_row) = sqlx::query("SELECT sku_category_id FROM sku_list WHERE id = ? LIMIT 1")
            .bind(item.sku_id)
            .fetch_optional(&mut *tx)
            .await?
        {
            match inventory.get_mut(warehouse_id, item.sku_id, tx).await? {
                Some(product) => {
                    product.change(calc_quantity_by_order_type(product.latest_quantity(), item, order_type));
                    products.insert(item.sku_id);
                }
                None => {
                    let produtc_quantity = calc_quantity_by_order_type(0, item, order_type);
                    add(warehouse_id, item.sku_id, sku_row.get("sku_category_id"), produtc_quantity, tx)
                        .await?
                        .expect("Can't add the new inventory!");
                }
            }
        } else {
            bail!("SKU not found!")
        }
    }
    for sku_id in products {
        let vproduct = inventory.get_mut(warehouse_id, sku_id, tx).await?.expect("It must contain because called get_mut() before");
        let product = InventoryProduct {
            warehouse_id,
            sku_id,
            sku_category_id: vproduct.sku_category_id,
            quantity: vproduct.latest_quantity(),
        };
        if update(product, tx).await?.is_none() {
            warn!("Can't update the specified product by id {}!", sku_id);
            bail!("Please ensure order is correct!");
        }
    }

    Ok(())
}
