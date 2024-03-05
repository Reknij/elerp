use std::sync::Arc;

use super::{
    area_module::AreaModule, guest_order_module::GuestOrderModule, inventory_module::InventoryModule, order_module::OrderModule, order_payment_module::OrderPaymentModule, order_category_module::OrderCategoryModule, person_module::PersonModule, sku_category_module::SKUCategoryModule, sku_module::SKUModule, statistical_module::StatisticalModule, warehouse_module::WarehouseModule
};

#[derive(Debug, Clone)]
pub struct ModuleDependency {
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
