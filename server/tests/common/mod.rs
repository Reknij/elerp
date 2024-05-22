use area_module::AreaModule;
use elerp_common::{
    area_module::model::area::Area,
    get_test_config,
    order_category_module::model::order_category::OrderCategory,
    person_module::model::person::Person,
    sku_category_module::model::sku_category::SKUCategory,
    sku_module::model::sku::SKU,
    user_system::model::user_info::{UserInfo, UserType},
    warehouse_module::model::warehouse::Warehouse,
};
use guest_order_module::GuestOrderModule;
use inventory_module::InventoryModule;
use order_category_module::OrderCategoryModule;
use order_module::OrderModule;
use order_payment_module::OrderPaymentModule;
use person_module::PersonModule;
use public_system::PublicSystem;
use sku_category_module::SKUCategoryModule;
use sku_module::SKUModule;
use user_system::UserSystem;
use warehouse_module::WarehouseModule;

pub struct TestContext {
    pub ps: PublicSystem,
    pub us: UserSystem,
    pub area: AreaModule,
    pub person: PersonModule,
    pub warehouse: WarehouseModule,
    pub sku: SKUModule,
    pub sku_category: SKUCategoryModule,
    pub order: OrderModule,
    pub order_category: OrderCategoryModule,
    pub order_payment: OrderPaymentModule,
    pub guest_order: GuestOrderModule,
    pub inventory: InventoryModule,
}

pub struct TestPrelude {
    pub user1: UserInfo,
    pub user2: UserInfo,
    pub area1: Area,
    pub area2: Area,
    pub person1: Person,
    pub person2: Person,
    pub warehouse1: Warehouse,
    pub warehouse2: Warehouse,
    pub sku_category1: SKUCategory,
    pub sku_category2: SKUCategory,
    pub sku1: SKU,
    pub sku2: SKU,
    pub order_category1: OrderCategory,
    pub order_category2: OrderCategory,
}

pub async fn init_ctx() -> TestContext {
    let config = get_test_config();
    let ps = PublicSystem::new(config.clone()).await;

    TestContext {
        us: UserSystem::new(ps.clone()).await,
        area: AreaModule::new(ps.clone()).await,
        person: PersonModule::new(ps.clone()).await,
        warehouse: WarehouseModule::new(ps.clone()).await,
        sku: SKUModule::new(ps.clone()).await,
        sku_category: SKUCategoryModule::new(ps.clone()).await,
        order: OrderModule::new(ps.clone()).await,
        order_category: OrderCategoryModule::new(ps.clone()).await,
        order_payment: OrderPaymentModule::new(ps.clone()).await,
        guest_order: GuestOrderModule::new(ps.clone()).await,
        inventory: InventoryModule::new(ps.clone()).await,
        ps,
    }
}

pub async fn prelude(ctx: &TestContext) -> TestPrelude {
    let mut tx = ctx.ps.begin_tx(true).await.unwrap();

    let area1 = ctx
        .area
        .add(
            Area {
                id: 0,
                name: "Area 1".to_owned(),
                description: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let area2 = ctx
        .area
        .add(
            Area {
                id: 0,
                name: "Area 2".to_owned(),
                description: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let person1 = ctx
        .person
        .add(
            Person {
                id: 0,
                name: "Person 1".to_owned(),
                description: "".to_owned(),
                address: "".to_owned(),
                area_id: area1.id,
                person_in_charge_id: 0,
                contact: "".to_owned(),
                email: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let person2 = ctx
        .person
        .add(
            Person {
                id: 0,
                name: "Person 2".to_owned(),
                description: "person in charge is person 1.".to_owned(),
                address: "".to_owned(),
                area_id: area2.id,
                person_in_charge_id: person1.id,
                contact: "".to_owned(),
                email: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let warehouse1 = ctx
        .warehouse
        .add(
            Warehouse {
                id: 0,
                name: "Warehouse 1".to_owned(),
                description: "".to_owned(),
                person_in_charge_id: person1.id,
                area_id: area1.id,
                address: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let warehouse2 = ctx
        .warehouse
        .add(
            Warehouse {
                id: 0,
                name: "Warehouse 2".to_owned(),
                description: "".to_owned(),
                person_in_charge_id: person2.id,
                area_id: area2.id,
                address: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let sku_category1 = ctx
        .sku_category
        .add(
            SKUCategory {
                id: 0,
                name: "SKU Category 1".to_owned(),
                description: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let sku_category2 = ctx
        .sku_category
        .add(
            SKUCategory {
                id: 0,
                name: "SKU Category 2".to_owned(),
                description: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let sku1 = ctx
        .sku
        .add(
            SKU {
                id: 0,
                sku_category_id: sku_category1.id,
                name: "SKU 1".to_owned(),
                description: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let sku2 = ctx
        .sku
        .add(
            SKU {
                id: 0,
                sku_category_id: sku_category1.id,
                name: "SKU 2".to_owned(),
                description: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let order_category1 = ctx
        .order_category
        .add(
            OrderCategory {
                id: 0,
                name: "Order Category 1".to_owned(),
                description: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    let order_category2 = ctx
        .order_category
        .add(
            OrderCategory {
                id: 0,
                name: "Order Category 2".to_owned(),
                description: "".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();

    let prelude = TestPrelude {
        user1: ctx
            .us
            .add_user(
                UserInfo {
                    id: 0,
                    alias: "User 1".to_owned(),
                    username: "user1".to_owned(),
                    password: "tester".to_owned(),
                    user_type: UserType::General,
                    permission: 0,
                },
                tx.as_mut(),
            )
            .await
            .unwrap(),
        user2: ctx
            .us
            .add_user(
                UserInfo {
                    id: 0,
                    alias: "User 2".to_owned(),
                    username: "user2".to_owned(),
                    password: "tester".to_owned(),
                    user_type: UserType::General,
                    permission: 0,
                },
                tx.as_mut(),
            )
            .await
            .unwrap(),
        area1,
        area2,
        person1,
        person2,
        warehouse1,
        warehouse2,
        sku_category1,
        sku_category2,
        sku1,
        sku2,
        order_category1,
        order_category2,
    };

    tx.commit().await.unwrap();
    prelude
}
