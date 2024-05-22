pub mod common;

use elerp_common::area_module::model::area::{Area, GetAreasQuery};

#[tokio::test]
async fn test_module() {
    let c = common::init_ctx().await;

    let mut tx = c.ps.begin_tx(false).await.unwrap();
    assert_eq!(c.area.is_limit_reached(tx.as_mut()).await.unwrap(), false);
    tx.commit().await.unwrap();

    let max = c.ps.get_config().limit.areas;
    let mut last_name = "".to_owned();
    let mut last_id = 0;
    for n in 0..max {
        let mut tx = c.ps.begin_tx(true).await.unwrap();
        let r = c
            .area
            .add(
                Area {
                    id: 0,
                    name: format!("Test area #{n}"),
                    description: "testing area".to_owned(),
                    color: None,
                    text_color: None,
                },
                tx.as_mut(),
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();
        last_name = r.name;
        last_id = r.id;
    }
    let mut tx = c.ps.begin_tx(false).await.unwrap();
    let count = c.area.get_count(&GetAreasQuery::default(), tx.as_mut()).await.unwrap();
    assert_eq!(count, max);
    assert_eq!(c.area.is_limit_reached(tx.as_mut()).await.unwrap(), true);
    assert_eq!(c.area.is_exists(last_id, tx.as_mut()).await.unwrap(), true);
    assert_eq!(c.area.is_exists_name(&last_name, None, tx.as_mut()).await.unwrap(), true);
    assert_eq!(c.area.is_exists_name(&last_name, Some(last_id), tx.as_mut()).await.unwrap(), false);

    let row = c.area.get(last_id, tx.as_mut()).await.unwrap();
    assert!(row.is_some())
}
