use elerp_common::{
    model::action_type::ActionType,
    order_module::model::order::{GetOrdersQuery, Order, OrderCurrency, OrderItem, OrderPaymentStatus, OrderType}, warehouse_module::model::fn_argument::{UserInfoID, WarehouseIsFrom},
};

mod common;

#[tokio::test]
async fn test_module() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let mut tx = c.ps.begin_tx(true).await.unwrap();

    for n in 0..10 {
        let mut order = Order {
            id: 0,
            created_by_user_id: 0,
            updated_by_user_id: 0,
            date: 0,
            last_updated_date: 0,
            person_in_charge_id: 0,
            order_category_id: p.order_category1.id,
            from_guest_order_id: 0,
            currency: OrderCurrency::USD,
            items: Some(vec![
                OrderItem {
                    sku_id: p.sku1.id,
                    quantity: 1000,
                    price: 18.5,
                    exchanged: false,
                },
                OrderItem {
                    sku_id: p.sku2.id,
                    quantity: 2500,
                    price: 10.0,
                    exchanged: false,
                },
            ]),
            total_amount: 0.0,
            total_amount_settled: 0.0,
            order_payment_status: OrderPaymentStatus::None,
            warehouse_id: p.warehouse1.id,
            person_related_id: p.person1.id,
            description: format!("Testing order #{n}"),
            order_type: OrderType::StockOut,
            is_record: true,
        };
        c.order.preprocess(&mut order, &p.user1, true, p.person2.id);
        c.order.add(order, tx.as_mut()).await.unwrap();
    }
    let q = GetOrdersQuery::empty();
    let total_count = c.statistical.get_total_count(&q, ActionType::Admin, tx.as_mut()).await.unwrap();
    assert_eq!(total_count.stock_out_count, 10);
    assert_eq!(total_count.any_count, total_count.stock_out_count);
    let total_count = c.statistical.get_total_count(&q, p.user1.as_action_type(true), tx.as_mut()).await.unwrap();
    assert_eq!(total_count.any_count, 0);
    let data = c.statistical.get_total_amount(&q, p.user1.as_action_type(false), tx.as_mut()).await.unwrap();
    assert!(data.is_empty());
    c.warehouse.link(p.warehouse1.id, p.user1.id, tx.as_mut()).await.unwrap();
    assert!(c.warehouse.is_linked(WarehouseIsFrom::ID(p.warehouse1.id), UserInfoID::ID(p.user1.id), tx.as_mut()).await.unwrap());
    let data = c.statistical.get_total_amount(&q, p.user1.as_action_type(true), tx.as_mut()).await.unwrap();
    assert!(!data.is_empty());
    assert_eq!(data[0].any, 435000.0);
}
