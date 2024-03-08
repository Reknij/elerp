use ahash::{HashMap, HashMapExt};
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};
use std::{borrow::Cow, sync::Arc};
use tokio::{sync::RwLock, task::JoinHandle};

use crate::public_system::{model::WebSocketFlags, PublicSystem};

pub mod model;
use self::model::statistical_data::{
    GetStatisticalDataQuery, PopularSKU, SalesAmountWithCurrency, StatisticalData, StatisticalOrderCountData, StatisticalOrderData
};

use super::{
    order_module::model::{GetOrdersQuery, OrderCurrency, OrderType}, ActionType, Result
};

#[derive(Debug, Clone)]
pub struct StatisticalModule {
    #[allow(dead_code)]
    ps: PublicSystem,
    last_data: Arc<RwLock<HashMap<(ActionType, Cow<'static, String>), StatisticalData>>>,
    handle: Arc<Option<JoinHandle<()>>>,
}

impl StatisticalModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let mut this = Self {
            ps: ps.clone(),
            last_data: Arc::new(RwLock::new(HashMap::new())),
            handle: Arc::new(None),
        };

        let t2 = this.clone();
        let handle = tokio::spawn(async move {
            let mut rx = ps.notication_subscribe().await;
            while let Ok(flag) = rx.recv().await {
                let mut map = t2.last_data.write().await;
                match flag {
                    WebSocketFlags::AddArea(_) => map.values_mut().for_each(|data|data.area_count += 1),
                    WebSocketFlags::RemoveArea(_) => map.values_mut().for_each(|data|data.area_count -= 1),
                    WebSocketFlags::AddPerson(_) => map.values_mut().for_each(|data|data.person_count += 1),
                    WebSocketFlags::RemovePerson(_) => map.values_mut().for_each(|data|data.person_count -= 1),
                    WebSocketFlags::AddSKUCategory(_)  => map.values_mut().for_each(|data|data.sku_category_count += 1),
                    WebSocketFlags::RemoveSKUCategory(_) => map.values_mut().for_each(|data|data.sku_category_count -= 1),
                    WebSocketFlags::AddSKU(_) => map.values_mut().for_each(|data|data.sku_count += 1),
                    WebSocketFlags::RemoveSKU(_) => map.values_mut().for_each(|data|data.sku_count -= 1),
                    
                    WebSocketFlags::AddWarehouse(_)
                    | WebSocketFlags::RemoveWarehouse(_)
                    | WebSocketFlags::AddOrder(_)
                    | WebSocketFlags::RemoveOrder(_)
                    | WebSocketFlags::AddGuestOrder(_)
                    | WebSocketFlags::RemoveGuestOrder(_)
                    | WebSocketFlags::ConfirmGuestOrder(_)
                    | WebSocketFlags::RecalcOrders
                    | WebSocketFlags::AddOrderCategory(_)
                    | WebSocketFlags::RemoveOrderCategory(_)
                    | WebSocketFlags::AddOrderPayment(_)
                    | WebSocketFlags::RemoveOrderPayment(_)
                    | WebSocketFlags::LinkedWarehouse(_)
                    | WebSocketFlags::UnlinkedWarehouse(_) => {
                        map.clear();
                    }

                    _ => (),
                };
            }
        });
        this.handle = Arc::new(Some(handle));
        this
    }

    async fn read(
        &self,
        query: &GetStatisticalDataQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<StatisticalData> {
        let mut order_query = query.get_order_query();
        let order_query_str = order_query.get_where_condition();
        {
            if let Some(data) = self
                .last_data
                .read()
                .await
                .get(&(action, Cow::Borrowed(&order_query_str)))
            {
                return Ok(data.clone());
            }
        }

        let area_count = self.get_count("areas", tx).await?;
        let person_count = self.get_count("persons", tx).await?;
        let warehouse_count = self.get_count("warehouses", tx).await?;
        let sku_count = self.get_count("sku_list", tx).await?;
        let sku_category_count = self.get_count("sku_categories", tx).await?;

        let total_count = self.get_total_count(&order_query, action, tx).await?;

        let order_category_count = self.get_count("order_categories", tx).await?;

        order_query.order_type = Some(OrderType::StockOut);
        let total_amount = self.get_total_amount(&order_query, action, tx).await?;

        let most_popular_skus = self.read_popular_skus(10, &order_query, action, tx).await?;

        let data = StatisticalData {
            area_count,
            person_count,
            warehouse_count,
            sku_category_count,
            sku_count,
            order: StatisticalOrderData {
                total_count,
                total_amount,
            },
            order_category_count,
            most_popular_skus,
        };
        self.last_data
            .write()
            .await
            .insert((action, Cow::Owned(order_query_str)), data.clone());

        Ok(data)
    }

    async fn read_popular_skus(
        &self,
        max: usize,
        query: &GetOrdersQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<PopularSKU>> {
        let qw = query.get_where_condition();
        let inner = self.get_order_inner(action);
        let mut arr = Vec::with_capacity(100);
        let rows = sqlx::query(&format!(
            "SELECT
            orders.warehouse_id,
            oi.sku_id AS id,
            orders.currency AS currency,
            COUNT(DISTINCT orders.id) AS order_count,
            SUM(oi.quantity) AS total_out,
            SUM(oi.amount) / SUM(oi.quantity) AS average_price
            FROM orders
            {inner}
            INNER JOIN order_items oi ON orders.id = oi.order_id
            {qw}
            GROUP BY oi.sku_id, orders.currency
            ORDER BY total_out DESC, average_price DESC
            LIMIT {max};"
        ))
        .fetch_all(&mut *tx)
        .await
        .expect("Get populars items failed");
        for row in rows {
            arr.push(PopularSKU {
                id: row.get("id"),
                currency: row.get("currency"),
                order_count: row.get("order_count"),
                average_price: row.get("average_price"),
                total_out: row.get("total_out"),
            })
        }
        Ok(arr)
    }

    pub async fn get(
        &self,
        query: &GetStatisticalDataQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<StatisticalData> {
        let data = self.read(&query, action, tx).await?;
        Ok(data)
    }

    pub async fn get_total_count(
        &self,
        query: &GetOrdersQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<StatisticalOrderCountData> {
        let qw = query.get_where_condition();
        let inner = self.get_order_inner(action);
        let mut data = StatisticalOrderCountData {
            stock_in_count: 0,
            stock_out_count: 0,
            return_count: 0,
            exchange_count: 0,
            any_count: 0,
            calibration_count: 0,
            calibration_strict_count: 0,
            verification_count: 0,
            verification_strict_count: 0,
        };
        if let Ok(rows) = sqlx::query(&format!(
            "SELECT COUNT(*) as count, order_type
            FROM orders
            {inner}
            {qw} 
            GROUP BY order_type"
        ))
        .fetch_all(&mut *tx)
        .await
        {
            for row in rows {
                let order_type: OrderType = row.get("order_type");
                let count: i64 = row.get("count");
                let ref_count = match order_type {
                    OrderType::StockIn => &mut data.stock_in_count,
                    OrderType::StockOut => &mut data.stock_out_count,
                    OrderType::Return => &mut data.return_count,
                    OrderType::Exchange => &mut data.exchange_count,
                    OrderType::Calibration => &mut data.calibration_count,
                    OrderType::CalibrationStrict => &mut data.calibration_strict_count,
                    OrderType::Verification => &mut data.verification_count,
                    OrderType::VerificationStrict => &mut data.verification_strict_count,
                };
                *ref_count = count;
            }
        }
        Ok(data)
    }

    pub async fn get_total_amount(
        &self,
        query: &GetOrdersQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<SalesAmountWithCurrency>> {
        let qw = query.get_where_condition();
        let inner = self.get_order_inner(action);
        if let Ok(rows) = sqlx::query(&format!(
            "SELECT warehouse_id, currency, SUM(total_amount) AS any,
            SUM(CASE WHEN order_payment_status='Unsettled' THEN total_amount ELSE 0.0 END) AS unsettled,
            SUM(CASE WHEN order_payment_status='Settled' THEN total_amount_settled ELSE 0.0 END) AS settled,
            SUM(CASE WHEN order_payment_status='PartialSettled' THEN total_amount_settled ELSE 0.0 END) AS partial_settled
            FROM orders
            {inner}
            {qw} 
            GROUP BY currency"
        ))
        .fetch_all(&mut *tx)
        .await
        {
            let mut arr: Vec<SalesAmountWithCurrency> = rows
                .into_iter()
                .map(|row| SalesAmountWithCurrency {
                    any: row.get("any"),
                    settled: row.get("settled"),
                    unsettled: row.get("unsettled"),
                    partial_settled: row.get("partial_settled"),
                    currency: row
                        .try_get("currency")
                        .unwrap_or(OrderCurrency::Unknown),
                })
                .collect();
            arr.sort_by(|a, b| b.cmp(a));
            Ok(arr)
        } else {
            Ok(vec![])
        }
    }

    fn get_order_inner(&self, action: ActionType) -> Cow<'static, str> {
        match action {
            ActionType::General(id) | ActionType::GeneralAllowed(id) => {
                format!("INNER JOIN warehouse_permission ON warehouse_permission.user_id={id} AND warehouse_permission.warehouse_id=orders.warehouse_id").into()
            }
            ActionType::Admin | ActionType::System => Cow::Borrowed(""),
        }
    }

    async fn get_count(&self, table: &str, tx: &mut SqliteConnection) -> Result<i64> {
        Ok(sqlx::query(&format!("SELECT COUNT(*) as count FROM {table}")).fetch(&mut *tx).try_next().await?.map(|row|row.get("count")).unwrap_or(0))
    }
}
