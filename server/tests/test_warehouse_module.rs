use elerp_common::warehouse_module::model::fn_argument::{UserInfoID, WarehouseIsFrom};

mod common;

#[tokio::test]
async fn test_link() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let mut tx = c.ps.begin_tx(true).await.unwrap();

    c.warehouse.link(p.warehouse1.id, p.user1.id, tx.as_mut()).await.unwrap();
    assert!(c.warehouse.is_linked(WarehouseIsFrom::ID(p.warehouse1.id), UserInfoID::ID(p.user1.id), tx.as_mut()).await.unwrap())
}