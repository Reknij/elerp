use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::bail;
use futures::TryStreamExt;
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, SqliteConnection};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    myhelper::set_to_string,
    public_system::{
        model::{Pagination, WebSocketFlags},
        PublicSystem,
    },
    user_system::models::user_info::{UserInfo, UserType},
};

use self::model::{
    CheckOrderResult, GetOrdersQuery, ItemNotAvailable, Order, OrderCurrency, OrderItem,
    OrderPaymentStatus, OrderType,
};
use super::{
    dependency::ModuleDependency, warehouse_module::model::warehouse::GetWarehousesQuery,
    ActionType, Result,
};

pub mod model;

#[derive(Debug, Clone)]
pub struct OrderModule {
    ps: PublicSystem,
    dependency: Arc<RwLock<Option<ModuleDependency>>>,
}

impl OrderModule {
    pub async fn set_dependency(&self, dep: ModuleDependency) {
        *self.dependency.write().await = Some(dep);
    }
    pub async fn new(ps: PublicSystem) -> Self {
        let mut tx = ps.begin_tx(true).await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS orders(
                id INTEGER PRIMARY KEY,
                from_guest_order_id INT NOT NULL,
                created_by_user_id INT NOT NULL,
                updated_by_user_id INT NOT NULL,
                warehouse_id INT NOT NULL,
                total_amount REAL NOT NULL,
                total_amount_settled REAL NOT NULL,
                order_payment_status TEXT NOT NULL,
                currency TEXT NOT NULL,
                person_related_id INT NOT NULL,
                person_in_charge_id INT NOT NULL,
                date INT NOT NULL,
                last_updated_date INT NOT NULL,
                description TEXT NOT NULL,
                order_type TEXT NOT NULL,
                order_category_id INT NOT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS order_warehouses
        ON orders(warehouse_id);
        CREATE INDEX IF NOT EXISTS order_currencies
        ON orders(currency);
        CREATE INDEX IF NOT EXISTS order_person_related_ids
        ON orders(person_related_id);
        CREATE INDEX IF NOT EXISTS order_person_in_charge_ids
        ON orders(person_in_charge_id);
        CREATE INDEX IF NOT EXISTS order_order_types
        ON orders(order_type);
        CREATE INDEX IF NOT EXISTS order_order_category_ids
        ON orders(order_category_id);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS order_items(
                order_id INT NOT NULL,
                sku_id INT NOT NULL,
                sku_category_id INT NOT NULL,
                quantity INT NOT NULL,
                price REAL NOT NULL,
                amount REAL NOT NULL,
                exchanged BOOLEAN NOT NULL
            )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS order_items_order_ids
        ON order_items(order_id);
        CREATE INDEX IF NOT EXISTS order_items_skus
        ON order_items(sku_id);
        CREATE INDEX IF NOT EXISTS order_items_sku_categories
        ON order_items(sku_category_id);
        CREATE INDEX IF NOT EXISTS order_items_exchanged
        ON order_items(exchanged);",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        let s = Self {
            ps: ps.clone(),
            dependency: Arc::new(RwLock::new(None)),
        };

        tx.commit().await.unwrap();
        s
    }

    pub async fn get_order_payment_status(
        &self,
        id: i64,
        tx: &mut SqliteConnection,
    ) -> Result<Option<OrderPaymentStatus>> {
        Ok(
            sqlx::query("SELECT order_payment_status FROM orders WHERE id = ?")
                .bind(id)
                .fetch(&mut *tx)
                .try_next()
                .await?
                .map(|row| row.get("order_payment_status")),
        )
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        self.ps.is_exists_in_table("orders", "id", id, tx).await
    }

    pub async fn is_from_guest_order(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        if let Some(row) = sqlx::query("SELECT from_guest_order_id FROM orders WHERE id=?")
            .bind(id)
            .fetch(&mut *tx)
            .try_next()
            .await?
        {
            let guest_id: i64 = row.get("from_guest_order_id");
            Ok(guest_id > 0)
        } else {
            Ok(false)
        }
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM orders;")
            .fetch_one(&mut *tx)
            .await?
            .get("count");
        Ok(count >= self.ps.get_config().limit.orders)
    }

    fn calc(&self, items: &Vec<OrderItem>) -> f64 {
        let mut total: f64 = 0.0;
        for item in items.iter() {
            if item.exchanged {
                continue;
            }
            total += (item.quantity as f64) * item.price;
        }
        total
    }

