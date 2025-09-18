#![expect(clippy::option_if_let_else)]
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<i32>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TimePaginated<T> {
    pub items: Vec<T>,
    /// RFC 3339 timestamp for next page, if available
    pub next_cursor: Option<DateTime<Utc>>,
}

impl<T> Paginated<T> {
    pub const fn nothing() -> Self {
        Self {
            items: vec![],
            next_cursor: None,
        }
    }

    pub fn map_items<U>(self, f: impl Fn(T) -> U) -> Paginated<U> {
        Paginated {
            items: self.items.into_iter().map(f).collect(),
            next_cursor: self.next_cursor,
        }
    }
}

impl<T> TimePaginated<T> {
    pub const fn nothing() -> Self {
        Self {
            items: vec![],
            next_cursor: None,
        }
    }

    pub fn map_items<U>(self, f: impl Fn(T) -> U) -> TimePaginated<U> {
        TimePaginated {
            items: self.items.into_iter().map(f).collect(),
            next_cursor: self.next_cursor,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Cursor {
    pub at: u32,
    pub limit: u8,
}

#[derive(Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct TimeCursor {
    /// Optional timestamp cursor for pagination (RFC 3339 format)
    pub after: Option<DateTime<Utc>>,
    /// Number of items to return (1-100, default 20)
    #[serde(default = "default_limit")]
    pub limit: u8,
}

const fn default_limit() -> u8 {
    20
}

impl Default for TimeCursor {
    fn default() -> Self {
        Self {
            after: None,
            limit: default_limit(),
        }
    }
}
