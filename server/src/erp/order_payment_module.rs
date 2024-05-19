use std::sync::Arc;

use anyhow::bail;
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};
use tokio::sync::RwLock;

use crate::{
    public_system::{
        model::{Pagination, WebSocketFlags},
        PublicSystem,
    },
    user_system::models::user_info::{UserInfo, UserType},
};

use self::model::{GetOrderPaymentsQuery, OrderPayment};
pub mod model;
use super::{
    dependency::ModuleDependency, order_module::model::OrderPaymentStatus, ActionType, Result,
};

#[derive(Debug, Clone)]
pub struct OrderPaymentModule {
    ps: PublicSystem,
    dependency: Arc<RwLock<Option<ModuleDependency>>>,
}

impl OrderPaymentModule {
    pub async fn set_dependency(&self, dep: ModuleDependency) {
        *self.dependency.write().await = Some(dep);
    }

    pub async fn new(ps: PublicSystem) -> Self {
        let mut tx = ps.get_conn().begin().await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS order_payments(
                id INTEGER PRIMARY KEY,
                created_by_user_id INT NOT NULL,
                person_in_charge_id INT NOT NULL,
                warehouse_id INT NOT NULL,
                order_id INT NOT NULL,
                total_amount REAL NOT NULL,
                creation_date INT NOT NULL,
                actual_date INT NOT NULL,
                remark TEXT NOT NULL
            );",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS order_payments_person_in_charge
    ON order_payments(person_in_charge_id);
    CREATE INDEX IF NOT EXISTS order_payments_created_by_user_ids
    ON order_payments(created_by_user_id);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        let s = Self {
            ps,
            dependency: Arc::new(RwLock::new(None)),
        };
        tx.commit().await.unwrap();
        s
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        self.ps
            .is_exists_in_table("order_payments", "id", id, tx)
            .await
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM order_payments;")
            .fetch_one(&mut *tx)
            .await?
            .get("count");
        Ok(count >= self.ps.get_config().limit.order_payments)
    }

    pub fn preprocess(&self, v: &mut OrderPayment, user: &UserInfo) {
        v.creation_date = self.ps.get_timestamp_seconds() as i64;
        v.created_by_user_id = user.id;
    }

    pub async fn can_access(
        &self,
        id: i64,
        user: &UserInfo,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        Ok(user.user_type == UserType::Admin
            || sqlx::query(
                "SELECT id FROM order_payments WHERE id=? AND created_by_user_id=? LIMIT 1",
            )
            .bind(id)
            .bind(user.id)
            .fetch(&mut *tx)
            .try_next()
            .await?
            .is_some())
    }

    pub async fn add(
        &self,
        mut v: OrderPayment,
        tx: &mut SqliteConnection,
    ) -> Result<OrderPayment> {
        let l = self.dependency.read().await;
        let dep = l.as_ref().unwrap();
        Ok(
            if let Some(mut order) = dep.order.get(v.order_id, tx).await? {
                if order.order_payment_status == OrderPaymentStatus::Settled {
                    bail!("Order's payment is settled!");
                }

                let r = sqlx::query(
                "INSERT INTO order_payments (order_id, warehouse_id, created_by_user_id, person_in_charge_id, total_amount, creation_date, actual_date, remark) VALUES(?, ?, ?, ?, ?, ?, ?, ?)",
            )
                .bind(order.id)
                .bind(order.warehouse_id)
                .bind(v.created_by_user_id)
                .bind(v.person_in_charge_id)
                .bind(v.total_amount)
                .bind(v.creation_date)
                .bind(v.actual_date)
                .bind(&v.remark)
                .execute(&mut *tx)
                .await?;
                if r.rows_affected() != 1 {
                    bail!("Can't add order status");
                }

                order.total_amount_settled += v.total_amount;
                order.order_payment_status = match order.total_amount_settled {
                    v if v >= order.total_amount => OrderPaymentStatus::Settled,
                    v if v > 0.0 => OrderPaymentStatus::PartialSettled,
                    _ => OrderPaymentStatus::Unsettled,
                };
                dep.order
                    .update(order.id, order, ActionType::System, tx)
                    .await?;

                v.id = self
                    .ps
                    .try_set_standard_id(r.last_insert_rowid(), "order_payments", tx)
                    .await?;
                self.ps
                    .notice(WebSocketFlags::AddOrderPayment(v.id))
                    .await?;
                v
            } else {
                bail!("Order is not found!");
            },
        )
    }

    pub async fn remove(&self, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        if let Some(op) = self.get(id, tx).await? {
            let r = self
                .ps
                .remove_row_from_table(id, "order_payments", tx)
                .await?;
            if r {
                sqlx::query(
                    "UPDATE orders SET total_amount_settled = total_amount_settled - ?, 
                order_payment_status = CASE 
                WHEN total_amount_settled - ? > total_amount THEN 'Settled'
                WHEN total_amount_settled - ? > 0 THEN 'PartialSettled'
                WHEN total_amount_settled - ? < 1 THEN 'Unsettled' 
                END WHERE id=?",
                )
                .bind(op.total_amount)
                .bind(op.total_amount)
                .bind(op.total_amount)
                .bind(op.total_amount)
                .bind(op.order_id)
                .execute(&mut *tx)
                .await?;
                // order.order_payment_status = match order.total_amount_settled {
                //     v if v >= order.total_amount => OrderPaymentStatus::Settled,
                //     v if v > 0.0 => OrderPaymentStatus::PartialSettled,
                //     _ => OrderPaymentStatus::Unsettled,
                // };
            }
            if notice {
                self.ps
                    .notice(WebSocketFlags::RemoveOrderPayment(id))
                    .await?;
            }
            Ok(r)
        } else {
            Ok(false)
        }
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<OrderPayment>> {
        self.ps
            .get_row_from_table("order_payments", "id", id, tx)
            .await
    }

    fn get_permission_inner(&self, action: ActionType) -> String {
        match action {
            ActionType::General(id) | ActionType::GeneralAllowed(id) => {
                format!("INNER JOIN warehouse_permission
                ON warehouse_permission.user_id={id} AND warehouse_permission.warehouse_id=order_payments.warehouse_id")
            }
            ActionType::Admin | ActionType::System => String::new(),
        }
    }

    pub async fn get_multiple(
        &self,
        pagination: &Pagination,
        query: &GetOrderPaymentsQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<OrderPayment>> {
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let inner = self.get_permission_inner(action);
        let rows = sqlx::query(&format!(
            "SELECT
            order_payments.id,
            order_payments.warehouse_id,
            order_payments.created_by_user_id,
            order_payments.order_id,
            order_payments.person_in_charge_id,
            order_payments.creation_date,
            order_payments.actual_date,
            order_payments.remark,
            order_payments.total_amount
            FROM order_payments
            {inner}
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
        query: &GetOrderPaymentsQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();
        let inner = self.get_permission_inner(action);
        let rows = sqlx::query(&format!(
            "SELECT
            id
            FROM order_payments
            {inner}
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
        query: &GetOrderPaymentsQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<i64> {
        let qw = query.get_where_condition();
        let inner = self.get_permission_inner(action);
        let row = sqlx::query(&format!(
            "SELECT count(*) as count FROM order_payments {inner} {qw}"
        ))
        .fetch_one(&mut *tx)
        .await?;
        Ok(row.get("count"))
    }
}
