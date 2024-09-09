use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain::record::entity::Record;

#[derive(Deserialize, Debug, sqlx::FromRow, Serialize, Clone, PartialEq)]
pub(crate) struct NewRecord {
    book_id: i32,
    sub_category_id: i32,
    amount: i32,
    memo: Option<String>,
    target_dt: NaiveDateTime,
    asset_id: Option<i32>,
    connect_ids: Option<Vec<i32>>,
}

impl NewRecord {
    pub(crate) fn new(
        book_id: i32,
        sub_category_id: i32,
        amount: i32,
        memo: Option<String>,
        target_dt: NaiveDateTime,
        asset_id: Option<i32>,
        connect_ids: Option<Vec<i32>>,
    ) -> Self {
        Self {
            book_id,
            sub_category_id,
            amount,
            memo,
            target_dt,
            asset_id,
            connect_ids,
        }
    }

    pub(crate) fn to_entity(&self) -> Record {
        Record::new(
            self.book_id,
            self.sub_category_id,
            self.amount,
            self.target_dt,
            self.asset_id,
        )
        .memo(self.memo.clone())
        .build()
    }

    pub fn get_connect_ids(&self) -> Option<Vec<i32>> {
        self.connect_ids.clone()
    }
}
