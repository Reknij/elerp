mod common;

use elerp_common::{
    guest_order_module::model::guest_order::{GuestOrder, GuestOrderStatus},
    order_module::model::order::{OrderCurrency, OrderItem, OrderType},
};

#[tokio::test]
async fn test_order_preprocess() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let mut guest = GuestOrder {
        id: 0,
        created_by_user_id: 0,
        person_in_charge_id: 0,
        date: 0,
        confirmed_date: 0,
        sub_token: "testing".to_owned(),
        currency: OrderCurrency::USD,
        warehouse_id: p.warehouse1.id,
        person_related_id: p.person2.id,
        description: "".to_owned(),
        order_type: OrderType::StockOut,
        is_record: false,
        non_payment: false,
        guest_order_status: GuestOrderStatus::Expired,
        order_id: 0,
        order_category_id: p.order_category1.id,
        items: Some(vec![]),
    };
    c.guest_order.preprocess(&mut guest, &p.user1, p.person1.id);
    assert_eq!(guest.created_by_user_id, p.user1.id);
    assert_eq!(guest.person_in_charge_id, p.person1.id);
    assert!(guest.items.is_none())
}

#[tokio::test]
async fn test_module() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let max = c.ps.get_config().limit.guest_orders;

    // Stock in <max> orders.
    for n in 0..max {
        let mut tx = c.ps.begin_tx(true).await.unwrap();

        let mut guest = GuestOrder {
            id: 0,
            created_by_user_id: 0,
            person_in_charge_id: 0,
            date: 0,
            confirmed_date: 0,
            sub_token: format!("testing #{n}"),
            currency: OrderCurrency::USD,
            warehouse_id: p.warehouse1.id,
            person_related_id: p.person2.id,
            description: "".to_owned(),
            order_type: OrderType::StockOut,
            is_record: false,
            non_payment: false,
            guest_order_status: GuestOrderStatus::Expired,
            order_id: 0,
            order_category_id: p.order_category1.id,
            items: Some(vec![OrderItem {
                sku_id: p.sku1.id,
                quantity: 5,
                price: 1.0,
                exchanged: false,
            }]),
        };
        let mut to_confirm = guest.clone();
        c.guest_order.preprocess(&mut guest, &p.user1, p.person1.id);

        let r = c.guest_order.add("testing_changed", guest, tx.as_mut()).await.unwrap();
        assert!(r.date > 0);
        assert!(r.confirmed_date > 0);
        assert_eq!(r.sub_token, "testing_changed");

        let r = c.guest_order.add("testing_changed", to_confirm.clone(), tx.as_mut()).await.unwrap();
        let result = c.guest_order.confirm(r.id, to_confirm.clone(), tx.as_mut()).await.unwrap().unwrap();
        assert!(!result.check_result.items_not_available.is_empty());
        assert!(result.order.is_none());

        to_confirm.order_type = OrderType::Calibration;
        let r = c.guest_order.add("testing_changed", to_confirm.clone(), tx.as_mut()).await.unwrap();
        let result = c.guest_order.confirm(r.id, to_confirm.clone(), tx.as_mut()).await.unwrap().unwrap();
        assert!(result.check_result.items_not_available.is_empty());
        assert_eq!(c.inventory.get(to_confirm.warehouse_id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 5);
        assert!(result.order.is_some());

        to_confirm.order_type = OrderType::StockOut;
        to_confirm.items.as_mut().map(|items| items[0].quantity = 3);
        let r = c.guest_order.add("testing_changed", to_confirm.clone(), tx.as_mut()).await.unwrap();
        let result = c.guest_order.confirm(r.id, to_confirm.clone(), tx.as_mut()).await.unwrap().unwrap();
        assert!(result.check_result.items_not_available.is_empty());
        assert_eq!(c.inventory.get(to_confirm.warehouse_id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 2);
        assert!(result.order.is_some());

        to_confirm.is_record = true;
        to_confirm.items.as_mut().map(|items| items[0].quantity = 3);
        let r = c.guest_order.add("testing_changed", to_confirm.clone(), tx.as_mut()).await.unwrap();
        let result = c.guest_order.confirm(r.id, to_confirm.clone(), tx.as_mut()).await.unwrap().unwrap();
        assert!(result.check_result.items_not_available.is_empty());
        assert_eq!(c.inventory.get(to_confirm.warehouse_id, p.sku1.id, tx.as_mut()).await.unwrap().unwrap().quantity, 2);
        assert!(result.order.is_some());

        tx.commit().await.unwrap();
    }
}
