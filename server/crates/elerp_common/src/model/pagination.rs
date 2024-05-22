use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct Pagination {
    /// index of pagination. It is using in sql query with `offset` = `pagination.index` * `pagination.limit`
    index: i64,
    /// limit of the list length.
    limit: i64,

    /// Manual set the offset without index.
    offset: Option<i64>,
}

impl Pagination {
    pub fn new(index: i64, limit: i64) -> Self {
        Self {
            index, limit , offset: None
        }
    }
    pub fn new_with_offset(offset: i64, limit: i64) -> Self {
        Self {
            index: 0,
            offset: Some(offset),
            limit,
        }
    }
    pub fn correct(mut self) -> Self {
        if self.index < 0 {
            self.index = 0;
        }
        if self.limit < 0 {
            self.limit = 0;
        }
        self
    }
    pub fn max() -> Self {
        Self {
            index: 0,
            limit: i64::MAX,
            offset: None,
        }
    }
    pub fn offset(&self) -> i64 {
        match self.offset {
            Some(v) => v,
            None => self.index * self.limit,
        }
    }
    pub fn limit(&self) -> i64 {
        self.limit
    }
    pub fn set_offset(&mut self, v: i64) -> &Self {
        self.offset = Some(v);
        self
    }
    pub fn next(&mut self) -> &Self {
        self.index += 1;
        self
    }
    pub fn contain(&mut self, count: i64) -> bool {
        self.offset() < count
    }
}
