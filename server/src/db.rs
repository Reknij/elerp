use std::time::Duration;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use tokio::fs;

use crate::config::AppConfig;


pub async fn init_db(config: &AppConfig, one_time: bool) -> Result<Pool<Sqlite>> {
    let path = config.data_path.join("elerp.db");
    if one_time && path.is_file() {
        fs::remove_file(&path).await.unwrap();
    }
    let options = SqliteConnectOptions::new()
        .filename(path)
        .busy_timeout(Duration::from_millis(6000))
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(12)
        .connect_with(options)
        .await?;
    Ok(pool)
}
