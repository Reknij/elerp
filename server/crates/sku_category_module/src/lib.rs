use anyhow::bail;
use elerp_common::sql::{get_row_from_table, is_exists_in_table, remove_row_from_table, row_is_duplicate_col_in_table, rows_to_objects};
use elerp_common::{
    model::{Pagination, WebSocketFlags},
    sku_category_module::model::sku_category::{GetSKUCategoriesQuery, SKUCategory},
    sql,
};
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

use anyhow::Result;
use public_system::PublicSystem;

#[derive(Debug, Clone)]
pub struct SKUCategoryModule {
    ps: PublicSystem,
}

impl SKUCategoryModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let conn = ps.get_conn();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sku_categories(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                color TEXT NULL,
                text_color TEXT NULL
            )",
        )
        .execute(conn)
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS sku_category_names
    ON sku_categories(name);",
        )
        .execute(conn)
        .await
        .unwrap();

        Self { ps }
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        is_exists_in_table("sku_categories", "id", id, tx).await
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM sku_categories;").fetch_one(&mut *tx).await?.get("count");
        Ok(count >= self.ps.get_config().limit.sku_categories)
    }

    pub async fn is_exists_name(&self, name: &str, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<bool> {
        row_is_duplicate_col_in_table(name, prev, "sku_categories", "name", tx).await
    }

    pub async fn add(&self, mut v: SKUCategory, tx: &mut SqliteConnection) -> Result<SKUCategory> {
        let r = sqlx::query("INSERT INTO sku_categories (name, description, color, text_color) VALUES(?, ?, ?, ?)")
            .bind(&v.name)
            .bind(&v.description)
            .bind(&v.color)
            .bind(&v.text_color)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add sku category");
        }
        v.id = sql::try_set_standard_id(r.last_insert_rowid(), "sku_categories", tx).await?;
        self.ps.notice(WebSocketFlags::AddSKUCategory(v.id)).await?;
        Ok(v)
    }

    pub async fn remove(&self, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        let r = remove_row_from_table(id, "sku_categories", tx).await?;
        if notice {
            self.ps.notice(WebSocketFlags::RemoveSKUCategory(id)).await?;
        }
        Ok(r)
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<SKUCategory>> {
        get_row_from_table("sku_categories", "id", id, tx).await
    }

    pub async fn get_multiple(&self, pagination: &Pagination, query: &GetSKUCategoriesQuery, tx: &mut SqliteConnection) -> Result<Vec<SKUCategory>> {
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            sku_categories.id,
            sku_categories.name,
            sku_categories.description,
            sku_categories.color,
            sku_categories.text_color
            FROM sku_categories
            {qw} {ob} LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        rows_to_objects(rows)
    }

    pub async fn get_multiple_ids(&self, pagination: &Pagination, query: &GetSKUCategoriesQuery, tx: &mut SqliteConnection) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            id
            FROM sku_categories
            {qw}  LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }

    pub async fn get_count(&self, query: &GetSKUCategoriesQuery, tx: &mut SqliteConnection) -> Result<i64> {
        let qw = query.get_where_condition();
        let row = sqlx::query(&format!("SELECT count(*) as count FROM sku_categories {qw}")).fetch_one(&mut *tx).await?;
        Ok(row.get("count"))
    }

    pub async fn update(&self, id: i64, mut v: SKUCategory, tx: &mut SqliteConnection) -> Result<Option<SKUCategory>> {
        let r = sqlx::query("UPDATE sku_categories SET name=?, description=?, color=?, text_color=? WHERE id=?")
            .bind(&v.name)
            .bind(&v.description)
            .bind(&v.color)
            .bind(&v.text_color)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        Ok(if r.rows_affected() == 1 {
            v.id = id;
            self.ps.notice(WebSocketFlags::UpdateSKUCategory(v.id)).await?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn is_depend_by_another(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(sqlx::query("SELECT id FROM sku_list WHERE sku_category_id=?").bind(id).fetch(&mut *tx).try_next().await?.is_some())
    }
}
