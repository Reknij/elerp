use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};

use futures::{SinkExt, StreamExt};
use tracing::{error, info, warn};

use crate::{
    public_system::model::WebSocketFlags,
    services::models::web_socket_flag_json::WebSocketFlagsJson,
};

use super::{models::authenticated_user::AuthenticatedUser, AppState};

pub fn get_services() -> Router<AppState> {
    Router::new()
        .route("/", get(ws_handler))
        .route("/ping", get(ws_ping_handler))
}

async fn ws_ping_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.protocols(["elerp-ws"])
        .on_upgrade(move |socket| ping_socket(socket, addr))
}

async fn ping_socket(mut socket: WebSocket, who: SocketAddr) {
    while socket
        .send(Message::Text("Hello from server!".to_owned()))
        .await
        .is_ok()
    {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await
    }
    // returning from the handler closes the websocket connection
    info!("Ping Websocket context {who} destroyed");
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(s): State<AppState>,
    auth: AuthenticatedUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    info!("{addr} try to connect...");
    ws.protocols(["elerp-ws"])
        .on_upgrade(move |socket| handle_socket(s, auth, socket, addr))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(
    s: AppState,
    auth: AuthenticatedUser,
    mut socket: WebSocket,
    _who: SocketAddr,
) {
    let username = &auth.user.username;
    let success = 'status: {
        if let Ok(mut tx) = s.ps.begin_tx(true).await {
            if let Ok(success) = s.us.try_connect_socket(&auth.user, tx.as_mut()).await {
                if success {
                    if let Err(err) = tx.commit().await {
                        error!("Transaction commit failed, will break web socket: {err}");
                    } else {
                        break 'status true;
                    }
                } else {
                    //can't connect to socket because socket count more than 1.
                    warn!("User '{username}' not allowed to connect socket because socket count more than maximum! Will clear user tokens!");
                    let flag = WebSocketFlags::UserRepeatLogin(auth.user.id);

                    // Notice other logged in device and current trying connect device.
                    s.ps.notice(flag.clone())
                        .await
                        .expect("Notice failed at ws_service:144");
                    let data = serde_json::to_string(&WebSocketFlagsJson::from(flag)).unwrap();
                    if socket.send(Message::Text(data)).await.is_ok() {
                        warn!("Notice User '{username}' failed, will destroy the websocket context...");
                    }
                    // Notice end.

                    if let Ok(_) = s.us.remove_token(auth.user.id, tx.as_mut()).await
                    //remove user token, let user login again.
                    {
                        if let Err(err) = tx.commit().await {
                            error!("Transaction commit failed, will break web socket: {err}");
                            return;
                        }
                    }
                }
            }
        }
        false
    };

    if success {
        let mut rx = s.ps.notication_subscribe().await;
        let (mut sender, mut receiver) = socket.split();

        let name = username.clone();
        let user = auth.user.clone();

        if sender
            .send(Message::Text(
                serde_json::to_string(&WebSocketFlagsJson::from(WebSocketFlags::ReadyAccess))
                    .unwrap(),
            ))
            .await
            .is_ok()
        {
            info!("Ready access noticed {name}..");
        } else {
            info!("Ready access noticed {name} failed");
        }

        let mut recv_task = tokio::spawn(async move {
            if let Some(msg) = receiver.next().await {
                if msg.is_err() {
                    return;
                }
            }
        });
        let s: AppState = s.clone();
        let mut send_task = tokio::spawn(async move {
            while let Ok(flag) = rx.recv().await {
                let mut destroy = false;
                let mut print_info = true;

                match flag {
                    WebSocketFlags::UserRepeatLogin(user_id) => {
                        if user.id == user_id {
                            destroy = true;
                            warn!(
                                "User '{name}' repeat login, will destroy the websocket context..."
                            );
                        } else {
                            continue; //Don't notice other users.
                        }
                    },
                    WebSocketFlags::Ping => print_info = false,
                    _ => (),
                }

                let flag_name = flag.to_string();
                let data = serde_json::to_string(&WebSocketFlagsJson::from(flag)).unwrap(); // convert to json to send to user.

                if sender.send(Message::Text(data)).await.is_ok() {
                    if print_info {
                        info!("Flag '{flag_name}' updated noticed {name}...");
                    }
                } else {
                    warn!("Notice User '{name}' failed, will destroy the websocket context...");
                    // no Error here since the only thing we can do is to close the connection.
                    // If we can not send messages, there is no way to salvage the statemachine anyway.
                    destroy = true;
                }
                if destroy {
                    return;
                }
            }
        });

        // If any one of the tasks exit, abort the other.
        tokio::select! {
            _ = (&mut send_task) => {
                recv_task.abort();
            },
            _ = (&mut recv_task) => {
                send_task.abort();
            }
        }
        if let Ok(mut tx) = s.ps.begin_tx(true).await {
            if let Ok(_) = s.us.disconnect_socket(&user, tx.as_mut()).await {
                if let Err(err) = tx.commit().await {
                    error!("Transaction commit failed, will break web socket: {err}");
                    return;
                }
            }
        }
        info!("User '{username}' disconnected socket...");
        return;
    }
}
