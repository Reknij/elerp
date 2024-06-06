mod common;

use elerp_common::{
    model::action_type::ActionType,
    order_module::model::order::{GetOrdersQuery, Order, OrderCurrency, OrderItem, OrderPaymentStatus, OrderType},
    sql,
};

#[tokio::test]
async fn test_order_preprocess() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

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
                quantity: 100,
                price: 18.5,
                exchanged: true,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 250,
                price: 10.0,
                exchanged: false,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing order #1"),
        order_type: OrderType::StockIn,
        is_record: false,
        non_payment: false,
    };
    c.order.preprocess(&mut order, &p.user1, true, p.person2.id);
    assert_eq!(order.created_by_user_id, p.user1.id);
    assert_eq!(order.updated_by_user_id, p.user1.id);
    assert_eq!(order.person_in_charge_id, p.person2.id);
    assert_eq!(order.from_guest_order_id, 0);
    assert!(order.date > 0);
    assert_eq!(order.items.as_ref().unwrap().len(), 1);
    assert_eq!(order.items.as_ref().unwrap()[0].sku_id, p.sku2.id);
    assert_eq!(order.items.as_ref().unwrap()[0].quantity, 250);
    assert_eq!(order.items.as_ref().unwrap()[0].price, 10.0);
    assert_eq!(order.items.as_ref().unwrap()[0].exchanged, false);
    assert_eq!(order.total_amount, 2500.0);
    assert_eq!(order.total_amount_settled, 0.0);
    assert_eq!(order.order_payment_status, OrderPaymentStatus::Unsettled);

    order.non_payment = true;
    c.order.preprocess(&mut order, &p.user1, true, p.person2.id);
    assert_eq!(order.total_amount, 2500.0);
    assert_eq!(order.total_amount_settled, 0.0);
    assert_eq!(order.order_payment_status, OrderPaymentStatus::None);

    order.items.take();
    order.non_payment = false;
    c.order.preprocess(&mut order, &p.user1, true, p.person2.id);
    assert_eq!(order.total_amount, 0.0);
    assert_eq!(order.total_amount_settled, 0.0);
    assert_eq!(order.order_payment_status, OrderPaymentStatus::None);
}

#[tokio::test]
async fn test_module() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let max = c.ps.get_config().limit.orders;

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
        items: Some(vec![
            OrderItem {
                sku_id: p.sku1.id,
                quantity: 100,
                price: 18.5,
                exchanged: false,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 250,
                price: 10.0,
                exchanged: false,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing order"),
        order_type: OrderType::StockOut,
        is_record: true,
        non_payment: false,
    };
    c.order.preprocess(&mut order, &p.user1, true, p.person1.person_in_charge_id);

    let r = c.order.add(order, tx.as_mut()).await.unwrap();
    assert!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().is_none());
    assert!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().is_none());
    c.order.remove(r.id, true, false, ActionType::System, tx.as_mut()).await.unwrap();
    assert!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().is_none());
    assert!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().is_none());
    tx.commit().await.unwrap();

    // Stock in <max> orders.
    for n in 0..max {
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
            items: Some(vec![
                OrderItem {
                    sku_id: p.sku1.id,
                    quantity: 100,
                    price: 18.5,
                    exchanged: false,
                },
                OrderItem {
                    sku_id: p.sku2.id,
                    quantity: 250,
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
            order_type: OrderType::StockIn,
            is_record: false,
            non_payment: false,
        };
        c.order.preprocess(&mut order, &p.user1, true, p.person1.person_in_charge_id);

        let r = c.order.add(order, tx.as_mut()).await.unwrap();

        assert_eq!(r.total_amount, 4350.0);
        assert_eq!(r.total_amount_settled, 0.0);

        assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 100 * (n + 1));
        assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 250 * (n + 1));

        assert_eq!(c.order.is_limit_reached(tx.as_mut()).await.unwrap(), if n == max - 1 { true } else { false });
        assert!(c.order.is_exists(r.id, tx.as_mut()).await.unwrap());
        assert!(!c.order.is_exists(r.id + 1, tx.as_mut()).await.unwrap());

        tx.commit().await.unwrap();
    }

    let mut tx = c.ps.begin_tx(false).await.unwrap();
    let count = c.order.get_count(&GetOrdersQuery::default(), ActionType::System, tx.as_mut()).await.unwrap();
    assert_eq!(count, max);

    let sku1_qty = 100 * max;
    let sku2_qty = 250 * max;
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku1_qty);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku2_qty);
    tx.commit().await.unwrap();

    // Test other type orders.
    let mut tx = c.ps.begin_tx(true).await.unwrap();
    let mut stock_out_order = Order {
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
                quantity: 50 * max,
                price: 1.0,
                exchanged: false,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 125 * max,
                price: 5.0,
                exchanged: false,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing stock out..."),
        order_type: OrderType::StockOut,
        is_record: false,
        non_payment: false,
    };
    c.order.preprocess(&mut stock_out_order, &p.user1, true, p.person1.person_in_charge_id);
    let stock_out_order = c.order.add(stock_out_order, tx.as_mut()).await.unwrap();
    assert_eq!(stock_out_order.order_payment_status, OrderPaymentStatus::Unsettled);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku1_qty / 2);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku2_qty / 2);

    c.order.remove(stock_out_order.id, true, false, ActionType::System, tx.as_mut()).await.unwrap();
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku1_qty);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku2_qty);

    let mut exchange_order = Order {
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
                quantity: 50 * max,
                price: 0.0,
                exchanged: false,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 125 * max,
                price: 0.0,
                exchanged: true,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing stock out..."),
        order_type: OrderType::Exchange,
        is_record: false,
        non_payment: false,
    };
    c.order.preprocess(&mut exchange_order, &p.user1, true, p.person1.person_in_charge_id);
    let exchange_order = c.order.add(exchange_order, tx.as_mut()).await.unwrap();
    assert_eq!(exchange_order.order_payment_status, OrderPaymentStatus::None);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku1_qty / 2);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 125 * max + sku2_qty);

    c.order.remove(exchange_order.id, true, false, ActionType::System, tx.as_mut()).await.unwrap();
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku1_qty);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, sku2_qty);

    let mut calibration_order = Order {
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
                quantity: 999,
                price: 0.0,
                exchanged: false,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 666,
                price: 0.0,
                exchanged: false,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing stock out..."),
        order_type: OrderType::Calibration,
        is_record: false,
        non_payment: false,
    };
    c.order.preprocess(&mut calibration_order, &p.user1, true, p.person1.person_in_charge_id);
    let calibration_order = c.order.add(calibration_order, tx.as_mut()).await.unwrap();
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 999);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 666);

    let mut stock_in_order = Order {
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
                quantity: 1,
                price: 0.0,
                exchanged: false,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 334,
                price: 0.0,
                exchanged: false,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing stock out..."),
        order_type: OrderType::StockIn,
        is_record: false,
        non_payment: false,
    };
    c.order.preprocess(&mut stock_in_order, &p.user1, true, p.person1.person_in_charge_id);
    let stock_in_order = c.order.add(stock_in_order, tx.as_mut()).await.unwrap();
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 1000);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 1000);

    c.order.remove(calibration_order.id, true, false, ActionType::System, tx.as_mut()).await.unwrap();
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 100 * max + 1);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 250 * max + 334);

    c.order.remove(stock_in_order.id, true, false, ActionType::System, tx.as_mut()).await.unwrap();
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 100 * max);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 250 * max);

    let mut calibration_strict_order = Order {
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
            price: 0.0,
            exchanged: false,
        }]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing stock out..."),
        order_type: OrderType::CalibrationStrict,
        is_record: false,
        non_payment: false,
    };
    c.order.preprocess(&mut calibration_strict_order, &p.user1, true, p.person1.person_in_charge_id);
    let calibration_strict_order = c.order.add(calibration_strict_order, tx.as_mut()).await.unwrap();
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 1314);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 0);

    c.order.remove(calibration_strict_order.id, true, false, ActionType::System, tx.as_mut()).await.unwrap();
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 100 * max);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 250 * max);

    tx.commit().await.unwrap();

    let mut tx = c.ps.begin_tx(true).await.unwrap();
    for n in 0..max {
        let id = sql::get_standard_id(n + 1);
        assert!(c.order.is_exists(id, tx.as_mut()).await.unwrap());
        c.order.remove(id, true, false, ActionType::System, tx.as_mut()).await.unwrap();
        assert!(!c.order.is_exists(id, tx.as_mut()).await.unwrap());
        assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 100 * max - 100 * (n + 1));
        assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 250 * max - 250 * (n + 1));
    }

    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 0);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 0);
}

