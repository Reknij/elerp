pub mod models;

use std::{sync::Arc, time::Duration};

use self::models::{
    user_configure::{UserConfigure, UserConfigureDefaults},
    user_info::{GetUsersQuery, UserInfo, UserType},
};
use crate::{
    erp::ActionType,
    public_system::{
        model::{Pagination, WebSocketFlags},
        PublicSystem,
    },
    user_system::models::user_permission::{
        ADD_ORDER, MANAGE_AREA, MANAGE_PERSON, MANAGE_SKU, MANAGE_SKU_CATEGORY, MANAGE_WAREHOUSE,
        UPDATE_REMOVE_ORDER,
    },
};
use anyhow::{bail, Result};
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct UserSystem {
    ps: PublicSystem,
    handle: Option<Arc<JoinHandle<()>>>,
    ping_handle: Option<Arc<JoinHandle<()>>>,
}

impl UserSystem {
    pub async fn new(ps: PublicSystem) -> Self {
        let mut tx = ps.get_conn().begin().await.unwrap();
        let _r = sqlx::query(
            "CREATE TABLE IF NOT EXISTS users(
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                alias TEXT NOT NULL,
                user_type TEXT NOT NULL,
                permission INT NOT NULL
        )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        let _r = sqlx::query(
            "CREATE TABLE IF NOT EXISTS configures(
            user_id INT NOT NULL,
            language TEXT NOT NULL,
            d_order_type TEXT NOT NULL,
            d_order_category_id INT NOT NULL,
            d_warehouse_id INT NOT NULL,
            d_person_related_id INT NOT NULL,
            d_order_currency TEXT NOT NULL
        )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query("DROP TABLE IF EXISTS tokens")
            .execute(tx.as_mut())
            .await
            .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sub_tokens(
                id INTEGER PRIMARY KEY,
                created_at INT NOT NULL,
                user_id INT NOT NULL,
                token TEXT NOT NULL
        )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tokens(
                id INTEGER PRIMARY KEY,
                created_at INT NOT NULL,
                user_id INT NOT NULL,
                token TEXT NOT NULL,
                socket_count INT NOT NULL
        )",
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        let have_admin = sqlx::query("SELECT id FROM users WHERE user_type=?")
            .bind(UserType::Admin)
            .fetch(tx.as_mut())
            .try_next()
            .await
            .unwrap()
            .is_some();

        let mut this = Self {
            ps: ps.clone(),
            handle: None,
            ping_handle: None,
        };

        if !have_admin {
            let username = "admin".to_owned();
            let password = "admin123".to_owned();
            this.add_user(
                UserInfo {
                    id: 0,
                    alias: "Admin".to_owned(),
                    username: username.clone(),
                    password: password.clone(),
                    user_type: UserType::Admin,
                    permission: MANAGE_AREA
                        + MANAGE_PERSON
                        + MANAGE_SKU
                        + MANAGE_SKU_CATEGORY
                        + MANAGE_WAREHOUSE
                        + ADD_ORDER
                        + UPDATE_REMOVE_ORDER,
                },
                tx.as_mut(),
            )
            .await
            .unwrap();
            info!("Admin created, username: {username}, password: {password}");
        }
        tx.commit().await.unwrap();

        let t2 = this.clone();
        this.handle = Some(Arc::new(tokio::spawn(async move {
            loop {
                if let Ok(mut tx) = t2.ps.begin_tx(true).await {
                    if let Err(err) = t2.clear_sub_token(None, true, tx.as_mut()).await {
                        error!("Can't clear sub tokens. {}", err);
                    } else {
                        if let Err(err) = tx.commit().await {
                            error!("Can't commit the transaction for clear sub tokens. {}", err);
                        }
                        info!("Clear sub tokens finish.");
                    }
                } else {
                    error!("Can't get transaction to clear sub tokens.");
                }
                tokio::time::sleep(Duration::from_secs(28800)).await;
            }
        })));
        if let Some(seconds) = ps.get_config().ws.ping {
            this.ping_handle = Some(Arc::new(tokio::spawn(async move {
                loop {
                    if let Err(err) = ps.notice(WebSocketFlags::Ping).await {
                        error!("Send the Ping flag failed, error: {err}");
                    }
                    tokio::time::sleep(Duration::from_secs(seconds as _)).await;
                }
            })));
        }

        this
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM users;")
            .fetch_one(&mut *tx)
            .await?
            .get("count");
        Ok(count >= self.ps.get_config().limit.users)
    }

