use anyhow::bail;
use elerp_common::sql::{self, get_row_from_table, is_exists_in_table, remove_row_from_table, rows_to_objects};
use elerp_common::{
    model::{Pagination, WebSocketFlags},
    sku_module::model::sku::{GetSKUsQuery, SKU},
};
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

use anyhow::Result;
use public_system::PublicSystem;

#[derive(Debug, Clone)]
pub struct SKUModule {
    ps: PublicSystem,
}

impl SKUModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let conn = ps.get_conn();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sku_list(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                sku_category_id INT NOT NULL,
                description TEXT NOT NULL,
                color TEXT NULL,
                text_color TEXT NULL
            )",
        )
        .execute(conn)
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS sku_list_names
    ON sku_list(name);
    CREATE INDEX IF NOT EXISTS sku_list_categories
    ON sku_list(sku_category_id);",
        )
        .execute(conn)
        .await
        .unwrap();

        Self { ps }
    }

    #[allow(dead_code)]
    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        is_exists_in_table("sku_list", "id", id, tx).await
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM sku_list;").fetch_one(&mut *tx).await?.get("count");
        Ok(count >= self.ps.get_config().limit.skus)
    }

    pub async fn is_exists_name(&self, name: &str, category: i64, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<bool> {
        let q1 = "SELECT id FROM sku_list WHERE name=? AND sku_category_id=? AND id<>? LIMIT 1";
        let q2 = "SELECT id FROM sku_list WHERE name=? AND sku_category_id=? LIMIT 1";
        let mut r = if let Some(prev) = prev {
            sqlx::query(q1).bind(name).bind(category).bind(prev).fetch(&mut *tx)
        } else {
            sqlx::query(q2).bind(name).bind(category).fetch(&mut *tx)
        };
        Ok(r.try_next().await?.is_some())
    }

    pub async fn add(&self, mut v: SKU, tx: &mut SqliteConnection) -> Result<SKU> {
        let r = sqlx::query("INSERT INTO sku_list (name, description, sku_category_id, color, text_color) VALUES(?, ?, ?, ?, ?)")
            .bind(&v.name)
            .bind(&v.description)
            .bind(v.sku_category_id)
            .bind(&v.color)
            .bind(&v.text_color)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add sku");
        }
        v.id = sql::try_set_standard_id(r.last_insert_rowid(), "sku_list", tx).await?;
        self.ps.notice(WebSocketFlags::AddSKU(v.id)).await?;
        Ok(v)
    }

    pub async fn remove(&self, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        let r = remove_row_from_table(id, "sku_list", tx).await?;
        if notice {
            self.ps.notice(WebSocketFlags::RemoveSKU(id)).await?;
        }
        Ok(r)
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<SKU>> {
        get_row_from_table("sku_list", "id", id, tx).await
    }

    pub async fn get_multiple(&self, pagination: &Pagination, query: &GetSKUsQuery, tx: &mut SqliteConnection) -> Result<Vec<SKU>> {
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            sku_list.id,
            sku_list.name,
            sku_list.sku_category_id,
            sku_list.description,
            sku_list.color,
            sku_list.text_color,
            sku_categories.name AS sku_category_name
            FROM sku_list
            INNER JOIN sku_categories ON sku_list.sku_category_id=sku_categories.id
            {qw} {ob} LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        rows_to_objects(rows)
    }

    pub async fn get_multiple_ids(&self, pagination: &Pagination, query: &GetSKUsQuery, tx: &mut SqliteConnection) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            id
            FROM sku_list
            {qw}  LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }

    pub async fn get_count(&self, query: &GetSKUsQuery, tx: &mut SqliteConnection) -> Result<i64> {
        let qw = query.get_where_condition();
        let row = sqlx::query(&format!("SELECT count(*) as count FROM sku_list {qw}")).fetch_one(&mut *tx).await?;
        Ok(row.get("count"))
    }

    pub async fn update(&self, id: i64, mut v: SKU, tx: &mut SqliteConnection) -> Result<Option<SKU>> {
        let r = sqlx::query("UPDATE sku_list SET name=?, description=?, sku_category_id=?, color=?, text_color=? WHERE id=?")
            .bind(&v.name)
            .bind(&v.description)
            .bind(v.sku_category_id)
            .bind(&v.color)
            .bind(&v.text_color)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        Ok(if r.rows_affected() == 1 {
            v.id = id;
            self.ps.notice(WebSocketFlags::UpdateSKU(v.id)).await?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn is_depend_by_another(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(sqlx::query("SELECT sku_id FROM order_items WHERE sku_id=? LIMIT 1")
            .bind(id)
            .fetch(&mut *tx)
            .try_next()
            .await?
            .is_some())
    }
}
