use area_module::AreaModule;
use elerp_common::{
    area_module::model::area::Area,
    get_test_config,
    model::action_type::ActionType,
    person_module::model::person::{GetPersonsQuery, Person},
};
use person_module::PersonModule;
use public_system::PublicSystem;

#[tokio::test]
async fn test_module() {
    let config = get_test_config();
    let ps = PublicSystem::new(config.clone()).await;
    let m = PersonModule::new(ps.clone()).await;
    let area_m = AreaModule::new(ps.clone()).await;
    let mut tx = ps.begin_tx(true).await.unwrap();
    let area = area_m
        .add(
            Area {
                id: 0,
                name: "Test area for persons".to_owned(),
                description: "testing".to_owned(),
                color: None,
                text_color: None,
            },
            tx.as_mut(),
        )
        .await
        .unwrap();
    assert_eq!(m.is_limit_reached(tx.as_mut()).await.unwrap(), false);
    tx.commit().await.unwrap();

    let max = config.limit.persons;
    let mut last_name = "".to_owned();
    let mut last_id = 0;
    for n in 0..max {
        let mut tx = ps.begin_tx(true).await.unwrap();
        let r = m
            .add(
                Person {
                    id: 0,
                    name: format!("Test area #{n}"),
                    description: "testing area".to_owned(),
                    color: None,
                    text_color: None,
                    address: "address...".to_owned(),
                    area_id: area.id,
                    person_in_charge_id: 0,
                    contact: "0123456789".to_owned(),
                    email: "example@email.com".to_owned(),
                },
                tx.as_mut(),
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();
        last_name = r.name;
        last_id = r.id;
    }
    let mut tx = ps.begin_tx(false).await.unwrap();
    let mut query = GetPersonsQuery::default();
    query.area_id = Some(area.id);
    let count = m.get_count(&query, tx.as_mut()).await.unwrap();
    assert_eq!(count, max);
    assert_eq!(m.is_limit_reached(tx.as_mut()).await.unwrap(), true);
    assert_eq!(m.is_exists(last_id, tx.as_mut()).await.unwrap(), true);
    assert_eq!(
        m.is_exists_name(&last_name, None, tx.as_mut())
            .await
            .unwrap(),
        true
    );
    assert_eq!(
        m.is_exists_name(&last_name, Some(last_id), tx.as_mut())
            .await
            .unwrap(),
        false
    );

    let row = m
        .get(last_id, ActionType::System, tx.as_mut())
        .await
        .unwrap();
    assert!(row.is_some())
}