    pub fn preprocess(
        &self,
        order: &mut Order,
        user: &UserInfo,
        initial: bool,
        person_in_charge_id: i64,
    ) {
        if let Some(items) = order.items.clone() {
            order.items = Some(
                items
                    .into_iter()
                    .filter(|item| {
                        let pass_quantity = match order.order_type {
                            OrderType::StockIn | OrderType::StockOut | OrderType::Return => {
                                item.quantity > 0
                            }
                            _ => true,
                        };
                        let pass_exchange = if order.order_type != OrderType::Exchange {
                            !item.exchanged
                        } else {
                            true
                        };
                        let pass_sku_not_empty = item.sku_id > 0;

                        pass_quantity && pass_exchange && pass_sku_not_empty
                    })
                    .collect(),
            );
        };

        let now = self.ps.get_timestamp_seconds() as i64;
        order.updated_by_user_id = user.id;
        if initial {
            order.date = now;
            order.created_by_user_id = user.id;
        }
        order.last_updated_date = now;
        order.person_in_charge_id = person_in_charge_id;
        order.from_guest_order_id = 0;
    }

    pub async fn can_access(
        &self,
        id: i64,
        user: &UserInfo,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        Ok(user.user_type == UserType::Admin
            || sqlx::query("SELECT id FROM orders WHERE id=? AND created_by_user_id=? LIMIT 1")
                .bind(id)
                .bind(user.id)
                .fetch(&mut *tx)
                .try_next()
                .await?
                .is_some())
    }