    pub async fn is_sub_token_active(
        &self,
        token: &str,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        if let Some(row) = sqlx::query("SELECT created_at FROM sub_tokens WHERE token=? LIMIT 1")
            .bind(token)
            .fetch(&mut *tx)
            .try_next()
            .await?
        {
            let now = self.ps.get_timestamp_seconds() as i64;
            let created_at: i64 = row.get("created_at");
            let expired = now < created_at + 28800; // expired after 8 hours
            Ok(expired)
        } else {
            Ok(false)
        }
    }

    pub async fn clear_sub_token(
        &self,
        user_id: Option<i64>,
        only_inactive: bool,
        tx: &mut SqliteConnection,
    ) -> Result<()> {
        let now = self.ps.get_timestamp_seconds() as i64;
        match user_id {
            Some(id) => {
                if only_inactive {
                    sqlx::query(
                        "DELETE FROM sub_tokens WHERE user_id=? AND created_at + 28800 < ?",
                    )
                    .bind(id)
                    .bind(now)
                    .execute(&mut *tx)
                    .await?;
                } else {
                    sqlx::query("DELETE FROM sub_tokens WHERE user_id=?")
                        .bind(id)
                        .execute(&mut *tx)
                        .await?;
                }
            }
            None => {
                if only_inactive {
                    sqlx::query("DELETE FROM sub_tokens WHERE created_at + 28800 < ?")
                        .bind(now)
                        .execute(&mut *tx)
                        .await?;
                } else {
                    sqlx::query("DELETE FROM sub_tokens")
                        .execute(&mut *tx)
                        .await?;
                }
            }
        }
        Ok(())
    }

    pub async fn get_sub_token(
        &self,
        user: &UserInfo,
        tx: &mut SqliteConnection,
    ) -> Result<String> {
        let now = self.ps.get_timestamp_seconds();
        let to_calc = user.id as usize + now as usize;
        let token = self.ps.calculate_hash(&to_calc, true).await.to_string();
        let r = sqlx::query("INSERT INTO sub_tokens (created_at, user_id, token) VALUES (?, ?, ?)")
            .bind(now as i64)
            .bind(user.id)
            .bind(&token)
            .bind(0)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() == 1 {
            Ok(token)
        } else {
            bail!("Can't insert the token to sub_tokens!")
        }
    }

    pub async fn get_sub_token_owner(
        &self,
        token: &str,
        tx: &mut SqliteConnection,
    ) -> Result<Option<UserInfo>> {
        let user_id: i64 = if let Some(row) =
            sqlx::query("SELECT user_id FROM sub_tokens WHERE token=?")
                .bind(token)
                .fetch(&mut *tx)
                .try_next()
                .await?
        {
            row.get("user_id")
        } else {
            0
        };
        if user_id > 0 {
            self.get_user(user_id, ActionType::System, &mut *tx).await
        } else {
            Ok(None)
        }
    }

    pub async fn get_token(&self, user: &UserInfo, tx: &mut SqliteConnection) -> Result<String> {
        let now: u64 = self.ps.get_timestamp_seconds();

        if self.remove_token(user.id, &mut *tx).await? {
            warn!("User '{}' logged in, will refresh token...", &user.username);
            self.ps
                .notice(WebSocketFlags::UserRepeatLogin(user.id))
                .await?;
        }

        let token = self.ps.calculate_hash(&user, true).await.to_string();
        let r = sqlx::query(
            "INSERT INTO tokens (created_at, user_id, token, socket_count) VALUES (?, ?, ?, ?)",
        )
        .bind(now as i64)
        .bind(user.id)
        .bind(&token)
        .bind(0)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() == 1 {
            Ok(token)
        } else {
            bail!("Can't insert the token to tokens!")
        }
    }

