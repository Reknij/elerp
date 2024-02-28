use anyhow::bail;
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

use crate::public_system::{
    model::{Pagination, WebSocketFlags},
    PublicSystem,
};

use self::model::{GetOrderStatusQuery, OrderStatus};
pub mod model;
use super::Result;

#[derive(Debug, Clone)]
pub struct OrderStatusModule {
    ps: PublicSystem,
}

impl OrderStatusModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let mut tx = ps.get_conn().begin().await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS order_status_list(
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
            "CREATE INDEX IF NOT EXISTS order_status_list_names
    ON order_status_list(name);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        let s = Self { ps };

        let count = s
            .get_count(
                &GetOrderStatusQuery {
                    id: None,
                    name: None,
                    sorters: None,
                },
                tx.as_mut(),
            )
            .await
            .unwrap();
        if count == 0 {
            s.add(
                OrderStatus {
                    id: 1,
                    name: "Preparing".to_owned(),
                    description: "Prepare the order.".to_owned(),
                    color: Some("#4287f5".to_owned()),
                    text_color: Some("white".to_owned()),
                },
                tx.as_mut(),
            )
            .await
            .unwrap();
            s.add(
                OrderStatus {
                    id: 2,
                    name: "Shipping".to_owned(),
                    description: "Shipping the order.".to_owned(),
                    color: Some("#f5c242".to_owned()),
                    text_color: None,
                },
                tx.as_mut(),
            )
            .await
            .unwrap();
            s.add(
                OrderStatus {
                    id: 3,
                    name: "Parcel Received".to_owned(),
                    description: "Order's parcel received.".to_owned(),
                    color: Some("#75f542".to_owned()),
                    text_color: None,
                },
                tx.as_mut(),
            )
            .await
            .unwrap();
        }
        tx.commit().await.unwrap();
        s
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        self.ps.is_exists_in_table("order_status_list", "id", id, tx).await
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM order_status_list;")
            .fetch_one(&mut *tx)
            .await?
            .get("count");
        Ok(count >= self.ps.get_config().limit.order_statuses)
    }

    pub async fn is_exists_name(
        &self,
        name: &str,
        prev: Option<i64>,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        self.ps
            .row_is_duplicate_col_in_table(name, prev, "order_status_list", "name", tx)
            .await
    }

    pub async fn add(&self, mut v: OrderStatus, tx: &mut SqliteConnection) -> Result<OrderStatus> {
        let r = sqlx::query(
            "INSERT INTO order_status_list (name, description, color, text_color) VALUES(?, ?, ?, ?)",
        )
        .bind(&v.name)
        .bind(&v.description)
        .bind(&v.color)
        .bind(&v.text_color)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add order status");
        }
        v.id = self
            .ps
            .try_set_standard_id(r.last_insert_rowid(), "order_status_list", tx)
            .await?;
        self.ps.notice(WebSocketFlags::AddOrderStatus(v.id)).await?;
        Ok(v)
    }

    pub async fn remove(&self, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        let r = self
            .ps
            .remove_row_from_table(id, "order_status_list", tx)
            .await?;
        if notice {
            self.ps
            .notice(WebSocketFlags::RemoveOrderStatus(id))
            .await?;
        }
        Ok(r)
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<OrderStatus>> {
        self.ps
            .get_row_from_table("order_status_list", "id", id, tx)
            .await
    }

    pub async fn get_multiple(
        &self,
        pagination: &Pagination,
        query: &GetOrderStatusQuery,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<OrderStatus>> {
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            order_status_list.id,
            order_status_list.name,
            order_status_list.description,
            order_status_list.color,
            order_status_list.text_color
            FROM order_status_list
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
        query: &GetOrderStatusQuery,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            id
            FROM order_status_list
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
        query: &GetOrderStatusQuery,
        tx: &mut SqliteConnection,
    ) -> Result<i64> {
        let qw = query.get_where_condition();
        let row = sqlx::query(&format!(
            "SELECT count(*) as count FROM order_status_list {qw}"
        ))
        .fetch_one(&mut *tx)
        .await?;
        Ok(row.get("count"))
    }

    pub async fn update(
        &self,
        id: i64,
        mut v: OrderStatus,
        tx: &mut SqliteConnection,
    ) -> Result<Option<OrderStatus>> {
        let r = sqlx::query(
            "UPDATE order_status_list SET name=?, description=?, color=?, text_color=? WHERE id=?",
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
                .notice(WebSocketFlags::UpdateOrderStatus(v.id))
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
