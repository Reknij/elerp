use elerp_common::model::action_type::ActionType;

mod common;

#[tokio::test]
async fn test_module() {
    let c = common::init_ctx().await;
    let p = common::prelude(&c).await;

    let mut tx = c.ps.begin_tx(true).await.unwrap();
    let token = c.us.get_token(&p.user1, tx.as_mut()).await.unwrap();
    let user1 = c.us.token_to_user(&token, ActionType::System, tx.as_mut()).await.unwrap();
    assert!(user1.is_some());
    let new_token = c.us.get_token(&p.user1, tx.as_mut()).await.unwrap();
    let user1 = c.us.token_to_user(&token, ActionType::System, tx.as_mut()).await.unwrap();
    assert!(user1.is_none(), "Old token is expected inactive.");
    let user1 = c.us.token_to_user(&new_token, ActionType::System, tx.as_mut()).await.unwrap();
    assert!(user1.is_some());

    let user1 = user1.unwrap();
    assert_eq!(user1.id, p.user1.id);

    assert!(!user1.is_connected, "User is expected disconnected!");
    assert!(c.us.try_connect_socket(&user1, tx.as_mut()).await.unwrap());

    let user1 = c.us.get_user(user1.id, ActionType::System, tx.as_mut()).await.unwrap().unwrap();
    assert!(user1.is_connected, "User is expected connected!");

    assert!(c.us.disconnect_socket(&user1, tx.as_mut()).await.unwrap());
    assert!(
        !c.us.get_user(user1.id, ActionType::System, tx.as_mut()).await.unwrap().unwrap().is_connected,
        "User is expected disconnected!"
    );
}
