use anyhow::bail;
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

use crate::public_system::{
    model::{Pagination, WebSocketFlags},
    PublicSystem,
};

use self::model::{GetOrderCategoryQuery, OrderCategory};
pub mod model;
use super::Result;

#[derive(Debug, Clone)]
pub struct OrderCategoryModule {
    ps: PublicSystem,
}

impl OrderCategoryModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let mut tx = ps.get_conn().begin().await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS order_categories(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                color TEXT NULL,
                text_color TEXT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS order_categories_names
    ON order_categories(name);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        tx.commit().await.unwrap();
        Self { ps }
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        self.ps
            .is_exists_in_table("order_categories", "id", id, tx)
            .await
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM order_categories;")
            .fetch_one(&mut *tx)
            .await?
            .get("count");
        Ok(count >= self.ps.get_config().limit.order_categories)
    }

    pub async fn is_exists_name(
        &self,
        name: &str,
        prev: Option<i64>,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        self.ps
            .row_is_duplicate_col_in_table(name, prev, "order_categories", "name", tx)
            .await
    }

    pub async fn add(
        &self,
        mut v: OrderCategory,
        tx: &mut SqliteConnection,
    ) -> Result<OrderCategory> {
        let r = sqlx::query(
            "INSERT INTO order_categories (name, description, color, text_color) VALUES(?, ?, ?, ?)",
        )
        .bind(&v.name)
        .bind(&v.description)
        .bind(&v.color)
        .bind(&v.text_color)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add order category");
        }
        v.id = self
            .ps
            .try_set_standard_id(r.last_insert_rowid(), "order_categories", tx)
            .await?;
        self.ps
            .notice(WebSocketFlags::AddOrderCategory(v.id))
            .await?;
        Ok(v)
    }

    pub async fn remove(&self, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        let r = self
            .ps
            .remove_row_from_table(id, "order_categories", tx)
            .await?;
        if notice {
            self.ps
                .notice(WebSocketFlags::RemoveOrderCategory(id))
                .await?;
        }
        Ok(r)
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<OrderCategory>> {
        self.ps
            .get_row_from_table("order_categories", "id", id, tx)
            .await
    }

    pub async fn get_multiple(
        &self,
        pagination: &Pagination,
        query: &GetOrderCategoryQuery,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<OrderCategory>> {
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            order_categories.id,
            order_categories.name,
            order_categories.description,
            order_categories.color,
            order_categories.text_color
            FROM order_categories
            {qw} {ob} LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        self.ps.rows_to_objects(rows)
    }

    pub async fn get_multiple_ids(
        &self,
        pagination: &Pagination,
        query: &GetOrderCategoryQuery,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            id
            FROM order_categories
            {qw}  LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }

    pub async fn get_count(
        &self,
        query: &GetOrderCategoryQuery,
        tx: &mut SqliteConnection,
    ) -> Result<i64> {
        let qw = query.get_where_condition();
        let row = sqlx::query(&format!(
            "SELECT count(*) as count FROM order_categories {qw}"
        ))
        .fetch_one(&mut *tx)
        .await?;
        Ok(row.get("count"))
    }

    pub async fn update(
        &self,
        id: i64,
        mut v: OrderCategory,
        tx: &mut SqliteConnection,
    ) -> Result<Option<OrderCategory>> {
        let r = sqlx::query(
            "UPDATE order_categories SET name=?, description=?, color=?, text_color=? WHERE id=?",
        )
        .bind(&v.name)
        .bind(&v.description)
        .bind(&v.color)
        .bind(&v.text_color)
        .bind(id)
        .execute(&mut *tx)
        .await?;
        Ok(if r.rows_affected() == 1 {
            v.id = id;
            self.ps
                .notice(WebSocketFlags::UpdateOrderCategory(v.id))
                .await?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn is_depend_by_another(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(sqlx::query("SELECT id FROM orders WHERE order_status_id=?")
            .bind(id)
            .fetch(&mut *tx)
            .try_next()
            .await?
            .is_some())
    }
}
