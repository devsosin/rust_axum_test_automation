use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    domain::record::entity::{Record, UpdateRecord},
    global::constants::FieldUpdate,
};

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
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

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub(crate) struct EditRecord {
    sub_category_id: Option<i32>,
    amount: Option<i32>,
    memo: Option<String>,
    target_dt: Option<NaiveDateTime>,
    asset_id: Option<i32>,
}

impl EditRecord {
    pub(crate) fn new(
        sub_category_id: Option<i32>,
        amount: Option<i32>,
        memo: Option<String>,
        target_dt: Option<NaiveDateTime>,
        asset_id: Option<i32>,
    ) -> Self {
        Self {
            sub_category_id,
            amount,
            memo,
            target_dt,
            asset_id,
        }
    }

    pub(crate) fn to_update(self) -> UpdateRecord {
        let sub_category_id = match self.sub_category_id {
            Some(v) => FieldUpdate::Set(v),
            None => FieldUpdate::NoChange,
        };
        let amount = match self.amount {
            Some(v) => FieldUpdate::Set(v),
            None => FieldUpdate::NoChange,
        };
        let memo = match &self.memo {
            Some(v) if v == "NULL" => FieldUpdate::SetNone,
            Some(v) => FieldUpdate::Set(v.to_string()),
            None => FieldUpdate::NoChange,
        };
        let target_dt = match self.target_dt {
            Some(v) => FieldUpdate::Set(v),
            None => FieldUpdate::NoChange,
        };
        let asset_id = match self.asset_id {
            Some(v) => FieldUpdate::Set(v),
            None => FieldUpdate::NoChange,
        };
        UpdateRecord::new(sub_category_id, amount, memo, target_dt, asset_id)
    }
}