    pub async fn try_connect_socket(
        &self,
        user: &UserInfo,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        let r = sqlx::query("SELECT socket_count FROM tokens WHERE user_id=?")
            .bind(user.id)
            .fetch(&mut *tx)
            .try_next()
            .await?;

        if let Some(row) = r {
            let socket_count: i64 = row.get("socket_count");
            if socket_count < 1 {
                let r =
                    sqlx::query("UPDATE tokens SET socket_count=socket_count+1 WHERE user_id=?")
                        .bind(user.id)
                        .execute(&mut *tx)
                        .await?;
                return Ok(r.rows_affected() > 0);
            }
        }
        Ok(false)
    }

    pub async fn disconnect_socket(
        &self,
        user: &UserInfo,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        let r = sqlx::query(
            "UPDATE tokens SET socket_count=socket_count-1 WHERE user_id=? AND socket_count > 0",
        )
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn token_to_user(
        &self,
        token: &str,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Option<UserInfo>> {
        let r = sqlx::query("SELECT user_id FROM tokens WHERE token=?")
            .bind(token)
            .fetch(&mut *tx)
            .try_next()
            .await?;
        Ok(if let Some(row) = r {
            let user_id = row.get("user_id");
            self.get_user(user_id, action, &mut *tx).await?
        } else {
            None
        })
    }

    pub async fn is_socket_connected(
        &self,
        user_id: i64,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        Ok(
            sqlx::query("SELECT user_id FROM tokens WHERE user_id=? AND socket_count > 0")
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await
                .is_ok(),
        )
    }

    pub async fn remove_token(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        let r = sqlx::query("DELETE FROM tokens WHERE user_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn remove_sub_token(&self, token: &str, tx: &mut SqliteConnection) -> Result<bool> {
        let r = sqlx::query("DELETE FROM sub_tokens WHERE token=?")
            .bind(token)
            .execute(&mut *tx)
            .await?;
        Ok(r.rows_affected() > 0)
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        self.ps.is_exists_in_table("users", "id", id, tx).await
    }

    pub async fn is_exists_name(
        &self,
        name: &str,
        prev: Option<i64>,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        self.ps
            .row_is_duplicate_col_in_table(name, prev, "users", "username", &mut *tx)
            .await
    }

    pub async fn add_user(
        &self,
        mut user: UserInfo,
        tx: &mut SqliteConnection,
    ) -> Result<UserInfo> {
        let r = sqlx::query(
            "INSERT INTO users (username, password, alias, user_type, permission) VALUES(?, ?, ?, ?, ?)",
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(&user.alias)
        .bind(&user.user_type)
        .bind(user.permission)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add user");
        }
        user.id = r.last_insert_rowid() as i64;
        self.add_configure(
            UserConfigure {
                user_id: user.id,
                language: "en".to_owned(),
                defaults: UserConfigureDefaults::default(),
            },
            &mut *tx,
        )
        .await?;
        self.ps.notice(WebSocketFlags::AddUser(user.id)).await?;
        Ok(user)
    }

    pub async fn remove_user(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        let r = self.ps.remove_row_from_table(id, "users", &mut *tx).await?;
        self.remove_configure(id, &mut *tx).await?;
        self.ps.notice(WebSocketFlags::RemoveUser(id)).await?;
        Ok(r)
    }

    pub async fn get_user(
        &self,
        id: i64,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Option<UserInfo>> {
        self.ps
            .get_row_from_table("users", "id", id, &mut *tx)
            .await
            .map(|opt| {
                opt.map(|mut user: UserInfo| {
                    match action {
                        ActionType::General(_) | ActionType::GeneralAllowed(_) => {
                            user.username = String::new();
                            user.password = String::new();
                            user.permission = 0;
                        }
                        ActionType::Admin | ActionType::System => (),
                    }
                    user
                })
            })
    }

    pub async fn get_user_by_username(
        &self,
        username: &str,
        tx: &mut SqliteConnection,
    ) -> Result<Option<UserInfo>> {
        self.ps
            .get_row_from_table("users", "username", username.to_owned(), &mut *tx)
            .await
    }

    pub async fn get_users(
        &self,
        pagination: &Pagination,
        query: &GetUsersQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<UserInfo>> {
        let qw = query.get_where_condition();
        let rows = sqlx::query(&format!("SELECT * FROM users {qw} LIMIT ? OFFSET ?"))
            .bind(pagination.limit())
            .bind(pagination.offset())
            .fetch_all(&mut *tx)
            .await?;
        self.ps.rows_to_objects::<UserInfo>(rows).map(|mut items| {
            for item in items.iter_mut() {
                match action {
                    ActionType::General(_) | ActionType::GeneralAllowed(_) => {
                        item.username = String::new();
                        item.password = String::new();
                        item.permission = 0;
                    }
                    ActionType::Admin | ActionType::System => (),
                }
            }
            items
        })
    }

    pub async fn get_users_count(
        &self,
        query: &GetUsersQuery,
        tx: &mut SqliteConnection,
    ) -> Result<i64> {
        let qw = query.get_where_condition();
        let row = sqlx::query(&format!("SELECT count(*) AS count FROM users {qw}"))
            .fetch_one(&mut *tx)
            .await?;
        Ok(row.get("count"))
    }

    pub async fn update_user(
        &self,
        id: i64,
        mut v: UserInfo,
        tx: &mut SqliteConnection,
    ) -> Result<Option<UserInfo>> {
        let r =
            sqlx::query("UPDATE users SET username=?, password=?, alias=?, user_type=?, permission=? WHERE id=?")
                .bind(&v.username)
                .bind(&v.password)
                .bind(&v.alias)
                .bind(&v.user_type)
                .bind(v.permission)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        Ok(if r.rows_affected() == 1 {
            v.id = id;
            self.ps.notice(WebSocketFlags::UpdateUser(id)).await?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn add_configure(
        &self,
        config: UserConfigure,
        tx: &mut SqliteConnection,
    ) -> Result<UserConfigure> {
        let r = sqlx::query("INSERT INTO configures (user_id, language) VALUES(?, ?)")
            .bind(config.user_id)
            .bind(&config.language)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add configure");
        }

        Ok(config)
    }

    pub async fn get_configure(
        &self,
        user_id: i64,
        tx: &mut SqliteConnection,
    ) -> Result<Option<UserConfigure>> {
        let mut r = sqlx::query("SELECT * FROM configures WHERE user_id=?")
            .bind(user_id)
            .fetch(&mut *tx);
        Ok(if let Some(row) = r.try_next().await? {
            Some(UserConfigure {
                user_id,
                language: row.get("language"),
                defaults: UserConfigureDefaults {
                    order_type: row.get("d_order_type"),
                    order_category_id: row.get("d_order_category_id"),
                    warehouse_id: row.get("d_warehouse_id"),
                    person_related_id: row.get("d_person_related_id"),
                    order_currency: row.get("d_order_currency"),
                },
            })
        } else {
            None
        })
    }

    pub async fn update_configure(
        &self,
        user_id: i64,
        config: UserConfigure,
        tx: &mut SqliteConnection,
    ) -> Result<Option<UserConfigure>> {
        let r = sqlx::query("UPDATE configures SET language=?, d_order_type=?, d_order_category_id=?, d_warehouse_id=?, d_person_related_id=?, d_order_currency=? WHERE user_id=?")
            .bind(&config.language)
            .bind(&config.defaults.order_type)
            .bind(config.defaults.order_category_id)
            .bind(config.defaults.warehouse_id)
            .bind(config.defaults.person_related_id)
            .bind(&config.defaults.order_currency)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        Ok(if r.rows_affected() == 1 {
            Some(config)
        } else {
            None
        })
    }

    pub async fn remove_configure(&self, user_id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        let r = sqlx::query("DELETE FROM configures WHERE user_id=?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        Ok(r.rows_affected() == 1)
    }
}
