use ahash::{HashMap, HashMapExt};
use anyhow::Result;
use sqlx::{Row, SqliteConnection};

#[derive(Debug, Clone)]
pub struct VirtualInventory {
    inventory: HashMap<(i64, i64), Option<VirtualInventoryProduct>>,
}

#[derive(Debug, Clone)]
pub struct VirtualInventoryProduct {
    pub sku_category_id: i64,
    quantity: i64,
    latest_quantity: i64,
}

impl VirtualInventoryProduct {
    pub fn quantity(&self) -> i64 {
        self.quantity
    }

    pub fn latest_quantity(&self) -> i64 {
        self.latest_quantity
    }

    pub fn change(&mut self, v: i64) -> i64 {
        self.latest_quantity = v;
        self.latest_quantity
    }
}

impl VirtualInventory {
    pub fn new(capicity: usize) -> Self {
        Self {
            inventory: HashMap::with_capacity(capicity),
        }
    }

    pub async fn get_mut(
        &mut self,
        warehouse_id: i64,
        sku_id: i64,
        tx: &mut SqliteConnection,
    ) -> Result<Option<&mut VirtualInventoryProduct>> {
        Ok(if self.inventory.contains_key(&(warehouse_id, sku_id)) {
            let p = self.inventory.get_mut(&(warehouse_id, sku_id)).unwrap();
            p.as_mut()
        } else {
            let p =sqlx::query("SELECT sku_category_id, quantity FROM inventory WHERE warehouse_id = ? AND sku_id = ? LIMIT 1").bind(warehouse_id).bind(sku_id)
                .fetch_optional(&mut *tx).await?
                .map(|p| {
                    let quantity = p.get("quantity");
                    VirtualInventoryProduct {
                        sku_category_id: p.get("sku_category_id"),
                        quantity,
                        latest_quantity: quantity,
                    }
                });
            self.inventory.insert((warehouse_id, sku_id), p);
            self.inventory
                .get_mut(&(warehouse_id, sku_id))
                .unwrap()
                .as_mut()
        })
    }
}
