use std::{path::PathBuf, sync::Arc};

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::bail;
use futures::TryStreamExt;
use sqlx::{FromRow, Row, SqliteConnection};
use tokio::{fs, io::AsyncWriteExt, sync::RwLock};
use tracing::warn;

use crate::{
    erp::{
        sku_category_module::model::SKUCategory, sku_module::model::SKU,
        warehouse_module::model::warehouse::Warehouse, ActionType,
    },
    public_system::{model::Pagination, PublicSystem},
};

use self::model::{GetInventoryQuery, InventoryProduct};
use super::order_module::model::{OrderItem, OrderType};
use super::{dependency::ModuleDependency, Result};
pub mod model;

#[derive(Debug, Clone)]
pub struct VirtualInventory {
    inventory_module: InventoryModule,
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
    pub fn new(inventory_module: InventoryModule, capicity: usize) -> Self {
        Self {
            inventory_module,
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
            let p = self
                .inventory_module
                .get(warehouse_id, sku_id, &mut *tx)
                .await?.map(|p| {
                    VirtualInventoryProduct {
                        sku_category_id: p.sku_category_id,
                        quantity: p.quantity,
                        latest_quantity: p.quantity,
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

#[derive(Debug, Clone)]
pub struct InventoryModule {
    ps: PublicSystem,
    dependency: Arc<RwLock<Option<ModuleDependency>>>,
}

impl InventoryModule {
    pub async fn set_dependency(&self, dep: ModuleDependency) {
        *self.dependency.write().await = Some(dep);
    }

    pub async fn create_table(&self, tx: &mut SqliteConnection) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS inventory(
                warehouse_id INT NOT NULL,
                sku_id INT NOT NULL,
                sku_category_id INT NOT NULL,
                quantity INT NOT NULL
            )",
        )
        .execute(&mut *tx)
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS inventory_warehouses
        ON inventory(warehouse_id);
        CREATE INDEX IF NOT EXISTS inventory_skus
        ON inventory(sku_id);
        CREATE INDEX IF NOT EXISTS inventory_warehouses_and_skus
        ON inventory(warehouse_id, sku_id);
        CREATE INDEX IF NOT EXISTS inventory_sku_categories
        ON inventory(sku_category_id);",
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        Ok(())
    }

    pub async fn new(ps: PublicSystem) -> Self {
        let mut tx = ps.get_conn().begin().await.unwrap();
        let s = Self {
            ps,
            dependency: Arc::new(RwLock::new(None)),
        };
        s.create_table(tx.as_mut()).await.unwrap();
        tx.commit().await.unwrap();
        s
    }

    pub fn get_virtual(&self, capicity: usize) -> VirtualInventory {
        VirtualInventory::new(self.clone(), capicity)
    }

    async fn add(
        &self,
        warehouse_id: i64,
        sku: &SKU,
        quantity: i64,
        tx: &mut SqliteConnection,
    ) -> Result<Option<InventoryProduct>> {
        let r = sqlx::query("INSERT INTO inventory (warehouse_id, sku_id, sku_category_id, quantity) VALUES (?, ?, ?, ?)")
        .bind(warehouse_id)
        .bind(sku.id)
        .bind(sku.sku_category_id)
        .bind(quantity)
        .execute(&mut *tx)
        .await?;

        Ok(if r.rows_affected() != 1 {
            None
        } else {
            Some(InventoryProduct {
                warehouse_id,
                sku_id: sku.id,
                sku_category_id: sku.sku_category_id,
                quantity,
            })
        })
    }

    pub async fn get(
        &self,
        warehouse_id: i64,
        sku_id: i64,
        tx: &mut SqliteConnection,
    ) -> Result<Option<InventoryProduct>> {
        Ok(sqlx::query_as::<_, InventoryProduct>(
            "SELECT * FROM inventory WHERE warehouse_id=? AND sku_id=? LIMIT 1",
        )
        .bind(warehouse_id)
        .bind(sku_id)
        .fetch(&mut *tx)
        .try_next()
        .await?)
    }

    async fn update(
        &self,
        product: InventoryProduct,
        tx: &mut SqliteConnection,
    ) -> Result<Option<InventoryProduct>> {
        let r =
            sqlx::query("UPDATE inventory SET quantity=? WHERE warehouse_id = ? AND sku_id = ?")
                .bind(product.quantity)
                .bind(product.warehouse_id)
                .bind(product.sku_id)
                .execute(&mut *tx)
                .await?;

        Ok(if r.rows_affected() != 1 {
            None
        } else {
            Some(product)
        })
    }

    pub fn calc_quantity_by_order_type(
        &self,
        mut inventory_quantity: i64,
        item: &OrderItem,
        order_type: OrderType,
    ) -> Result<i64> {
        match order_type {
            OrderType::StockIn => {
                if !item.exchanged {
                    inventory_quantity += item.quantity
                }
            }
            OrderType::Return | OrderType::StockOut => {
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
        Ok(inventory_quantity)
    }

    pub async fn change(
        &self,
        warehouse_id: i64,
        items: &Vec<OrderItem>,
        order_type: OrderType,
        tx: &mut SqliteConnection,
    ) -> Result<()> {
        let depl = self.dependency.read().await;
        let dep: &ModuleDependency = depl.as_ref().unwrap();
        let mut products = HashSet::with_capacity(items.len());
        let mut inventory = self.get_virtual(items.len());

        if order_type == OrderType::CalibrationStrict {
            sqlx::query("UPDATE inventory SET quantity=0 WHERE warehouse_id=?")
                .bind(warehouse_id)
                .execute(&mut *tx)
                .await?;
        }
        for item in items {
            if item.exchanged && order_type != OrderType::Exchange {
                continue;
            }
            if let Some(sku) = dep.sku.get(item.sku_id, tx).await? {
                match inventory.get_mut(warehouse_id, item.sku_id, tx).await? {
                    Some(product) => {
                        product.change(self
                            .calc_quantity_by_order_type(product.latest_quantity(), item, order_type)
                            .expect("Calc quantity by order type failed!"));
                        products.insert(item.sku_id);
                    }
                    None => {
                        let produtc_quantity = self
                            .calc_quantity_by_order_type(0, item, order_type)
                            .expect("Calc quantity by order type failed!");
                        self.add(warehouse_id, &sku, produtc_quantity, tx)
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
            let product = InventoryProduct { warehouse_id, sku_id, sku_category_id: vproduct.sku_category_id, quantity: vproduct.latest_quantity() };
            if self.update(product, tx).await?.is_none() {
                warn!("Can't update the specified product by id {}!", sku_id);
                bail!("Please ensure order is correct!");
            }
        }

        Ok(())
    }

    fn get_permission_inner(&self, action: ActionType) -> String {
        match action {
            ActionType::General(id) | ActionType::GeneralAllowed(id) => {
                format!("INNER JOIN warehouse_permission ON warehouse_permission.user_id={id} AND warehouse_permission.warehouse_id=inventory.warehouse_id")
            }
            ActionType::Admin | ActionType::System => String::new(),
        }
    }

    const SELECT: &'static str = "SELECT
    inventory.warehouse_id,
    inventory.sku_id,
    inventory.sku_category_id,
    inventory.quantity,

    warehouses.name AS warehouse_name,
    sku_list.name AS sku_name,
    sku_categories.name AS sku_category_name
    
    FROM inventory
    INNER JOIN warehouses ON inventory.warehouse_id=warehouses.id
    INNER JOIN sku_list ON inventory.sku_id=sku_list.id
    INNER JOIN sku_categories ON inventory.sku_category_id=sku_categories.id";

    pub async fn list(
        &self,
        pagination: &Pagination,
        query: &GetInventoryQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<InventoryProduct>> {
        let select = Self::SELECT;
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let inner = self.get_permission_inner(action);
        let rows = sqlx::query(&format!("{select} {inner} {qw} {ob} LIMIT ? OFFSET ?"))
        .bind(pagination.limit())
        .bind(pagination.offset())
            .fetch_all(&mut *tx)
            .await?;
        let mut arr = Vec::with_capacity(rows.len());
        for row in rows {
            arr.push(InventoryProduct::from_row(&row).unwrap())
        }

        Ok(arr)
    }

    pub async fn get_excel(
        &self,
        query: &GetInventoryQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<PathBuf> {
        use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, Worksheet};

        let inner = self.get_permission_inner(action);
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let rows = sqlx::query(&format!("SELECT * FROM inventory {inner} {qw} {ob}"))
            .fetch_all(&mut *tx)
            .await?;
        let mut arr = Vec::with_capacity(rows.len());
        for row in rows {
            arr.push(InventoryProduct::from_row(&row).unwrap())
        }

        // Create a new Excel file object.
        let mut workbook = Workbook::new();
        // Create some formats to use in the worksheet.
        let header_format = Format::new()
            .set_background_color(Color::Theme(4, 0))
            .set_font_color(Color::Theme(0, 0))
            .set_border(FormatBorder::Thin)
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter);
        let data_format = Format::new()
            .set_border(FormatBorder::Thin)
            .set_align(FormatAlign::Center)
            .set_align(FormatAlign::VerticalCenter);

        // Add a worksheet to the workbook.
        let worksheet: &mut Worksheet = workbook.add_worksheet();
        worksheet.write_row_with_format(
            0,
            0,
            ["Warehouse", "SKU", "SKU Category", "Quantity"],
            &header_format,
        )?;
        let depl = self.dependency.read().await;
        let dep = depl.as_ref().unwrap();
        let mut warehouses: HashMap<i64, Warehouse> = HashMap::new();
        let mut skus: HashMap<i64, SKU> = HashMap::new();
        let mut sku_categories: HashMap<i64, SKUCategory> = HashMap::new();
        for (i, p) in arr.iter().enumerate() {
            let warehouse = warehouses
                .entry(p.warehouse_id)
                .or_insert(dep.warehouse.get(p.warehouse_id, tx).await?.unwrap());
            let sku = skus
                .entry(p.sku_id)
                .or_insert(dep.sku.get(p.sku_id, tx).await?.unwrap());
            let sku_category = sku_categories
                .entry(p.sku_category_id)
                .or_insert(dep.sku_category.get(p.sku_category_id, tx).await?.unwrap());

            let row = (i + 1) as u32;
            worksheet.write_row_with_format(
                row,
                0,
                [&warehouse.name, &sku.name, &sku_category.name],
                &data_format,
            )?;
            worksheet.write_with_format(row, 3, p.quantity, &data_format)?;
        }
        let excels = self.ps.get_data_path().join("excels").join("inventory");
        if !excels.is_dir() {
            fs::create_dir_all(&excels).await?;
        }
        let path = excels.join(format!(
            "inventory-{}.xlsx",
            self.ps.get_timestamp_seconds()
        ));
        if path.is_file() {
            fs::remove_file(&path).await?;
        }
        let mut file = fs::File::create(&path).await?;
        let buffer = workbook.save_to_buffer()?;
        file.write_all(&buffer).await?;
        Ok(path)
    }

    pub async fn clear_cache(&self) -> Result<()> {
        let excels = self.ps.get_data_path().join("excels");
        if excels.is_dir() {
            fs::remove_dir_all(&excels).await?;
        }
        Ok(())
    }

    pub async fn get_count(
        &self,
        query: &GetInventoryQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<i64> {
        let s = Self::SELECT;
        let qw = query.get_where_condition();
        let inner = self.get_permission_inner(action);
        let row = sqlx::query(&format!(
            "SELECT count(*) as count FROM ({s} {inner} {qw}) AS tbl"
        ))
        .fetch_one(&mut *tx)
        .await?;
        Ok(row.get("count"))
    }
}