    pub async fn add(&self, mut order: Order, tx: &mut SqliteConnection) -> Result<Order> {
        let depl = self.dependency.read().await;
        let dep = depl.as_ref().unwrap();
        let items = order.items.as_ref();
        let total_amount = if let Some(items) = items {
            dep.inventory
                .change(order.warehouse_id, items, order.order_type, tx)
                .await?;
            self.calc(items)
        } else {
            0.0
        };
        let order_payment_status = if order.order_type == OrderType::StockOut && total_amount > 0.0
        {
            OrderPaymentStatus::Unsettled
        } else {
            OrderPaymentStatus::None
        };
        let r = sqlx::query("INSERT INTO orders (from_guest_order_id, created_by_user_id, updated_by_user_id, warehouse_id, currency, total_amount, person_related_id, person_in_charge_id, date, last_updated_date, description, order_type, order_category_id, total_amount_settled, order_payment_status) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(order.from_guest_order_id)
        .bind(order.created_by_user_id)
        .bind(order.updated_by_user_id)
            .bind(order.warehouse_id)
            .bind(&order.currency)
            .bind(total_amount)
            .bind(order.person_related_id)
            .bind(order.person_in_charge_id)
            .bind(order.date)
            .bind(order.last_updated_date)
            .bind(&order.description)
            .bind(&order.order_type)
            .bind(self.ps.get_standard_id(1))
            .bind(0)
            .bind(order_payment_status)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() != 1 {
            bail!("Can't insert the order to history!");
        }

        order.id = self
            .ps
            .try_set_standard_id(r.last_insert_rowid(), "orders", tx)
            .await?;
        order.total_amount = total_amount;
        order.total_amount_settled = 0.0;
        order.order_payment_status = order_payment_status;

        self.add_order_items(&order, tx).await?;

        self.ps.notice(WebSocketFlags::AddOrder(order.id)).await?;
        Ok(order)
    }

    async fn add_order_items(&self, order: &Order, tx: &mut SqliteConnection) -> Result<()> {
        let items = order.items.as_ref();
        if items.is_none() || items.as_ref().unwrap().len() == 0 {
            return Ok(());
        }
        let mut query_builder = QueryBuilder::new("INSERT INTO order_items (order_id, sku_id, sku_category_id, quantity, price, exchanged, amount) ");
        query_builder.push_values(items.unwrap(), |mut b, item| {
            b.push_bind(order.id)
                .push_bind(item.sku_id)
                .push(format!(
                    "(SELECT sku_category_id FROM sku_list WHERE id = {})",
                    item.sku_id
                ))
                .push_bind(item.quantity)
                .push_bind(item.price)
                .push_bind(item.exchanged)
                .push_bind(item.quantity as f64 * item.price);
        });
        let query = query_builder.build();
        query.execute(&mut *tx).await?;
        Ok(())
    }

    async fn exists_order_type(
        &self,
        ot: OrderType,
        order_start: &Order,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        Ok(
            sqlx::query("SELECT id FROM orders WHERE order_type=? AND date>=? AND id<>? LIMIT 1")
                .bind(ot)
                .bind(order_start.date)
                .bind(order_start.id)
                .fetch(&mut *tx)
                .try_next()
                .await?
                .is_some(),
        )
    }

    pub async fn recall(
        &self,
        mut order: Order,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        let depl = self.dependency.read().await;
        let dep = depl.as_ref().unwrap();
        if !self
            .exists_order_type(OrderType::Calibration, &order, tx)
            .await?
        {
            let warehouse_id = order.warehouse_id;
            let mut _empty_arr = vec![];
            let items = order.items.as_mut().unwrap_or(&mut _empty_arr);
            match &order.order_type {
                OrderType::Return | OrderType::StockIn => {
                    dep.inventory
                        .change(warehouse_id, items, OrderType::StockOut, tx)
                        .await?;
                }
                OrderType::StockOut => {
                    dep.inventory
                        .change(warehouse_id, items, OrderType::StockIn, tx)
                        .await?;
                }
                OrderType::Exchange => {
                    for item in items.iter_mut() {
                        item.exchanged = !item.exchanged;
                    }
                    dep.inventory
                        .change(warehouse_id, items, OrderType::Exchange, tx)
                        .await?;
                }
                OrderType::Calibration => {
                    let mut skus = HashSet::with_capacity(items.len());
                    for item in items {
                        skus.insert(item.sku_id);
                    }

                    self.recalc_all(
                        Some(HashSet::from_iter([order.warehouse_id])),
                        Some(skus),
                        Some(&order),
                        action,
                        tx,
                    )
                    .await?;
                }
                OrderType::CalibrationStrict => {
                    self.recalc_all(
                        Some(HashSet::from_iter([order.warehouse_id])),
                        None,
                        Some(&order),
                        action,
                        tx,
                    )
                    .await?;
                }
                OrderType::Verification | OrderType::VerificationStrict => (),
            }
        }
        Ok(true)
    }

    pub async fn recalc_all(
        &self,
        warehouse_ids: Option<HashSet<i64>>,
        skus_filter: Option<HashSet<i64>>,
        to_remove: Option<&Order>,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<()> {
        let depl = self.dependency.read().await;
        let dep = depl.as_ref().unwrap();
        match skus_filter {
            Some(skus) => {
                let skus = set_to_string(&skus, ",");
                match &warehouse_ids {
                    Some(ids) => {
                        let ids = set_to_string(&ids, ",");
                        sqlx::query(&format!("UPDATE inventory SET quantity=0 WHERE warehouse_id IN ({ids}) AND sku_id IN ({skus})"))
                        .execute(&mut *tx)
                        .await?;
                    }
                    None => {
                        sqlx::query("UPDATE inventory SET quantity=0")
                            .execute(&mut *tx)
                            .await?;
                    }
                }
            }
            None => match &warehouse_ids {
                Some(ids) => {
                    let ids = set_to_string(&ids, ",");
                    sqlx::query(&format!(
                        "UPDATE inventory SET quantity=0 WHERE warehouse_id IN ({ids})"
                    ))
                    .execute(&mut *tx)
                    .await?;
                }
                None => {
                    sqlx::query("UPDATE inventory SET quantity=0")
                        .execute(&mut *tx)
                        .await?;
                }
            },
        }

        let mut p = Pagination::new(0, 100);

        let warehouse_count = if warehouse_ids.is_none() {
            dep.warehouse
                .get_count(
                    &GetWarehousesQuery {
                        id: None,
                        name: None,
                        person_in_charge_id: None,
                        address: None,
                        area_id: None,
                        sorters: None,
                    },
                    action,
                    &mut *tx,
                )
                .await?
        } else {
            1
        };

        let mut q = GetOrdersQuery::empty();
        q.sorters = Some(vec!["date".to_owned()]);
        q.warehouse_ids = warehouse_ids;

        let order_total = self.get_count(&q, action, tx).await?;

        let mut temp = HashMap::<i64, HashMap<i64, i64>>::with_capacity(warehouse_count as _);
        while p.offset() < order_total {
            let mut orders = self
                .get_multiple(p.next(), &q, ActionType::System, tx)
                .await?;
            for order in orders.iter_mut() {
                if let Some(to_remove) = to_remove {
                    if to_remove.id == order.id {
                        continue;
                    }
                }
                let items = self
                    .get_order_items(order.id, &Pagination::max(), tx)
                    .await?;
                for item in &items {
                    let it = temp
                        .entry(order.warehouse_id)
                        .or_insert(HashMap::with_capacity(items.len()));
                    let qty = it.entry(item.sku_id).or_insert(0);

                    *qty = dep
                        .inventory
                        .calc_quantity_by_order_type(*qty, item, order.order_type)
                        .expect("Calc inventory quantity failed!");
                }
            }
        }
        for (warehouse_id, items) in temp {
            let calibration_items: Vec<OrderItem> = items
                .into_iter()
                .map(|(sku_id, quantity)| OrderItem {
                    sku_id,
                    quantity,
                    price: 0.0,
                    exchanged: false,
                })
                .collect();
            dep.inventory
                .change(
                    warehouse_id,
                    &calibration_items,
                    OrderType::Calibration,
                    &mut *tx,
                )
                .await?;
        }
        self.ps.notice(WebSocketFlags::RecalcOrders).await?;
        Ok(())
    }

    pub async fn remove(
        &self,
        id: i64,
        recall: bool,
        notice: bool,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        if let Some(order) = self.get(id, tx).await? {
            let recalled = if recall {
                self.recall(order, action, tx).await?
            } else {
                true
            };
            if recalled {
                self.ps.remove_row_from_table(id, "orders", tx).await?;
                let r = sqlx::query("DELETE FROM order_items WHERE order_id=?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
                if notice {
                    self.ps.notice(WebSocketFlags::RemoveOrder(id)).await?;
                }
                return Ok(r.rows_affected() > 0);
            }
        }
        Ok(false)
    }

    fn row_to_order(&self, row: SqliteRow) -> Order {
        let id = row.get("id");
        Order {
            id,
            from_guest_order_id: row.get("from_guest_order_id"),
            created_by_user_id: row.get("created_by_user_id"),
            updated_by_user_id: row.get("updated_by_user_id"),
            currency: row.try_get("currency").unwrap_or(OrderCurrency::Unknown),
            total_amount: row.get("total_amount"),
            total_amount_settled: row.get("total_amount_settled"),
            order_payment_status: row.get("order_payment_status"),
            warehouse_id: row.get("warehouse_id"),
            person_related_id: row.get("person_related_id"),
            person_in_charge_id: row.get("person_in_charge_id"),
            date: row.get("date"),
            last_updated_date: row.get("last_updated_date"),
            description: row.get("description"),
            order_type: row.get("order_type"),
            order_category_id: row.get("order_category_id"),
            items: None,
        }
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<Order>> {
        let r = sqlx::query("SELECT * FROM orders WHERE id = ?")
            .bind(id)
            .fetch(&mut *tx)
            .try_next()
            .await?;
        Ok(if let Some(row) = r {
            Some(self.row_to_order(row))
        } else {
            None
        })
    }

    pub async fn get_order_items(
        &self,
        order_id: i64,
        pagination: &Pagination,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<OrderItem>> {
        let rows = sqlx::query("SELECT * FROM order_items WHERE order_id=? LIMIT ? OFFSET ?")
            .bind(order_id)
            .bind(pagination.limit())
            .bind(pagination.offset())
            .fetch_all(&mut *tx)
            .await?;
        let arr = self.ps.rows_to_objects(rows)?;
        Ok(arr)
    }

    fn get_permission_inner(&self, action: ActionType) -> String {
        match action {
            ActionType::General(id) | ActionType::GeneralAllowed(id) => {
                format!("INNER JOIN warehouse_permission
                ON warehouse_permission.user_id={id} AND warehouse_permission.warehouse_id=orders.warehouse_id")
            }
            ActionType::Admin | ActionType::System => String::new(),
        }
    }
    const SELECT_MULTIPLE: &'static str = "
    SELECT
    orders.id,
    orders.from_guest_order_id,
    orders.created_by_user_id,
    orders.updated_by_user_id,
    orders.description,
    orders.date,
    orders.last_updated_date,
    orders.person_related_id,
    orders.person_in_charge_id,
    orders.warehouse_id,
    orders.currency,
    orders.order_type,
    orders.order_category_id,
    orders.total_amount,
    orders.total_amount_settled,
    orders.order_payment_status,
    persons_related.name AS person_related_name,
    COALESCE(persons_in_charge.name, 'Empty') AS person_in_charge_name,
    warehouses.name AS warehouse_name,
    order_categories.name AS status_name
    FROM orders
    ";
    const INNERS: &'static str = "
    INNER JOIN persons AS persons_related ON orders.person_related_id=persons_related.id
    LEFT JOIN persons AS persons_in_charge ON orders.person_in_charge_id=persons_in_charge.id
    INNER JOIN order_categories ON orders.order_category_id=order_categories.id
    INNER JOIN warehouses ON orders.warehouse_id=warehouses.id
    ";

    pub async fn get_multiple(
        &self,
        pagination: &Pagination,
        query: &GetOrdersQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<Order>> {
        let s = Self::SELECT_MULTIPLE;
        let inners = Self::INNERS;
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();

        let inner = self.get_permission_inner(action);

        let rows = sqlx::query(&format!("{s} {inners} {inner} {qw} {ob} LIMIT ? OFFSET ?"))
            .bind(pagination.limit())
            .bind(pagination.offset())
            .fetch_all(&mut *tx)
            .await?;
        let mut arr = Vec::with_capacity(rows.len());
        for row in rows {
            arr.push(self.row_to_order(row))
        }

        Ok(arr)
    }

    pub async fn get_multiple_ids(
        &self,
        pagination: &Pagination,
        query: &GetOrdersQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();

        let inner = self.get_permission_inner(action);

        let rows = sqlx::query(&format!(
            "SELECT id FROM orders {inner} {qw} LIMIT ? OFFSET ?"
        ))
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

    pub async fn get_count(
        &self,
        query: &GetOrdersQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<i64> {
        let s = Self::SELECT_MULTIPLE;
        let inners = Self::INNERS;
        let qw = query.get_where_condition();
        let inner = self.get_permission_inner(action);
        let row = sqlx::query(&format!(
            "SELECT count(*) as count FROM ({s} {inners} {inner} {qw}) AS tbl"
        ))
        .fetch_one(&mut *tx)
        .await?;
        Ok(row.get("count"))
    }

    pub async fn is_check_pass(&self, order: &Order, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(self
            .check(order, true, tx)
            .await?
            .items_not_available
            .is_empty())
    }

    pub async fn check(
        &self,
        order: &Order,
        fast_check: bool,
        tx: &mut SqliteConnection,
    ) -> Result<CheckOrderResult> {
        let mut items_not_available = Vec::new();
        let items = if let Some(items) = order.items.as_ref() {
            items
        } else {
            return Ok(CheckOrderResult {
                items_not_available: vec![],
            });
        };
        let mut item_map = HashMap::with_capacity(items.len());
        for item in items {
            if !item.exchanged {
                item_map
                    .entry(item.sku_id)
                    .and_modify(|q| *q += item.quantity)
                    .or_insert(item.quantity);
            }
        }
        let mut inventory = {
            let depl = self.dependency.read().await;
            let dep = depl.as_ref().unwrap();
            dep.inventory.get_virtual(items.len())
        };
        match order.order_type {
            OrderType::Exchange | OrderType::StockOut => {
                for (sku_id, require_quantity) in item_map {
                    let (latest_quantity, actual_quantity) = inventory
                        .get_mut(order.warehouse_id, sku_id, tx)
                        .await?
                        .map(|p| {
                            (
                                p.change(p.latest_quantity() - require_quantity),
                                p.quantity(),
                            )
                        })
                        .unwrap_or((0, 0));
                    if latest_quantity < 0 {
                        items_not_available.push(ItemNotAvailable {
                            sku_id,
                            require_quantity,
                            actual_quantity,
                        });
                        if fast_check {
                            return Ok(CheckOrderResult {
                                items_not_available,
                            });
                        }
                    }
                }
            }
            OrderType::StockIn
            | OrderType::Return
            | OrderType::Calibration
            | OrderType::CalibrationStrict => (),
            OrderType::Verification => {
                for (sku_id, require_quantity) in item_map {
                    let actual_quantity = inventory
                        .get_mut(order.warehouse_id, sku_id, tx)
                        .await?
                        .map(|p| p.quantity())
                        .unwrap_or(0);
                    if actual_quantity != require_quantity {
                        items_not_available.push(ItemNotAvailable {
                            sku_id,
                            require_quantity,
                            actual_quantity,
                        });
                        if fast_check {
                            return Ok(CheckOrderResult {
                                items_not_available,
                            });
                        }
                    }
                }
            }
            OrderType::VerificationStrict => {
                let mut item_ids = Vec::with_capacity(item_map.len());
                for (sku_id, require_quantity) in item_map {
                    item_ids.push(sku_id);
                    let actual_quantity = inventory
                        .get_mut(order.warehouse_id, sku_id, tx)
                        .await?
                        .map(|p| p.quantity())
                        .unwrap_or(0);
                    if actual_quantity != require_quantity {
                        items_not_available.push(ItemNotAvailable {
                            sku_id,
                            require_quantity,
                            actual_quantity,
                        });
                        if fast_check {
                            return Ok(CheckOrderResult {
                                items_not_available,
                            });
                        }
                    }
                }
                let ids = item_ids
                    .into_iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                if fast_check {
                    if let Some(row) = sqlx::query(&format!("SELECT sku_id, quantity FROM inventory WHERE warehouse_id=? AND sku_id NOT IN ({ids}) AND quantity <> 0 LIMIT 1"))
                        .bind(order.warehouse_id)
                        .fetch(&mut *tx)
                        .try_next()
                        .await?
                    {
                        items_not_available.push(ItemNotAvailable {
                            sku_id: row.get("sku_id"),
                            require_quantity: 0,
                            actual_quantity: row.get("quantity"),
                        });
                        return Ok(CheckOrderResult {
                            items_not_available,
                        });
                    }
                } else {
                    let q = format!("SELECT sku_id, quantity FROM inventory WHERE warehouse_id=? AND sku_id NOT IN ({ids}) AND quantity <> 0");
                    let mut r = sqlx::query(&q).bind(order.warehouse_id).fetch(&mut *tx);
                    while let Some(row) = r.try_next().await? {
                        items_not_available.push(ItemNotAvailable {
                            sku_id: row.get("sku_id"),
                            require_quantity: 0,
                            actual_quantity: row.get("quantity"),
                        });
                    }
                }
            }
        }
        Ok(CheckOrderResult {
            items_not_available,
        })
    }

    pub async fn update(
        &self,
        id: i64,
        mut v: Order,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Option<Order>> {
        let r = match action {
            ActionType::GeneralAllowed(_)|
            ActionType::General(_) => sqlx::query(
                "UPDATE orders SET updated_by_user_id=?, last_updated_date=?, person_related_id=?, person_in_charge_id=?, description=?, currency=?, order_category_id=? WHERE id=?",
            )
            .bind(v.updated_by_user_id)
            .bind(v.last_updated_date)
            .bind(v.person_related_id)
            .bind(v.person_in_charge_id)
            .bind(&v.description)
            .bind(&v.currency)
            .bind(v.order_category_id)
            .bind(id)
            .execute(&mut *tx)
            .await?,
            ActionType::Admin => sqlx::query(
                "UPDATE orders SET updated_by_user_id=?, last_updated_date=?, date=?, person_related_id=?, person_in_charge_id=?, description=?, currency=?, order_category_id=? WHERE id=?",
            )
            .bind(v.updated_by_user_id)
            .bind(v.last_updated_date)
            .bind(v.date)
            .bind(v.person_related_id)
            .bind(v.person_in_charge_id)
            .bind(&v.description)
            .bind(&v.currency)
            .bind(v.order_category_id)
            .bind(id)
            .execute(&mut *tx)
            .await?,
            ActionType::System => sqlx::query(
                "UPDATE orders SET date=?, person_related_id=?, person_in_charge_id=?, description=?, currency=?, order_category_id=?, total_amount_settled=?, order_payment_status=? WHERE id=?",
            )
            .bind(v.date)
            .bind(v.person_related_id)
            .bind(v.person_in_charge_id)
            .bind(&v.description)
            .bind(&v.currency)
            .bind(v.order_category_id)
            .bind(v.total_amount_settled)
            .bind(v.order_payment_status)
            .bind(id)
            .execute(&mut *tx)
            .await?,
        };
        Ok(if r.rows_affected() == 1 {
            v.id = id;
            self.ps.notice(WebSocketFlags::UpdateOrder(v.id)).await?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn is_depend_by_another(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(
            sqlx::query("SELECT id FROM order_payments WHERE order_id=?")
                .bind(id)
                .fetch(&mut *tx)
                .try_next()
                .await?
                .is_some(),
        )
    }
}
