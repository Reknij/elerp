use anyhow::bail;
use futures::TryStreamExt;
use sqlx::{Row, SqliteConnection};

use crate::public_system::{
    model::{Pagination, WebSocketFlags},
    PublicSystem,
};

use self::model::{GetPersonsQuery, Person};
use super::{ActionType, Result};
pub mod model;

#[derive(Debug, Clone)]
pub struct PersonModule {
    ps: PublicSystem,
}

impl PersonModule {
    pub async fn new(ps: PublicSystem) -> Self {
        let conn = ps.get_conn();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS persons(
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                address TEXT NOT NULL,
                area_id INT NOT NULL,
                person_in_charge_id INT NOT NULL,
                contact TEXT NOT NULL,
                email TEXT NOT NULL,
                color TEXT NULL,
                text_color TEXT NULL
            )",
        )
        .execute(conn)
        .await
        .unwrap();
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS person_names
    ON persons(name);
    CREATE INDEX IF NOT EXISTS person_area_ids
    ON persons(area_id);
    CREATE INDEX IF NOT EXISTS person_person_in_charge_ids
    ON persons(person_in_charge_id);",
        )
        .execute(conn)
        .await
        .unwrap();

        Self { ps }
    }

    pub async fn is_exists(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        self.ps.is_exists_in_table("persons", "id", id, tx).await
    }

    pub async fn is_limit_reached(&self, tx: &mut SqliteConnection) -> Result<bool> {
        let count: i64 = sqlx::query("SELECT COUNT(*) as count FROM persons;")
            .fetch_one(&mut *tx)
            .await?
            .get("count");
        Ok(count >= self.ps.get_config().limit.persons)
    }

    pub async fn is_exists_name(
        &self,
        name: &str,
        prev: Option<i64>,
        tx: &mut SqliteConnection,
    ) -> Result<bool> {
        self.ps
            .row_is_duplicate_col_in_table(name, prev, "persons", "name", tx)
            .await
    }

    pub async fn add(&self, mut person: Person, tx: &mut SqliteConnection) -> Result<Person> {
        let r = sqlx::query(
            "INSERT INTO persons (name, description, address, area_id, person_in_charge_id, contact, email, color, text_color) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&person.name)
        .bind(&person.description)
        .bind(&person.address)
        .bind(person.area_id)
        .bind(person.person_in_charge_id)
        .bind(&person.contact)
        .bind(&person.email)
        .bind(&person.color)
        .bind(&person.text_color)
        .execute(&mut *tx)
        .await?;
        if r.rows_affected() != 1 {
            bail!("Can't add person!");
        }

        person.id = self
            .ps
            .try_set_standard_id(r.last_insert_rowid(), "persons", tx)
            .await?;
        self.ps.notice(WebSocketFlags::AddPerson(person.id)).await?;
        Ok(person)
    }

    pub async fn remove(&self, id: i64, notice: bool, tx: &mut SqliteConnection) -> Result<bool> {
        let r = self.ps.remove_row_from_table(id, "persons", tx).await?;
        if notice {
            self.ps.notice(WebSocketFlags::RemovePerson(id)).await?;
        }
        Ok(r)
    }

    pub async fn get(
        &self,
        id: i64,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Option<Person>> {
        self.ps
            .get_row_from_table("persons", "id", id, tx)
            .await
            .map(|opt: Option<Person>| {
                opt.map(|mut v| {
                    match action {
                        ActionType::General(_) => {
                            v.address = String::new();
                            v.contact = String::new();
                            v.email = String::new();
                        }
                        ActionType::Admin | ActionType::System | ActionType::GeneralAllowed(_) => {
                            ()
                        }
                    }
                    v
                })
            })
    }

    pub async fn get_multiple(
        &self,
        pagination: &Pagination,
        query: &GetPersonsQuery,
        action: ActionType,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<Person>> {
        let qw = query.get_where_condition();
        let ob = query.get_order_condition();
        let rows = sqlx::query(&format!(
            "
        SELECT
        persons.id,
        persons.name,
        persons.description,
        persons.person_in_charge_id,
        persons.area_id,
        persons.address,
        persons.contact,
        persons.email,
        persons.color,
        persons.text_color,
        persons2.name AS person_in_charge_name,
        areas.id AS area_name
        FROM persons
        LEFT JOIN persons AS persons2 ON persons.person_in_charge_id=persons2.id
        INNER JOIN areas ON persons.area_id=areas.id
        {qw} {ob} LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        self.ps.rows_to_objects(rows).map(|mut list: Vec<Person>| {
            for v in list.iter_mut() {
                match action {
                    ActionType::General(_) => {
                        v.address = String::new();
                        v.contact = String::new();
                        v.email = String::new();
                    }
                    ActionType::Admin | ActionType::System | ActionType::GeneralAllowed(_) => (),
                }
            }
            list
        })
    }

    pub async fn get_multiple_ids(
        &self,
        pagination: &Pagination,
        query: &GetPersonsQuery,
        tx: &mut SqliteConnection,
    ) -> Result<Vec<i64>> {
        let qw = query.get_where_condition();
        let rows = sqlx::query(&format!(
            "SELECT
            id
            FROM persons
            {qw}  LIMIT ? OFFSET ?"
        ))
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(&mut *tx)
        .await?;
        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }

    pub async fn get_count(
        &self,
        query: &GetPersonsQuery,
        tx: &mut SqliteConnection,
    ) -> Result<i64> {
        let qw = query.get_where_condition();
        let row = sqlx::query(&format!("SELECT count(*) as count FROM persons {qw}"))
            .fetch_one(&mut *tx)
            .await?;
        Ok(row.get("count"))
    }

    pub async fn update(
        &self,
        id: i64,
        mut v: Person,
        tx: &mut SqliteConnection,
    ) -> Result<Option<Person>> {
        let r = sqlx::query("UPDATE persons SET name=?, description=?, address=?, area_id=?, person_in_charge_id=?, contact=?, email=?, color=?, text_color=? WHERE id=?")
        .bind(&v.name).bind(&v.description).bind(&v.address).bind(v.area_id).bind(v.person_in_charge_id).bind(&v.contact).bind(&v.email).bind(&v.color).bind(&v.text_color).bind(id).execute(&mut *tx).await?;
        Ok(if r.rows_affected() == 1 {
            v.id = id;
            self.ps.notice(WebSocketFlags::UpdatePerson(id)).await?;
            Some(v)
        } else {
            None
        })
    }

    pub async fn is_depend_by_another(&self, id: i64, tx: &mut SqliteConnection) -> Result<bool> {
        Ok(
            sqlx::query("SELECT id FROM persons WHERE person_in_charge_id=?")
                .bind(id)
                .fetch(&mut *tx)
                .try_next()
                .await?
                .is_some()
                || sqlx::query("SELECT id FROM warehouses WHERE person_in_charge_id=?")
                    .bind(id)
                    .fetch(&mut *tx)
                    .try_next()
                    .await?
                    .is_some()
                || sqlx::query("SELECT id FROM order_payments WHERE person_in_charge_id=?")
                    .bind(id)
                    .fetch(&mut *tx)
                    .try_next()
                    .await?
                    .is_some()
                || sqlx::query("SELECT id FROM orders WHERE person_related_id=?")
                    .bind(id)
                    .fetch(&mut *tx)
                    .try_next()
                    .await?
                    .is_some(),
        )
    }
}
