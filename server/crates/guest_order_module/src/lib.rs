use anyhow::bail;
use anyhow::Result;
use elerp_common::guest_order_module::model::guest_order::GetGuestOrdersQuery;
use elerp_common::guest_order_module::model::guest_order::GuestOrder;
use elerp_common::guest_order_module::model::guest_order::GuestOrderConfirm;
use elerp_common::guest_order_module::model::guest_order::GuestOrderStatus;
use elerp_common::model::action_type::ActionType;
use elerp_common::model::Pagination;
use elerp_common::model::WebSocketFlags;
use elerp_common::order_module;
use elerp_common::order_module::model::order::Order;
use elerp_common::order_module::model::order::OrderCurrency;
use elerp_common::sql;
use elerp_common::sql::is_exists_in_table;
use elerp_common::sql::remove_row_from_table;
use elerp_common::user_system::model::user_info::UserInfo;
use elerp_common::user_system::model::user_info::UserType;
use futures::TryStreamExt;
use public_system::PublicSystem;
use sqlx::{sqlite::SqliteRow, Row, SqliteConnection};

#[derive(Debug, Clone)]
pub struct GuestOrderModule {
    ps: PublicSystem,
}

impl GuestOrderModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let mut tx = ps.begin_tx(true).await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS guest_orders(
                id INTEGER PRIMARY KEY,
                sub_token TEXT NOT NULL,
                created_by_user_id INT NOT NULL,
                warehouse_id INT NOT NULL,
                currency TEXT NOT NULL,
                person_related_id INT NOT NULL,
                person_in_charge_id INT NOT NULL,
                description TEXT NOT NULL,
                order_type TEXT NOT NULL,
                is_record BOOLEAN NOT NULL,
                non_payment BOOLEAN NOT NULL,
                guest_order_status TEXT NOT NULL,
                order_id INT NOT NULL DEFAULT 0,
                order_category_id INT NOT NULL,
                date INT NOT NULL,
                confirmed_date INT NOT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS guest_orders_warehouses
        ON guest_orders(warehouse_id);
        CREATE INDEX IF NOT EXISTS guest_orders_currencies
        ON guest_orders(currency);
        CREATE INDEX IF NOT EXISTS guest_orders_person_related_ids
        ON guest_orders(person_related_id);
        CREATE INDEX IF NOT EXISTS guest_orders_person_in_charge_ids
        ON guest_orders(person_in_charge_id);
        CREATE INDEX IF NOT EXISTS guest_orders_order_types
        ON guest_orders(order_type);
        CREATE INDEX IF NOT EXISTS guest_orders_status
        ON guest_orders(guest_order_status);
        CREATE INDEX IF NOT EXISTS guest_orders_order_ids
        ON guest_orders(order_id);
        CREATE INDEX IF NOT EXISTS guest_orders_order_category_ids
        ON guest_orders(order_category_id);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        let s = Self { ps: ps.clone() };

        tx.commit().await.unwrap();
        s
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        is_exists_in_table("guest_orders", "id", id, tx).await
    }

    pub async fn get_status(&self, id: i64, tx: &mut SqliteConnection) -> Result<GuestOrderStatus> {
        Ok(sqlx::query("SELECT guest_order_status FROM guest_orders WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?
            .get("guest_order_status"))
    }

    pub async fn is_token_match(&self, id: i64, token: &str, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(sqlx::query("SELECT id FROM guest_orders WHERE id=? AND sub_token=? LIMIT 1")
            .bind(id)
            .bind(token)
            .fetch(&mut *tx)
            .try_next()
            .await?
            .is_some())
    }

    pub fn preprocess(&self, order: &mut GuestOrder, user: &UserInfo, person_in_charge_id: i64) {
        order.created_by_user_id = user.id;
        order.person_in_charge_id = person_in_charge_id;
        order.items = None;
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM guest_orders;").fetch_one(&mut *tx).await?.get("count");
        Ok(count >= self.ps.get_config().limit.guest_orders)
    }

    pub async fn can_access(&self, id: i64, user: &UserInfo, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(user.user_type == UserType::Admin
            || sqlx::query("SELECT id FROM guest_orders WHERE id=? AND created_by_user_id=? LIMIT 1")
                .bind(id)
                .bind(user.id)
                .fetch(&mut *tx)
                .try_next()
                .await?
                .is_some())
    }

    pub async fn is_sub_token_same(&self, id: i64, sub_token: &str, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(sqlx::query("SELECT 1 FROM guest_orders WHERE id = ? AND sub_token = ? LIMIT 1")
            .bind(id)
            .bind(sub_token)
            .fetch_optional(&mut *tx)
            .await?
            .is_some())
    }

    pub async fn add(&self, sub_token: &str, mut order: GuestOrder, tx: &mut SqliteConnection) -> Result<GuestOrder> {
        let now = self.ps.get_timestamp_seconds() as i64;
        let r = sqlx::query("INSERT INTO guest_orders (date, confirmed_date, sub_token, created_by_user_id, warehouse_id, currency, person_related_id, person_in_charge_id, description, order_type, is_record, non_payment, guest_order_status, order_category_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(now)
        .bind(now)
        .bind(sub_token)
        .bind(order.created_by_user_id)
            .bind(order.warehouse_id)
            .bind(&order.currency)
            .bind(order.person_related_id)
            .bind(order.person_in_charge_id)
            .bind(&order.description)
            .bind(&order.order_type)
            .bind(order.is_record)
            .bind(order.non_payment)
            .bind(GuestOrderStatus::Pending)
            .bind(order.order_category_id)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() != 1 {
            bail!("Can't insert the order to guest_orders!");
        }
        order.date = now;
        order.confirmed_date = now;
        order.sub_token = sub_token.to_owned();
        order.guest_order_status = GuestOrderStatus::Pending;
        order.id = sql::try_set_standard_id(r.last_insert_rowid(), "guest_orders", tx).await?;

        self.ps.notice(WebSocketFlags::AddGuestOrder(order.id)).await?;
        Ok(order)
    }

    pub async fn remove(&self, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        if remove_row_from_table(id, "guest_orders", tx).await? {
            if notice {
                self.ps.notice(WebSocketFlags::RemoveGuestOrder(id)).await?;
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub async fn confirm(&self, id: i64, v: GuestOrder, tx: &mut SqliteConnection) -> Result<Option<GuestOrderConfirm>> {
        if let Some(mut guest_order) = self.get(id, &mut *tx).await? {
            guest_order.description = v.description;
            guest_order.items = v.items;

            let order: Order = guest_order.into();

            let mut result = GuestOrderConfirm {
                check_result: order_module::check(&order, false, &mut *tx).await?,
                order: None,
            };
            if result.check_result.items_not_available.is_empty() {
                let order = order_module::add(order, &mut *tx).await?;
                let now = self.ps.get_timestamp_seconds() as i64;
                sqlx::query("UPDATE guest_orders SET order_id=?, guest_order_status=?, confirmed_date=? WHERE id=?")
                    .bind(order.id)
                    .bind(GuestOrderStatus::Confirmed)
                    .bind(now)
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                let go = self.get(id, &mut *tx).await?.expect("Get empty guest order when confirm!");
                result.order = Some(go);
                self.ps.notice(WebSocketFlags::ConfirmGuestOrder(id)).await?;
            }

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn row_to_order(&self, row: SqliteRow, tx: &mut SqliteConnection) -> Result<GuestOrder> {
        let id = row.get("id");
        let date = row.get("date");
        let now = self.ps.get_timestamp_seconds() as i64;
        let mut guest_order_status = row.get("guest_order_status");
        let order_id = row.get("order_id");
        match guest_order_status {
            GuestOrderStatus::Pending => {
                if now > date + 28800 {
                    sqlx::query("UPDATE guest_orders SET guest_order_status=? WHERE id=?")
                        .bind(GuestOrderStatus::Expired)
                        .bind(id)
                        .execute(&mut *tx)
                        .await?;
                    guest_order_status = GuestOrderStatus::Expired;
                }
            }
            GuestOrderStatus::Confirmed | GuestOrderStatus::Expired => (),
        };
        let v = GuestOrder {
            id,
            items: None,
            sub_token: row.get("sub_token"),
            created_by_user_id: row.get("created_by_user_id"),
            currency: row.try_get("currency").unwrap_or(OrderCurrency::Unknown),
            warehouse_id: row.get("warehouse_id"),
            person_related_id: row.get("person_related_id"),
            person_in_charge_id: row.get("person_in_charge_id"),
            description: row.get("description"),
            order_type: row.get("order_type"),
            is_record: row.get("is_record"),
            non_payment: row.get("non_payment"),
            guest_order_status,
            order_id,
            date,
            order_category_id: row.get("order_category_id"),
            confirmed_date: row.get("confirmed_date"),
        };
        Ok(v)
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<GuestOrder>> {
        let r = sqlx::query(
            "SELECT 
            guest_orders.id,
            guest_orders.sub_token,
            guest_orders.created_by_user_id,
            guest_orders.warehouse_id,
            guest_orders.is_record,
            guest_orders.non_payment,
            CASE WHEN guest_order_status='Confirmed' THEN orders.currency ELSE guest_orders.currency END AS currency,
            CASE WHEN guest_order_status='Confirmed' THEN orders.person_related_id ELSE guest_orders.person_related_id END AS person_related_id,
            CASE WHEN guest_order_status='Confirmed' THEN orders.person_in_charge_id ELSE guest_orders.person_in_charge_id END AS person_in_charge_id,
            CASE WHEN guest_order_status='Confirmed' THEN orders.description ELSE guest_orders.description END AS description,
            guest_orders.order_type,
            guest_orders.guest_order_status,
            guest_orders.order_id,
            CASE WHEN guest_order_status='Confirmed' THEN orders.order_category_id ELSE guest_orders.order_category_id END AS order_category_id,
            guest_orders.date,
            guest_orders.confirmed_date FROM guest_orders
            LEFT JOIN orders ON orders.id=order_id
            WHERE guest_orders.id = ?",
        )
        .bind(id)
        .fetch(&mut *tx)
        .try_next()
        .await?;
        Ok(if let Some(row) = r { Some(self.row_to_order(row, tx).await?) } else { None })
    }

    pub async fn get_order_id(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<i64>> {
        let r = sqlx::query("SELECT order_id FROM guest_orders WHERE id = ?").bind(id).fetch(&mut *tx).try_next().await?;
        Ok(if let Some(row) = r { Some(row.get("order_id")) } else { None })
    }

    fn get_permission_inner(&self, action: ActionType) -> String {
        match action {
            ActionType::General(id) | ActionType::GeneralAllowed(id) => {
                format!(
                    "INNER JOIN warehouse_permission
                ON warehouse_permission.user_id={id} AND warehouse_permission.warehouse_id=guest_orders.warehouse_id"
                )
            }
            ActionType::Admin | ActionType::System => String::new(),
        }
    }

    const SELECT_MULTIPLE: &'static str = "
    SELECT
    guest_orders.id,
    guest_orders.sub_token,
    guest_orders.created_by_user_id,
    guest_orders.description,
    guest_orders.person_related_id,
    guest_orders.person_in_charge_id,
    guest_orders.warehouse_id,
    guest_orders.currency,
    guest_orders.order_type,
    guest_orders.is_record,
    guest_orders.non_payment,
    guest_orders.guest_order_status,
    guest_orders.date,
    guest_orders.confirmed_date,
    guest_orders.order_id,
    guest_orders.order_category_id,
    persons_related.name AS person_related_name,
    COALESCE(persons_in_charge.name, 'Empty') AS person_in_charge_name,
    warehouses.name AS warehouse_name
    FROM guest_orders
    INNER JOIN persons AS persons_related ON guest_orders.person_related_id=persons_related.id
    LEFT JOIN persons AS persons_in_charge ON guest_orders.person_in_charge_id=persons_in_charge.id
    INNER JOIN warehouses ON guest_orders.warehouse_id=warehouses.id";

    pub async fn get_multiple(&self, pagination: &Pagination, query: &GetGuestOrdersQuery, action: ActionType, tx: &mut SqliteConnection) -> Result<Vec<GuestOrder>> {
        let s = Self::SELECT_MULTIPLE;
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();

        let inner = self.get_permission_inner(action);

        let rows = sqlx::query(&format!("{s} {inner} {qw} {ob} LIMIT ? OFFSET ?"))
            .bind(pagination.limit())
            .bind(pagination.offset())
            .fetch_all(&mut *tx)
            .await?;
        let mut arr = Vec::with_capacity(rows.len());
        for row in rows {
            arr.push(self.row_to_order(row, tx).await?)
        }

        Ok(arr)
    }

    pub async fn get_multiple_ids(&self, pagination: &Pagination, query: &GetGuestOrdersQuery, action: ActionType, tx: &mut SqliteConnection) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();

        let inner = self.get_permission_inner(action);

        let rows = sqlx::query(&format!("SELECT id FROM guest_orders {inner} {qw} LIMIT ? OFFSET ?"))
            .bind(pagination.limit())
            .bind(pagination.offset())
            .fetch_all(&mut *tx)
            .await?;
        let mut arr = Vec::with_capacity(rows.len());
        for row in rows {
            arr.push(row.get("id"))
        }

        Ok(arr)
    }

    pub async fn get_count(&self, query: &GetGuestOrdersQuery, action: ActionType, tx: &mut SqliteConnection) -> Result<i64> {
        let s = Self::SELECT_MULTIPLE;
        let qw = query.get_where_condition();
        let inner = self.get_permission_inner(action);
        let row = sqlx::query(&format!("SELECT count(*) as count FROM ({s} {inner} {qw}) AS tbl")).fetch_one(&mut *tx).await?;
        Ok(row.get("count"))
    }
}
