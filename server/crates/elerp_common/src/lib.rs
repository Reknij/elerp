use ahash::HashSet;

pub mod area_module;
pub mod config;
pub mod guest_order_module;
pub mod inventory_module;
pub mod meta;
pub mod model;
pub mod order_category_module;
pub mod order_module;
pub mod order_payment_module;
pub mod person_module;
pub mod sku_category_module;
pub mod sku_module;
pub mod statistical_module;
pub mod user_system;
pub mod warehouse_module;

pub fn get_test_config() -> config::AppConfig {
    use config::AppConfig;
    use tempfile::tempdir;

    use crate::config::{Limit, Web, TLS, WS};

    let tmp_dir = tempdir().expect("Get temp directory failed!");
    AppConfig {
        data_path: tmp_dir.into_path(),
        web: Web::default(),
        host: "0.0.0.0".to_owned(),
        port: 3345,
        limit: Limit {
            areas: 9,
            persons: 9,
            users: 9,
            warehouses: 9,
            sku_categories: 9,
            skus: 9,
            order_categories: 9,
            order_payments: 9,
            orders: 9,
            guest_orders: 9,
            statistics: 9,
        },
        tls: TLS::default(),
        ws: WS::default(),
    }
}

pub fn i64_safe_max() -> i64 {
    i64::MAX - 1
}

pub fn set_to_string<T: ToString>(set: &HashSet<T>, sep: &str) -> String {
    set.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(sep)
}

pub mod sql {
    use std::{borrow::Cow, sync::OnceLock};

    use ahash::HashSet;
    use anyhow::Result;
    use futures::TryStreamExt;
    use regex::Regex;
    use sqlx::{sqlite::SqliteRow, Encode, FromRow, Sqlite, SqliteConnection, Type};

    const STANDARD_ID_NUM: i64 = 10000;
    static RE: OnceLock<Regex> = OnceLock::new();

    pub fn get_search_where_condition(col: &str, query: &str) -> String {
        let re: &Regex = RE.get_or_init(|| Regex::new(r"[\s+\(\)\-\:\@（）]").unwrap());
        let mut tmp = [0u8; 4];
        let qs: Vec<&str> = query.trim().split(|a: char| re.is_match(a.encode_utf8(&mut tmp))).collect();
        let mut conditions = Vec::with_capacity(qs.len());
        for q in qs {
            conditions.push(format!("{col} LIKE '%{q}%'"));
        }
        conditions.join(" AND ").into()
    }

    pub fn get_sorter_str(sorter: &str) -> &'static str {
        if sorter.contains(":descend") {
            "DESC"
        } else {
            "ASC"
        }
    }

    pub fn get_sort_col_str(col: &str) -> String {
        col.replace(":ascend", "").replace(":descend", "").into()
    }

    pub fn eq_or_not(reverse: Option<&HashSet<String>>, col: &str) -> &'static str {
        match reverse {
            Some(reverse) => {
                if reverse.contains(col) {
                    "<>"
                } else {
                    "="
                }
            }
            None => "=",
        }
    }

    pub fn in_or_not(reverse: Option<&HashSet<String>>, col: &str) -> &'static str {
        match reverse {
            Some(reverse) => {
                if reverse.contains(col) {
                    " NOT IN "
                } else {
                    " IN "
                }
            }
            None => " IN ",
        }
    }

    pub fn exists_or_not(reverse: Option<&HashSet<String>>, col: &str) -> &'static str {
        match reverse {
            Some(reverse) => {
                if reverse.contains(col) {
                    " NOT EXISTS "
                } else {
                    " EXISTS "
                }
            }
            None => " EXISTS ",
        }
    }

    pub fn like_or_not(reverse: Option<&HashSet<String>>, col: &str) -> &'static str {
        match reverse {
            Some(reverse) => {
                if reverse.contains(col) {
                    " NOT LIKE "
                } else {
                    " LIKE "
                }
            }
            None => " LIKE ",
        }
    }

    pub async fn row_is_duplicate_col_in_table(col: &str, prev: Option<i64>, table_name: &str, col_name: &str, tx: &mut SqliteConnection) -> Result<bool> {
        let q1 = format!("SELECT id FROM {table_name} WHERE {col_name}=? AND id<>? LIMIT 1");
        let q2 = format!("SELECT id FROM {table_name} WHERE {col_name}=? LIMIT 1");
        let mut r = if let Some(prev) = prev {
            sqlx::query(&q1).bind(col).bind(prev).fetch(&mut *tx)
        } else {
            sqlx::query(&q2).bind(col).fetch(&mut *tx)
        };
        Ok(r.try_next().await?.is_some())
    }

    pub async fn get_row_from_table<T, V>(table_name: &str, col_name: &str, col_value: V, tx: &mut SqliteConnection) -> Result<Option<T>>
    where
        for<'q> V: 'q + Send + Encode<'q, Sqlite> + Type<Sqlite>,
        for<'r> T: FromRow<'r, SqliteRow> + Unpin + Send,
    {
        let q = format!("SELECT * FROM {table_name} WHERE {col_name} = ?");
        let mut r = sqlx::query(&q).bind(col_value).fetch(&mut *tx);
        Ok(if let Some(row) = r.try_next().await? {
            let v = T::from_row(&row)?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn is_exists_in_table<V>(table_name: &str, col_name: &str, col_value: V, tx: &mut SqliteConnection) -> Result<bool>
    where
        for<'q> V: 'q + Send + Encode<'q, Sqlite> + Type<Sqlite>,
    {
        let q = format!("SELECT {col_name} FROM {table_name} WHERE {col_name} = ?");
        let mut r = sqlx::query(&q).bind(col_value).fetch(&mut *tx);
        Ok(r.try_next().await?.is_some())
    }

    pub fn rows_to_objects<'a, T>(rows: Vec<SqliteRow>) -> Result<Vec<T>>
    where
        for<'r> T: FromRow<'r, SqliteRow> + Unpin + Send + 'a,
    {
        let mut arr = Vec::with_capacity(rows.len());
        for row in rows {
            arr.push(T::from_row(&row)?)
        }

        Ok(arr)
    }

    pub async fn remove_row_from_table(row_id: i64, table_name: &str, tx: &mut SqliteConnection) -> Result<bool> {
        let q = format!("DELETE FROM {table_name} WHERE id = ?");
        let r = sqlx::query(&q).bind(row_id).execute(&mut *tx).await?;
        Ok(r.rows_affected() == 1)
    }

    pub async fn try_set_standard_id(creation_id: i64, table_name: &str, tx: &mut SqliteConnection) -> Result<i64> {
        if creation_id == 1 {
            let nid = STANDARD_ID_NUM + 1;
            sqlx::query(&Cow::Owned(format!("UPDATE {table_name} SET id = {nid} WHERE id = {creation_id}")))
                .execute(&mut *tx)
                .await?;
            Ok(nid)
        } else {
            Ok(creation_id as _)
        }
    }

    pub fn get_standard_id(id: i64) -> i64 {
        STANDARD_ID_NUM + id
    }
}
