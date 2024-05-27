use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};

use crate::sql::{get_search_where_condition, get_sort_col_str, get_sorter_str};

#[derive(Debug, Deserialize, Serialize, ToSchema, FromRow)]
pub struct Person {
    /// Id will generated by the system.
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub area_id: i64,
    #[serde(default)]
    pub person_in_charge_id: i64,
    #[serde(default)]
    pub contact: String,
    #[serde(default)]
    pub email: String,
    pub color: Option<String>,
    pub text_color: Option<String>,
}

#[derive(Debug, Default, Deserialize, ToSchema, IntoParams)]
pub struct GetPersonsQuery {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub address: Option<String>,
    pub area_id: Option<i64>,
    pub person_in_charge_id: Option<i64>,
    pub contact: Option<String>,
    pub email: Option<String>,
    pub sorters: Option<Vec<String>>,
}

impl GetPersonsQuery {
    pub fn get_where_condition(&self) -> String {
        let mut conditions = Vec::with_capacity(4);
        if let Some(v) = &self.id {
            conditions.push(format!("persons.id = {v}").into());
        }
        if let Some(v) = &self.name {
            conditions.push(get_search_where_condition("persons.name", v));
        }
        if let Some(v) = &self.address {
            let v = v.trim();
            conditions.push(format!("persons.address LIKE '%{v}%'").into());
        }
        if let Some(v) = &self.area_id {
            conditions.push(format!("persons.area_id={v}").into());
        }
        if let Some(v) = &self.person_in_charge_id {
            conditions.push(format!("persons.person_in_charge_id={v}").into());
        }
        if let Some(v) = &self.contact {
            let v = v.trim();
            conditions.push(format!("persons.contact LIKE '%{v}%'").into());
        }
        if let Some(v) = &self.email {
            let v = v.trim();
            conditions.push(format!("persons.email LIKE '%{v}%'").into());
        }
        if !conditions.is_empty() {
            let c = conditions.join(" AND ");
            format!("WHERE {c}").into()
        } else {
            "".into()
        }
    }

    pub fn get_order_condition(&self) -> String {
        let mut conditions = vec![];
        if self.name.is_some() {
            conditions.push("length(persons.name) ASC".into());
        }
        if let Some(sorters) = self.sorters.as_ref() {
            for sorter in sorters {
                let mut col = get_sort_col_str(sorter);
                let sort = get_sorter_str(sorter);
                if col == "person_in_charge_id" {
                    col = format!("person_in_charge_name {sort}").into();
                } else if col == "area_id" {
                    col = format!("area_name {sort}").into();
                } else {
                    col = format!("persons.{col} {sort}").into();
                }

                conditions.push(col);
            }
        }

        if !conditions.is_empty() {
            let c = conditions.join(", ");
            format!("ORDER BY {c}").into()
        } else {
            "".into()
        }
    }
}