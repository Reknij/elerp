use std::sync::Arc;

use area_module::AreaModule;
use guest_order_module::GuestOrderModule;
use inventory_module::InventoryModule;
use order_category_module::OrderCategoryModule;
use order_module::OrderModule;
use order_payment_module::OrderPaymentModule;
use person_module::PersonModule;
use public_system::PublicSystem;
use sku_category_module::SKUCategoryModule;
use sku_module::SKUModule;
use statistical_module::StatisticalModule;
use warehouse_module::WarehouseModule;

type Result<T> = anyhow::Result<T>;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone)]
pub struct ERP {
    pub area: Arc<AreaModule>,
    pub person: Arc<PersonModule>,
    pub warehouse: Arc<WarehouseModule>,
    pub sku_category: Arc<SKUCategoryModule>,
    pub sku: Arc<SKUModule>,
    pub order: Arc<OrderModule>,
    pub guest_order: Arc<GuestOrderModule>,
    pub order_category: Arc<OrderCategoryModule>,
    pub order_payment: Arc<OrderPaymentModule>,
    pub inventory: Arc<InventoryModule>,
    pub statistical: Arc<StatisticalModule>,
}

impl ERP {
    pub async fn new(ps: PublicSystem) -> Self {
        let area = Arc::new(AreaModule::new(ps.clone()).await);
        let person = Arc::new(PersonModule::new(ps.clone()).await);

        let warehouse = Arc::new(WarehouseModule::new(ps.clone()).await);
        let sku_category = Arc::new(SKUCategoryModule::new(ps.clone()).await);
        let sku = Arc::new(SKUModule::new(ps.clone()).await);
        let order_category = Arc::new(OrderCategoryModule::new(ps.clone()).await);
        let order_payment = Arc::new(OrderPaymentModule::new(ps.clone()).await);
        let order = Arc::new(OrderModule::new(ps.clone()).await);
        let guest_order = Arc::new(GuestOrderModule::new(ps.clone()).await);
        let inventory = Arc::new(InventoryModule::new(ps.clone()).await);
        let statistical = Arc::new(StatisticalModule::new(ps.clone()).await);

        ERP {
            area,
            person,
            warehouse,
            sku,
            sku_category,
            order,
            guest_order,
            order_category,
            order_payment,
            inventory,
            statistical,
        }
    }

    pub async fn clear_cache(&self) -> Result<()> {
        self.inventory.clear_cache().await
    }
}
