use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use strum::AsRefStr;
use utoipa::{IntoParams, ToSchema};

use crate::model::action_type::ActionType;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    Hash,
    sqlx::Type,
    AsRefStr,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
)]
pub enum UserType {
    Admin,
    General,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Hash, FromRow, Clone)]
pub struct UserInfo {
    pub id: i64,
    pub alias: String,
    pub username: String,
    pub password: String,
    pub user_type: UserType,
    pub permission: i64,
    #[serde(default)]
    pub is_connected: bool,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetUsersQuery {
    pub alias: Option<String>,
    pub username: Option<String>,
    pub user_type: Option<UserType>,
    pub sorters: Option<Vec<String>>,
}

impl GetUsersQuery {
    pub fn emptpy() -> Self {
        Self {
            alias: None,
            user_type: None,
            username: None,
            sorters: None,
        }
    }
    pub fn get_where_condition(&self) -> String {
        let mut conditions = Vec::with_capacity(4);
        if let Some(v) = &self.username {
            conditions.push(format!("users.username LIKE '%{v}%'"));
        }
        if let Some(v) = &self.alias {
            conditions.push(format!("users.alias LIKE '%{v}%'"));
        }
        if let Some(v) = &self.user_type {
            conditions.push(format!("users.user_type={}", v.as_ref()));
        }
        if !conditions.is_empty() {
            let c = conditions.join(" AND ");
            format!("WHERE {c}")
        } else {
            "".to_owned()
        }
    }

    pub fn get_order_condition(&self) -> String {
        if self.sorters.is_none() {
            return String::new();
        }
        let mut conditions = vec![];
        for sorter in self.sorters.as_ref().unwrap() {
            let col = sorter.replace(":ascend", "").replace(":descend", "");
            if sorter.contains(":ascend") {
                conditions.push(format!("users.{} ASC", col))
            } else if sorter.contains(":descend") {
                conditions.push(format!("users.{} DESC", col))
            }
        }
        if !conditions.is_empty() {
            let c = conditions.join(", ");
            format!("ORDER BY {c}")
        } else {
            "".to_owned()
        }
    }
}

impl UserInfo {
    pub fn as_action_type(&self, have_permission: bool) -> ActionType {
        match self.user_type {
            UserType::Admin => ActionType::Admin,
            UserType::General => if have_permission {
                ActionType::GeneralAllowed(self.id)
            } else {
                ActionType::General(self.id)
            }
        }
    }
}