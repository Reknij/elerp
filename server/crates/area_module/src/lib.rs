use anyhow::bail;
use anyhow::Result;
use elerp_common::sql;
use elerp_common::sql::get_row_from_table;
use elerp_common::sql::is_exists_in_table;
use elerp_common::sql::remove_row_from_table;
use elerp_common::sql::row_is_duplicate_col_in_table;
use elerp_common::sql::rows_to_objects;
use elerp_common::{
    area_module::model::area::{Area, GetAreasQuery},
    model::{Pagination, WebSocketFlags},
};
use futures::TryStreamExt;
use public_system::PublicSystem;
use sqlx::{Row, SqliteConnection};

#[derive(Debug, Clone)]
pub struct AreaModule {
    ps: PublicSystem,
}

impl AreaModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let conn = ps.get_conn();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS areas(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                color TEXT NULL,
                text_color TEXT NULL
            )",
        )
        .execute(conn)
        .await
        .unwrap();
        sqlx::query(
            "CREATE UNIQUE INDEX IF NOT EXISTS area_names
        ON areas(name)",
        )
        .execute(conn)
        .await
        .unwrap();

        Self { ps }
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM areas;").fetch_one(&mut *tx).await?.get("count");
        Ok(count >= self.ps.get_config().limit.areas)
    }

    pub async fn is_exists_name(&self, name: &str, prev: Option<i64>, tx: &mut SqliteConnection) -> Result<bool> {
        row_is_duplicate_col_in_table(name, prev, "areas", "name", tx).await
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        is_exists_in_table("areas", "id", id, tx).await
    }

    pub async fn add(&self, mut v: Area, tx: &mut SqliteConnection) -> Result<Area> {
        let r = sqlx::query("INSERT INTO areas (name, description, color, text_color) VALUES(?, ?, ?, ?)")
            .bind(&v.name)
            .bind(&v.description)
            .bind(&v.color)
            .bind(&v.text_color)
            .execute(&mut *tx)
            .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add area");
        }
        v.id = sql::try_set_standard_id(r.last_insert_rowid(), "areas", tx).await?;
        self.ps.notice(WebSocketFlags::AddArea(v.id)).await?;
        Ok(v)
    }

    pub async fn remove(&self, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        let r = remove_row_from_table(id, "areas", tx).await?;
        if notice {
            self.ps.notice(WebSocketFlags::RemoveArea(id)).await?;
        }
        Ok(r)
    }

    pub async fn get(&self, id: i64, tx: &mut SqliteConnection) -> Result<Option<Area>> {
        get_row_from_table("areas", "id", id, tx).await
    }

    pub async fn get_multiple(&self, pagination: &Pagination, query: &GetAreasQuery, tx: &mut SqliteConnection) -> Result<Vec<Area>> {
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let rows = sqlx::query(&format!(
            "
        SELECT
        areas.id,
        areas.name,
        areas.description,
        areas.color,
        areas.text_color
        FROM areas
        {qw} {ob} LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        rows_to_objects(rows)
    }

    pub async fn get_multiple_ids(&self, pagination: &Pagination, query: &GetAreasQuery, tx: &mut SqliteConnection) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            id
            FROM areas
            {qw}  LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }

    pub async fn get_count(&self, query: &GetAreasQuery, tx: &mut SqliteConnection) -> Result<i64> {
        let qw = query.get_where_condition();
        let row = sqlx::query(&format!("SELECT count(*) as count FROM areas {qw}")).fetch_one(&mut *tx).await?;
        Ok(row.get("count"))
    }

    pub async fn update(&self, id: i64, mut v: Area, tx: &mut SqliteConnection) -> Result<Option<Area>> {
        let r = sqlx::query("UPDATE areas SET name=?, description=?, color=?, text_color=? WHERE id=?")
            .bind(&v.name)
            .bind(&v.description)
            .bind(&v.color)
            .bind(&v.text_color)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        Ok(if r.rows_affected() == 1 {
            v.id = id;
            self.ps.notice(WebSocketFlags::UpdateArea(id)).await?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn is_depend_by_another(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(sqlx::query("SELECT id FROM persons WHERE area_id=?").bind(id).fetch(&mut *tx).try_next().await?.is_some()
            || sqlx::query("SELECT id FROM warehouses WHERE area_id=?").bind(id).fetch(&mut *tx).try_next().await?.is_some())
    }
}
