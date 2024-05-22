use std::path::PathBuf;

use ahash::{HashMap, HashMapExt};
use anyhow::Result;

use elerp_common::{
    inventory_module::model::inventory::{GetInventoryQuery, InventoryProduct},
    model::{action_type::ActionType, Pagination},
};
use public_system::PublicSystem;
use sqlx::{FromRow, Row, SqliteConnection};
use tokio::{fs, io::AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct InventoryModule {
    ps: PublicSystem,
}

impl InventoryModule {
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
        let s = Self { ps };
        s.create_table(tx.as_mut()).await.unwrap();
        tx.commit().await.unwrap();
        s
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
        .fetch_optional(&mut *tx)
        .await?)
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
        let mut warehouses: HashMap<i64, String> = HashMap::new();
        let mut skus: HashMap<i64, String> = HashMap::new();
        let mut sku_categories: HashMap<i64, String> = HashMap::new();
        for (i, p) in arr.iter().enumerate() {
            let warehouse = if warehouses.contains_key(&p.warehouse_id) {
                warehouses.get(&p.warehouse_id).unwrap()
            } else {
                let v = sqlx::query("SELECT name FROM warehouses WHERE id = ? LIMIT 1")
                    .bind(p.warehouse_id)
                    .fetch_one(&mut *tx)
                    .await?
                    .get("name");
                warehouses.insert(p.warehouse_id, v);
                warehouses.get(&p.warehouse_id).unwrap()
            };
            let sku = if skus.contains_key(&p.sku_id) {
                skus.get(&p.sku_id).unwrap()
            } else {
                let v = sqlx::query("SELECT name FROM sku_list WHERE id = ? LIMIT 1")
                    .bind(p.sku_id)
                    .fetch_one(&mut *tx)
                    .await?
                    .get("name");
                skus.insert(p.sku_id, v);
                skus.get(&p.sku_id).unwrap()
            };
            let sku_category = if sku_categories.contains_key(&p.sku_category_id) {
                sku_categories.get(&p.sku_category_id).unwrap()
            } else {
                let v = sqlx::query("SELECT name FROM sku_categories WHERE id = ? LIMIT 1")
                    .bind(p.sku_category_id)
                    .fetch_one(&mut *tx)
                    .await?
                    .get("name");
                sku_categories.insert(p.sku_category_id, v);
                sku_categories.get(&p.sku_category_id).unwrap()
            };

            let row = (i + 1) as u32;
            worksheet.write_row_with_format(
                row,
                0,
                [warehouse, sku, sku_category],
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
