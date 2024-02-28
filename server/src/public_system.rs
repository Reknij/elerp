use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::Hasher,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ahash::{HashMap, HashMapExt};
use futures::TryStreamExt;

use sqlx::{
    sqlite::SqliteRow, Encode, FromRow, Pool, Row, Sqlite, SqliteConnection, Transaction, Type,
};

use anyhow::Result;

use std::hash::Hash;
use tokio::{
    fs,
    sync::{broadcast, Mutex, RwLock},
    task::JoinHandle,
};
use tracing::{error, info};

use crate::config::AppConfig;

use self::model::web_socket_flags::WebSocketFlags;

pub mod model;

#[derive(Debug, Clone)]
pub struct PublicSystem {
    pool: Pool<Sqlite>,
    config: AppConfig,
    backup_future: Option<Arc<JoinHandle<()>>>,
    notice_tx: broadcast::Sender<WebSocketFlags>,
    sqlite_tx_write: Arc<Mutex<()>>,
    count_state: Arc<RwLock<HashMap<String, i64>>>,
}

pub struct SqliteSafeTransaction<'a> {
    tx: Transaction<'a, Sqlite>,
    _ctx: Option<tokio::sync::MutexGuard<'a, ()>>,
}

impl<'a> Deref for SqliteSafeTransaction<'a> {
    type Target = Transaction<'a, Sqlite>;
    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl<'a> DerefMut for SqliteSafeTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tx
    }
}

impl<'a> SqliteSafeTransaction<'a> {
    pub async fn commit(self) -> Result<()> {
        Ok(self.tx.commit().await?)
    }
}

const STANDARD_ID_NUM: i64 = 10000;

impl PublicSystem {
    pub async fn new(pool: Pool<Sqlite>, config: AppConfig) -> Self {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS backup_records(
            filename TEXT NOT NULL,
            date INT NOT NULL  
        );",
        )
        .execute(&pool)
        .await
        .unwrap();

        let (tx, mut rx) = broadcast::channel(100);
        let mut s = Self {
            pool,
            config,
            backup_future: None,
            notice_tx: tx,
            sqlite_tx_write: Arc::new(Mutex::new(())),
            count_state: Arc::new(RwLock::new(HashMap::new())),
        };

