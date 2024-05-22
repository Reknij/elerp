use std::{
    collections::hash_map::DefaultHasher,
    hash::Hasher,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use ahash::{HashMap, HashMapExt};
use elerp_common::model::WebSocketFlags;
use futures::TryStreamExt;

use sqlx::{Pool, Row, Sqlite, SqliteConnection, Transaction};

use anyhow::Result;

use std::hash::Hash;
use tokio::{
    fs,
    sync::{broadcast, RwLock},
    task::JoinHandle,
};
use tracing::{error, info};

use elerp_common::config::AppConfig;

pub mod db;

#[derive(Debug, Clone)]
pub struct PublicSystem {
    pool: Pool<Sqlite>,
    config: AppConfig,
    backup_future: Option<Arc<JoinHandle<()>>>,
    notice_tx: broadcast::Sender<WebSocketFlags>,
    count_state: Arc<RwLock<HashMap<String, i64>>>,
}

pub struct SqliteSafeTransaction<'a> {
    tx: Transaction<'a, Sqlite>,
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
    pub fn new(tx: Transaction<'a, Sqlite>) -> Self {
        Self { tx }
    }
    pub async fn commit(self) -> Result<()> {
        Ok(self.tx.commit().await?)
    }
}

impl PublicSystem {
    pub async fn update(config: AppConfig) -> bool {
        let pool = db::init_db(&config, false).await.expect("Init db failed!");
        db::update(pool).await
    }
    pub async fn new(config: AppConfig) -> Self {
        let pool = db::init_db(&config, false).await.expect("Init db failed!");
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS backup_records(
            filename TEXT NOT NULL,
            date INT NOT NULL  
            );",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS public_table(
            id INTEGER PRIMARY KEY,
            reserved INT NOT NULL DEFAULT 1
            );
            INSERT OR IGNORE INTO public_table (id, reserved) VALUES (1, 1)",
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
            if entry.file_type().await?.is_file() && entry.file_name().to_str().unwrap().starts_with("backup-") {
                fs::remove_file(entry.path()).await?;
            }
        }
        self.backup_database().await?;
        Ok(())
    }

    pub async fn get_backup_count(&self, tx: &mut SqliteConnection) -> Result<i64> {
        let row = sqlx::query(&format!("SELECT count(*) AS count FROM backup_records")).fetch_one(&mut *tx).await?;
        Ok(row.get("count"))
    }

    async fn process_limit_backup(&self) -> Result<()> {
        let mut tx = self.begin_tx(true).await?;
        let count = self.get_backup_count(tx.as_mut()).await?;
        if count > 8 {
            let r = sqlx::query("SELECT filename FROM backup_records ORDER BY date LIMIT 1").fetch(tx.as_mut()).try_next().await?;
            if let Some(row) = r {
                let filename: String = row.get("filename");
                let target = self.config.data_path.join(&filename);
                if target.is_file() {
                    fs::remove_file(target).await?;
                    sqlx::query("DELETE FROM backup_records WHERE filename=?").bind(&filename).execute(tx.as_mut()).await?;
                }
            }
        }
        Ok(())
    }

    async fn backup_database(&self) -> Result<bool> {
        self.process_limit_backup().await?;
        let now = self.get_timestamp_seconds() as i64;
        let filename = format!("backup-{now}.db");
        let r = sqlx::query("INSERT INTO backup_records VALUES (?, ?)").bind(&filename).bind(now).execute(&self.pool).await?;

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
        let mut tx = self.pool.begin().await?;
        if write {
            sqlx::query("UPDATE public_table SET reserved=1 WHERE id=1").execute(tx.as_mut()).await?;
        }
        Ok(SqliteSafeTransaction::new(tx))
    }

    pub fn get_timestamp_seconds(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    pub async fn exists_table(&self, table_name: &str, tx: &mut SqliteConnection) -> bool {
        // check if a table exists
        let query = r#"
            SELECT name FROM sqlite_master WHERE type='table' AND name=?;
        "#;

        match sqlx::query(query).bind(table_name).fetch_optional(&mut *tx).await.unwrap() {
            Some(_row) => true,
            None => false,
        }
    }

    pub fn get_current_month_timestamp(&self) -> (i64, i64) {
        use chrono::*;
        let naive_time_max = NaiveTime::from_hms_milli_opt(23, 59, 59, 999).unwrap();
        let local: DateTime<Local> = Local::now();
        let start = NaiveDate::from_ymd_opt(local.year(), local.month(), 1).unwrap().and_time(NaiveTime::MIN);
        let end = NaiveDate::from_ymd_opt(local.year(), local.month(), 31)
            .unwrap_or(
                NaiveDate::from_ymd_opt(local.year(), local.month(), 30)
                    .unwrap_or(NaiveDate::from_ymd_opt(local.year(), local.month(), 29).unwrap_or(NaiveDate::from_ymd_opt(local.year(), local.month(), 28).unwrap())),
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
