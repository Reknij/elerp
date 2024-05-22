use elerp_common::{
    order_module::model::order::{Order, OrderCurrency, OrderItem, OrderPaymentStatus, OrderType},
    order_payment_module::model::order_payment::OrderPayment,
    sql,
};

mod common;

#[tokio::test]
async fn test_preprocess() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let mut payment = OrderPayment {
        id: 0,
        created_by_user_id: 0,
        order_id: 0,
        warehouse_id: 0,
        person_in_charge_id: 0,
        creation_date: 0,
        actual_date: 0,
        total_amount: 0.0,
        remark: "HelloWorld".to_owned(),
    };
    c.order_payment.preprocess(&mut payment, &p.user1);
    assert!(payment.creation_date > 0);
    assert_eq!(payment.created_by_user_id, p.user1.id);
}

#[tokio::test]
async fn test_module() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let mut tx = c.ps.begin_tx(true).await.unwrap();
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
        items: Some(vec![OrderItem {
            sku_id: p.sku1.id,
            quantity: 1314,
            price: 1.0,
            exchanged: false,
        }]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing stock out..."),
        order_type: OrderType::StockOut,
    };
    c.order.preprocess(&mut order, &p.user1, true, p.person1.person_in_charge_id);
    let order = c.order.add(order, tx.as_mut()).await.unwrap();

    let mut payment = OrderPayment {
        id: 0,
        created_by_user_id: 0,
        order_id: order.id,
        warehouse_id: 0,
        person_in_charge_id: 0,
        creation_date: 0,
        actual_date: 0,
        total_amount: 314.0,
        remark: "HelloWorld".to_owned(),
    };
    c.order_payment.preprocess(&mut payment, &p.user1);
    let payment = c.order_payment.add(payment, tx.as_mut()).await.unwrap();
    assert_eq!(payment.id, sql::get_standard_id(1));
    assert!(payment.creation_date > 0);
    assert_eq!(payment.warehouse_id, order.warehouse_id);

    let order = c.order.get(order.id, tx.as_mut()).await.unwrap().unwrap();
    assert_eq!(order.total_amount_settled, payment.total_amount);
    assert_eq!(order.order_payment_status, OrderPaymentStatus::PartialSettled);

    let mut payment2 = OrderPayment {
        id: 0,
        created_by_user_id: 0,
        order_id: order.id,
        warehouse_id: 0,
        person_in_charge_id: 0,
        creation_date: 0,
        actual_date: 0,
        total_amount: 1001.0,
        remark: "HelloWorld".to_owned(),
    };
    c.order_payment.preprocess(&mut payment2, &p.user1);
    let payment2 = c.order_payment.add(payment2, tx.as_mut()).await.unwrap();
    assert_eq!(payment2.id, sql::get_standard_id(2));
    assert!(payment2.creation_date > 0);

    let order = c.order.get(order.id, tx.as_mut()).await.unwrap().unwrap();
    assert_eq!(order.total_amount_settled, payment.total_amount + payment2.total_amount);
    assert_eq!(order.order_payment_status, OrderPaymentStatus::Settled);

    c.order_payment.remove(payment.id, false, tx.as_mut()).await.unwrap();
    let order = c.order.get(order.id, tx.as_mut()).await.unwrap().unwrap();
    assert_eq!(order.total_amount_settled, payment2.total_amount);
    assert_eq!(order.order_payment_status, OrderPaymentStatus::PartialSettled);

    c.order_payment.remove(payment2.id, false, tx.as_mut()).await.unwrap();
    let order = c.order.get(order.id, tx.as_mut()).await.unwrap().unwrap();
    assert_eq!(order.total_amount_settled, 0.0);
    assert_eq!(order.order_payment_status, OrderPaymentStatus::Unsettled);
}
