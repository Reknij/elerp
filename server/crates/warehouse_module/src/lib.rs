use anyhow::bail;
use anyhow::Result;
use elerp_common::model::action_type::ActionType;
use elerp_common::model::Pagination;
use elerp_common::model::WebSocketFlags;
use elerp_common::sql;
use elerp_common::sql::{get_row_from_table, is_exists_in_table, remove_row_from_table, row_is_duplicate_col_in_table, rows_to_objects};
use elerp_common::user_system::model::user_info::UserInfo;
use elerp_common::user_system::model::user_info::UserType;
use elerp_common::warehouse_module::model::fn_argument::UserInfoID;
use elerp_common::warehouse_module::model::fn_argument::WarehouseIsFrom;
use elerp_common::warehouse_module::model::warehouse::GetWarehousesQuery;
use elerp_common::warehouse_module::model::warehouse::Warehouse;
use futures_util::TryStreamExt;
use public_system::PublicSystem;
use sqlx::{Row, SqliteConnection};

#[derive(Debug, Clone)]
pub struct WarehouseModule {
    ps: PublicSystem,
}

impl WarehouseModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let conn = ps.get_conn();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS warehouses(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                person_in_charge_id INT NOT NULL,
                area_id INT NOT NULL,
                address TEXT NOT NULL,
                color TEXT NULL,
                text_color TEXT NULL
            )",
        )
        .execute(conn)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS warehouse_permission(
            warehouse_id INT NOT NULL,
            user_id INT NOT NULL
        )",
        )
        .execute(conn)
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS warehouse_names
    ON warehouses(name);
    CREATE INDEX IF NOT EXISTS warehouse_person_in_charge_ids
    ON warehouses(person_in_charge_id);
    CREATE INDEX IF NOT EXISTS warehouse_area_ids
    ON warehouses(area_id);",
        )
        .execute(conn)
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS warehouse_permission_users
    ON warehouse_permission(user_id);
    CREATE INDEX IF NOT EXISTS warehouse_permission_warehouses
    ON warehouse_permission(warehouse_id);",
        )
        .execute(conn)
        .await
        .unwrap();

        Self { ps }
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM warehouses;").fetch_one(&mut *tx).await?.get("count");
        Ok(count >= self.ps.get_config().limit.warehouses)
    }

    pub async fn is_exists_name(&self, name: &str, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<bool> {
        row_is_duplicate_col_in_table(name, prev, "warehouses", "name", tx).await
    }

    pub async fn add(&self, mut warehouse: Warehouse, tx: &mut SqliteConnection) -> Result<Warehouse> {
        let r = sqlx::query("INSERT INTO warehouses (name, description, address, person_in_charge_id, area_id, color, text_color) VALUES(?, ?, ?, ?, ?, ?, ?)")
            .bind(&warehouse.name)
            .bind(&warehouse.description)
            .bind(&warehouse.address)
            .bind(warehouse.person_in_charge_id)
            .bind(warehouse.area_id)
            .bind(&warehouse.color)
            .bind(&warehouse.text_color)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add warehouse");
        }
        warehouse.id = sql::try_set_standard_id(r.last_insert_rowid(), "warehouses", tx).await?;
        self.ps.notice(WebSocketFlags::AddWarehouse(warehouse.id)).await?;
        Ok(warehouse)
    }

    pub async fn remove(&self, warehouse_id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        let r = remove_row_from_table(warehouse_id, "warehouses", tx).await?;
        if notice {
            self.ps.notice(WebSocketFlags::RemoveWarehouse(warehouse_id)).await?;
        }
        Ok(r)
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<Warehouse>> {
        get_row_from_table("warehouses", "id", id, tx).await
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        is_exists_in_table("warehouses", "id", id, tx).await
    }

    fn get_permission_inner(&self, action: ActionType) -> String {
        match action {
            ActionType::General(id) | ActionType::GeneralAllowed(id) => {
                format!("INNER JOIN warehouse_permission ON warehouse_permission.user_id={id} AND warehouse_permission.warehouse_id=warehouses.id")
            }
            ActionType::Admin | ActionType::System => String::new(),
        }
    }
    pub async fn get_multiple(&self, pagination: &Pagination, query: &GetWarehousesQuery, action: ActionType, tx: &mut SqliteConnection) -> Result<Vec<Warehouse>> {
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let inner = self.get_permission_inner(action);
        let rows = sqlx::query(&format!(
            "SELECT 
            warehouses.id, 
            warehouses.name, 
            warehouses.description, 
            warehouses.person_in_charge_id, 
            warehouses.area_id, 
            warehouses.address, 
            warehouses.color, 
            warehouses.text_color, 
            persons.name AS person_name,
            areas.name AS area_name
            FROM warehouses 
            INNER JOIN persons ON warehouses.person_in_charge_id=persons.id 
            INNER JOIN areas ON warehouses.area_id=areas.id 
            {inner}
            {qw} {ob} LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        rows_to_objects(rows).map(|mut list: Vec<Warehouse>| {
            for v in list.iter_mut() {
                match action {
                    ActionType::General(_) => {
                        v.address = String::new();
                    }
                    ActionType::Admin | ActionType::System | ActionType::GeneralAllowed(_) => (),
                }
            }
            list
        })
    }

    pub async fn get_multiple_ids(&self, pagination: &Pagination, query: &GetWarehousesQuery, action: ActionType, tx: &mut SqliteConnection) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();
        let inner = self.get_permission_inner(action);
        let rows = sqlx::query(&format!(
            "SELECT
            id
            FROM warehouses
            {inner}
            {qw}  LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }

    pub async fn get_count(&self, query: &GetWarehousesQuery, action: ActionType, tx: &mut SqliteConnection) -> Result<i64> {
        let qw = query.get_where_condition();
        let inner = self.get_permission_inner(action);
        let row = sqlx::query(&format!("SELECT count(*) as count FROM warehouses {inner} {qw}")).fetch_one(&mut *tx).await?;
        Ok(row.get("count"))
    }

    pub async fn update(&self, id: i64, mut v: Warehouse, tx: &mut SqliteConnection) -> Result<Option<Warehouse>> {
        let r = sqlx::query("UPDATE warehouses SET name=?, description=?, person_in_charge_id=?, area_id=?, address=?, color=?, text_color=? WHERE id=?")
            .bind(&v.name)
            .bind(&v.description)
            .bind(v.person_in_charge_id)
            .bind(v.area_id)
            .bind(&v.address)
            .bind(&v.color)
            .bind(&v.text_color)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        Ok(if r.rows_affected() == 1 {
            v.id = id;
            self.ps.notice(WebSocketFlags::UpdateWarehouse(v.id)).await?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn is_depend_by_another(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(sqlx::query("SELECT id FROM orders WHERE warehouse_id=?").bind(id).fetch(&mut *tx).try_next().await?.is_some())
    }

    pub async fn link(&self, warehouse_id: i64, user_id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        let is_link = sqlx::query("INSERT INTO warehouse_permission VALUES (?, ?)")
            .bind(warehouse_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?
            .rows_affected()
            == 1;
        if is_link {
            self.ps.notice(WebSocketFlags::LinkedWarehouse(warehouse_id)).await?;
            self.ps.notice(WebSocketFlags::LinkedUser(user_id)).await?;
        }
        Ok(is_link)
    }

    pub async fn unlink(&self, warehouse_id: i64, user_id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        let is_unlink = sqlx::query("DELETE FROM warehouse_permission WHERE warehouse_id=? AND user_id=?;")
            .bind(warehouse_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?
            .rows_affected()
            == 1;
        if is_unlink {
            self.ps.notice(WebSocketFlags::UnlinkedWarehouse(warehouse_id)).await?;
            self.ps.notice(WebSocketFlags::UnlinkedUser(user_id)).await?;
        }
        Ok(is_unlink)
    }

    pub async fn is_linked<'a>(&self, warehouse: WarehouseIsFrom, user: UserInfoID<'a>, tx: &mut SqliteConnection) -> Result<bool> {
        let user_id = match &user {
            UserInfoID::InfoRef(info) => {
                if info.user_type == UserType::Admin {
                    return Ok(true);
                } else {
                    info.id
                }
            }
            UserInfoID::ID(id) => *id,
        };

        let query = match warehouse {
            WarehouseIsFrom::ID(warehouse_id) => sqlx::query("SELECT user_id FROM warehouse_permission WHERE warehouse_id=? AND user_id=?;")
                .bind(warehouse_id)
                .bind(user_id),
            WarehouseIsFrom::Order(order_id) => sqlx::query(
                "SELECT w.user_id
            FROM orders o
            JOIN warehouse_permission w ON o.warehouse_id = w.warehouse_id AND w.user_id=?
            WHERE o.id = 7;",
            )
            .bind(user_id)
            .bind(order_id),
            WarehouseIsFrom::GuestOrder(guest_id) => sqlx::query(
                "SELECT w.user_id
            FROM guest_orders o
            JOIN warehouse_permission w ON o.warehouse_id = w.warehouse_id AND w.user_id=?
            WHERE o.id = 7;",
            )
            .bind(user_id)
            .bind(guest_id),
            WarehouseIsFrom::OrderPayment(payment_id) => sqlx::query(
                "SELECT w.user_id
            FROM order_payments o
            JOIN warehouse_permission w ON o.warehouse_id = w.warehouse_id AND w.user_id=?
            WHERE o.id = 7;",
            )
            .bind(user_id)
            .bind(payment_id),
        };

        Ok(query.fetch(&mut *tx).try_next().await?.is_some())
    }

    pub async fn get_linked_users(&self, warehouse_id: i64, pagination: &Pagination, tx: &mut SqliteConnection) -> Result<Vec<UserInfo>> {
        let rows = sqlx::query(&format!(
            "SELECT users.*, 
            CASE WHEN tokens.user_id IS NULL OR tokens.socket_count < 1 THEN 0 ELSE 1 END AS is_connected FROM users 
            LEFT JOIN tokens ON users.id = tokens.user_id 
            INNER JOIN warehouse_permission
            ON warehouse_permission.warehouse_id = ? AND warehouse_permission.user_id = users.id LIMIT ? OFFSET ?"
        ))
        .bind(warehouse_id)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        rows_to_objects(rows)
    }

    #[allow(dead_code)]
    pub async fn get_linked_warehouses(&self, user_id: i64, pagination: &Pagination, tx: &mut SqliteConnection) -> Result<Vec<Warehouse>> {
        let rows = sqlx::query(&format!(
            "SELECT warehouses.*
            FROM warehouses
            INNER JOIN warehouse_permission
            ON warehouse_permission.user_id = ? AND warehouse_permission.warehouse_id = warehouses.id LIMIT ? OFFSET ?"
        ))
        .bind(user_id)
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        rows_to_objects(rows)
    }

    pub async fn get_linked_users_count(&self, warehouse_id: i64, tx: &mut SqliteConnection) -> Result<i64> {
        let row = sqlx::query(&format!("SELECT count(*) as count FROM warehouse_permission WHERE warehouse_id=?"))
            .bind(warehouse_id)
            .fetch_one(&mut *tx)
            .await?;
        Ok(row.get("count"))
    }
}
