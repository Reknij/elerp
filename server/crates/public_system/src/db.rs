use std::time::Duration;

use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Row, Sqlite,
};
use tokio::fs;

use elerp_common::config::AppConfig;

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

pub async fn update(pool: Pool<Sqlite>) -> bool {
    let mut tx = pool.begin().await.unwrap();
    let mut updated = 0;
    if sqlx::query("SELECT 1 FROM sqlite_schema WHERE type='table' AND name='configures'")
        .fetch_optional(tx.as_mut())
        .await
        .unwrap()
        .is_some()
    {
        let q = sqlx::query("
        SELECT
    IFNULL((SELECT 1 FROM pragma_table_info('configures') WHERE name='d_order_type'), 0) AS d_order_type,
    IFNULL((SELECT 1 FROM pragma_table_info('configures') WHERE name='d_order_category_id'), 0) AS d_order_category_id,
    IFNULL((SELECT 1 FROM pragma_table_info('configures') WHERE name='d_warehouse_id'), 0) AS d_warehouse_id,
    IFNULL((SELECT 1 FROM pragma_table_info('configures') WHERE name='d_person_related_id'), 0) AS d_person_related_id,
    IFNULL((SELECT 1 FROM pragma_table_info('configures') WHERE name='d_order_currency'), 0) AS d_order_currency;
        ").fetch_one(tx.as_mut()).await.unwrap();

        if !q.get::<bool, _>("d_order_type") {
            sqlx::query(
                "ALTER TABLE configures ADD d_order_type TEXT NOT NULL DEFAULT 'StockOut';",
            )
            .execute(tx.as_mut())
            .await
            .unwrap();
            updated += 1;
        }

        if !q.get::<bool, _>("d_order_category_id") {
            sqlx::query("ALTER TABLE configures ADD d_order_category_id INT NOT NULL DEFAULT 0;")
                .execute(tx.as_mut())
                .await
                .unwrap();
            updated += 1;
        }

        if !q.get::<bool, _>("d_warehouse_id") {
            sqlx::query("ALTER TABLE configures ADD d_warehouse_id INT NOT NULL DEFAULT 0;")
                .execute(tx.as_mut())
                .await
                .unwrap();
            updated += 1;
        }

        if !q.get::<bool, _>("d_person_related_id") {
            sqlx::query("ALTER TABLE configures ADD d_person_related_id INT NOT NULL DEFAULT 0;")
                .execute(tx.as_mut())
                .await
                .unwrap();
            updated += 1;
        }

        if !q.get::<bool, _>("d_order_currency") {
            sqlx::query("ALTER TABLE configures ADD d_order_currency TEXT NOT NULL DEFAULT 'USD';")
                .execute(tx.as_mut())
                .await
                .unwrap();
            updated += 1;
        }
    }
    tx.commit().await.unwrap();
    updated > 0
}
