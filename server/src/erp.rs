use std::sync::Arc;

use crate::public_system::PublicSystem;

use self::{
    area_module::AreaModule, dependency::ModuleDependency, guest_order_module::GuestOrderModule, inventory_module::InventoryModule, order_module::OrderModule, order_payment_module::OrderPaymentModule, order_status_module::OrderStatusModule, person_module::PersonModule, sku_category_module::SKUCategoryModule, sku_module::SKUModule, statistical_module::StatisticalModule, warehouse_module::WarehouseModule
};

pub mod area_module;
pub mod person_module;

pub mod inventory_module;
pub mod order_module;
pub mod guest_order_module;
pub mod order_payment_module;
pub mod order_status_module;
pub mod sku_category_module;
pub mod sku_module;
pub mod statistical_module;
pub mod warehouse_module;

pub mod dependency;
pub mod util;

type Result<T> = anyhow::Result<T>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionType {
    General(i64),
    GeneralAllowed(i64),
    Admin,
    System,
}

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
    pub order_status: Arc<OrderStatusModule>,
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
        let order_status = Arc::new(OrderStatusModule::new(ps.clone()).await);
        let order_payment = Arc::new(OrderPaymentModule::new(ps.clone()).await);
        let order = Arc::new(OrderModule::new(ps.clone()).await);
        let guest_order = Arc::new(GuestOrderModule::new(ps.clone()).await);
        let inventory = Arc::new(InventoryModule::new(ps.clone()).await);
        let statistical = Arc::new(StatisticalModule::new(ps.clone()).await);

        let dep = ModuleDependency {
            area: area.clone(),
            person: person.clone(),
            warehouse: warehouse.clone(),
            sku: sku.clone(),
            sku_category: sku_category.clone(),
            order: order.clone(),
            guest_order: guest_order.clone(),
            order_status: order_status.clone(),
            order_payment: order_payment.clone(),
            inventory: inventory.clone(),
            statistical: statistical.clone(),
        };
        order.set_dependency(dep.clone()).await;
        guest_order.set_dependency(dep.clone()).await;
        order_payment.set_dependency(dep.clone()).await;
        inventory.set_dependency(dep.clone()).await;
        ERP {
            area,
            person,
            warehouse,
            sku,
            sku_category,
            order,
            guest_order,
            order_status,
            order_payment,
            inventory,
            statistical,
        }
    }

    pub async fn clear_cache(&self) -> Result<()> {
        self.inventory.clear_cache().await
    }
}