        let s2 = s.clone();
        s.backup_future = Some(Arc::new(tokio::spawn(async move {
            let mut backup: Option<JoinHandle<()>> = None;
            while rx.recv().await.is_ok() {
                let s2 = s2.clone();
                if let Some(backup) = backup {
                    backup.abort();
                }
                backup = Some(tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(15 * 60)).await;
                    info!("Backup the database...");
                    if s2.backup_database().await.is_err() {
                        error!("Backup the database occurs error!");
                    }
                }));
            }
        })));
        s
    }

    pub async fn get_count(&self, key: &str) -> Option<i64> {
        let r = self.count_state.read().await;
        r.get(key).copied()
    }

    pub async fn insert_count(&self, key: String, count: i64) {
        let mut r = self.count_state.write().await;
        r.insert(key, count);
    }

    pub async fn try_update_count(&self, query_property: &str, f: impl Fn(&mut i64)) {
        let mut r = self.count_state.write().await;
        for (key, value) in r.iter_mut() {
            if key.contains(query_property) {
                f(value);
            }
        }
    }

    pub async fn try_set_standard_id(
        &self,
        creation_id: i64,
        table_name: &str,
        tx: &mut SqliteConnection,
    ) -> Result<i64> {
        if creation_id == 1 {
            let nid = STANDARD_ID_NUM + 1;
            sqlx::query(&Cow::Owned(format!(
                "UPDATE {table_name} SET id = {nid} WHERE id = {creation_id}"
            )))
            .execute(&mut *tx)
            .await?;
            Ok(nid)
        } else {
            Ok(creation_id as _)
        }
    }

    pub fn get_standard_id(&self, id: i64) -> i64 {
        STANDARD_ID_NUM + id
    }

    pub fn get_data_path(&self) -> &PathBuf {
        &self.config.data_path
    }

    pub async fn notice(&self, flag: WebSocketFlags) -> Result<()> {
        self.notice_tx.send(flag)?;
        Ok(())
    }

    pub async fn notication_subscribe(&self) -> broadcast::Receiver<WebSocketFlags> {
        self.notice_tx.subscribe()
    }

    pub async fn clear_cache(&self) -> Result<()> {
        let mut entrys = fs::read_dir(&self.config.data_path).await?;
        while let Some(entry) = entrys.next_entry().await? {
            if entry.file_type().await?.is_file()
                && entry.file_name().to_str().unwrap().starts_with("backup-")
            {
                fs::remove_file(entry.path()).await?;
            }
        }
        self.backup_database().await?;
        Ok(())
    }

    pub async fn get_backup_count(&self, tx: &mut SqliteConnection) -> Result<i64> {
        let row = sqlx::query(&format!("SELECT count(*) AS count FROM backup_records"))
            .fetch_one(&mut *tx)
            .await?;
        Ok(row.get("count"))
    }

    async fn process_limit_backup(&self) -> Result<()> {
        let mut tx = self.begin_tx(true).await?;
        let count = self.get_backup_count(tx.as_mut()).await?;
        if count > 8 {
            let r = sqlx::query("SELECT filename FROM backup_records ORDER BY date LIMIT 1")
                .fetch(tx.as_mut())
                .try_next()
                .await?;
            if let Some(row) = r {
                let filename: String = row.get("filename");
                let target = self.config.data_path.join(&filename);
                if target.is_file() {
                    fs::remove_file(target).await?;
                    sqlx::query("DELETE FROM backup_records WHERE filename=?")
                        .bind(&filename)
                        .execute(tx.as_mut())
                        .await?;
                }
            }
        }
        Ok(())
    }

    async fn backup_database(&self) -> Result<bool> {
        self.process_limit_backup().await?;
        let now = self.get_timestamp_seconds() as i64;
        let filename = format!("backup-{now}.db");
        let r = sqlx::query("INSERT INTO backup_records VALUES (?, ?)")
            .bind(&filename)
            .bind(now)
            .execute(&self.pool)
            .await?;

        if r.rows_affected() == 1 {
            let original = self.config.data_path.join("elerp.db");
            let target = self.config.data_path.join(&filename);
            fs::copy(original, target).await.unwrap();
        }
        Ok(r.rows_affected() == 1)
    }

    pub fn get_conn(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn begin_tx(&self, write: bool) -> Result<SqliteSafeTransaction<'_>> {
        let tx = SqliteSafeTransaction {
            tx: self.pool.begin().await?,
            _ctx: if write {
                Some(self.sqlite_tx_write.lock().await)
            } else {
                None
            },
        };
        Ok(tx)
    }

    pub async fn row_is_duplicate_col_in_table(
        &self,
        col: &str,
        prev: Option<i64>,
        table_name: &str,
        col_name: &str,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        let q1 = format!("SELECT id FROM {table_name} WHERE {col_name}=? AND id<>? LIMIT 1");
        let q2 = format!("SELECT id FROM {table_name} WHERE {col_name}=? LIMIT 1");
        let mut r = if let Some(prev) = prev {
            sqlx::query(&q1).bind(col).bind(prev).fetch(&mut *tx)
        } else {
            sqlx::query(&q2).bind(col).fetch(&mut *tx)
        };
        Ok(r.try_next().await?.is_some())
    }

    pub async fn get_row_from_table<T, V>(
        &self,
        table_name: &str,
        col_name: &str,
        col_value: V,
        tx: &mut SqliteConnection,
    ) -> Result<Option<T>>
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

    pub async fn is_exists_in_table<V>(
        &self,
        table_name: &str,
        col_name: &str,
        col_value: V,
        tx: &mut SqliteConnection,
    ) -> Result<bool>
    where
        for<'q> V: 'q + Send + Encode<'q, Sqlite> + Type<Sqlite>,
    {
        let q = format!("SELECT {col_name} FROM {table_name} WHERE {col_name} = ?");
        let mut r = sqlx::query(&q).bind(col_value).fetch(&mut *tx);
        Ok(r.try_next().await?.is_some())
    }

    pub fn rows_to_objects<'a, T>(&self, rows: Vec<SqliteRow>) -> Result<Vec<T>>
    where
        for<'r> T: FromRow<'r, SqliteRow> + Unpin + Send + 'a,
    {
        let mut arr = Vec::with_capacity(rows.len());
        for row in rows {
            arr.push(T::from_row(&row)?)
        }

        Ok(arr)
    }

    pub async fn remove_row_from_table(
        &self,
        row_id: i64,
        table_name: &str,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        let q = format!("DELETE FROM {table_name} WHERE id = ?");
        let r = sqlx::query(&q).bind(row_id).execute(&mut *tx).await?;
        Ok(r.rows_affected() == 1)
    }

    pub fn get_timestamp_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub async fn exists_table(&self, table_name: &str, tx: &mut SqliteConnection) -> bool {
        // check if a table exists
        let query = r#"
            SELECT name FROM sqlite_master WHERE type='table' AND name=?;
        "#;

        match sqlx::query(query)
            .bind(table_name)
            .fetch_optional(&mut *tx)
            .await
            .unwrap()
        {
            Some(_row) => true,
            None => false,
        }
    }

    pub fn get_current_month_timestamp(&self) -> (i64, i64) {
        use chrono::*;
        let naive_time_max = NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap();
        let local: DateTime<Local> = Local::now();
        let start = NaiveDate::from_ymd_opt(local.year(), local.month(), 1)
            .unwrap()
            .and_time(NaiveTime::MIN);
        let end = NaiveDate::from_ymd_opt(local.year(), local.month(), 31)
            .unwrap_or(
                NaiveDate::from_ymd_opt(local.year(), local.month(), 30).unwrap_or(
                    NaiveDate::from_ymd_opt(local.year(), local.month(), 29).unwrap_or(
                        NaiveDate::from_ymd_opt(local.year(), local.month(), 28).unwrap(),
                    ),
                ),
            )
            .and_time(naive_time_max);
        let sts = Local.from_local_datetime(&start).unwrap();
        let ets = Local.from_local_datetime(&end).unwrap();
        (sts.timestamp(), ets.timestamp())
    }

    pub async fn calculate_hash<T>(&self, t: &T, append_random: bool) -> u64
    where
        T: Hash,
    {
        let mut s = DefaultHasher::new();
        if append_random {
            let random_num: u64 = rand::random();
            s.write_u64(random_num);
        }
        t.hash(&mut s);
        s.finish()
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
}