async fn remove_after_calibration(strict: bool) {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let mut in_order = Order {
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
                quantity: 100,
                price: 18.5,
                exchanged: false,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 250,
                price: 10.0,
                exchanged: false,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing order #1"),
        order_type: OrderType::StockIn,
        is_record: false,
        non_payment: false,
    };

    let mut tx = c.ps.begin_tx(true).await.unwrap();
    c.order.preprocess(&mut in_order, &p.user1, true, p.person2.id);
    let in_order = c.order.add(in_order, tx.as_mut()).await.unwrap();

    let mut out_order = Order {
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
                quantity: 10,
                price: 18.5,
                exchanged: false,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 10,
                price: 10.0,
                exchanged: false,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing order #1"),
        order_type: OrderType::StockOut,
        is_record: false,
        non_payment: false,
    };

    c.order.preprocess(&mut out_order, &p.user1, true, p.person2.id);
    let out_order = c.order.add(out_order, tx.as_mut()).await.unwrap();

    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 90);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 240);

    let mut calibration_order = Order {
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
                quantity: 5,
                price: 18.5,
                exchanged: false,
            },
            OrderItem {
                sku_id: p.sku2.id,
                quantity: 6,
                price: 10.0,
                exchanged: false,
            },
        ]),
        total_amount: 0.0,
        total_amount_settled: 0.0,
        order_payment_status: OrderPaymentStatus::None,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person1.id,
        description: format!("Testing order #1"),
        order_type: if strict { OrderType::CalibrationStrict } else { OrderType::Calibration },
        is_record: false,
        non_payment: false,
    };

    c.order.preprocess(&mut calibration_order, &p.user1, true, p.person2.id);
    let _calibration_order = c.order.add(calibration_order, tx.as_mut()).await.unwrap();

    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 5);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 6);

    c.order.remove(out_order.id, true, false, ActionType::System, tx.as_mut()).await.unwrap();

    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 5);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 6);

    c.order.remove(in_order.id, true, false, ActionType::System, tx.as_mut()).await.unwrap();

    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 5);
    assert_eq!(c.inventory.get(p.warehouse1.id, p.sku2.id, tx.as_mut()).await.unwrap().unwrap().quantity, 6);
}

#[tokio::test]
async fn test_remove_after_calibration() {
    remove_after_calibration(false).await;
}

#[tokio::test]
async fn test_remove_after_calibration_strict() {
    remove_after_calibration(true).await;
}
