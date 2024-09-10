use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, sqlx::FromRow, Serialize, Clone, PartialEq)]
pub(crate) struct Record {
    id: Option<i64>,
    book_id: i32,
    sub_category_id: i32,
    amount: i32,
    memo: Option<String>,
    target_dt: NaiveDateTime,
    created_at: Option<NaiveDateTime>,
    updated_at: Option<NaiveDateTime>,
    asset_id: Option<i32>,
}

impl Record {
    pub(crate) fn new(
        book_id: i32,
        sub_category_id: i32,
        amount: i32,
        target_dt: NaiveDateTime,
        asset_id: Option<i32>,
    ) -> Self {
        Self {
            id: None,
            book_id,
            sub_category_id,
            amount,
            memo: None,
            target_dt,
            created_at: None,
            updated_at: None,
            asset_id,
        }
    }

    pub(crate) fn id(mut self, id: Option<i64>) -> Self {
        self.id = id;
        self
    }

    pub(crate) fn memo(mut self, memo: Option<String>) -> Self {
        self.memo = memo;
        self
    }

    pub(crate) fn updated_at(mut self, updated_at: Option<NaiveDateTime>) -> Self {
        self.updated_at = updated_at;
        self
    }

    pub(crate) fn build(&self) -> Self {
        Self {
            id: self.id,
            book_id: self.book_id,
            sub_category_id: self.sub_category_id,
            amount: self.amount,
            memo: self.memo.clone(),
            target_dt: self.target_dt,
            created_at: self.created_at,
            updated_at: self.updated_at,
            asset_id: self.asset_id,
        }
    }

    pub(crate) fn get_id(&self) -> i64 {
        if let Some(id) = self.id {
            id
        } else {
            -1
        }
    }
    pub(crate) fn get_book_id(&self) -> i32 {
        self.book_id
    }
    pub(crate) fn get_sub_category_id(&self) -> i32 {
        self.sub_category_id
    }
    pub(crate) fn get_amount(&self) -> i32 {
        self.amount
    }
    pub(crate) fn get_memo(&self) -> &Option<String> {
        &self.memo
    }
    pub(crate) fn get_target_dt(&self) -> NaiveDateTime {
        self.target_dt
    }
    pub(crate) fn get_created_at(&self) -> Option<NaiveDateTime> {
        self.created_at
    }

    pub(crate) fn get_updated_at(&self) -> &Option<NaiveDateTime> {
        &self.updated_at
    }
    pub(crate) fn get_asset_id(&self) -> &Option<i32> {
        &self.asset_id
    }
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum FieldUpdate<T> {
    Set(T),
    SetNone,
    NoChange,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) struct UpdateRecord {
    sub_category_id: FieldUpdate<i32>,
    amount: FieldUpdate<i32>,
    memo: FieldUpdate<String>,
    target_dt: FieldUpdate<NaiveDateTime>,
    asset_id: FieldUpdate<i32>,
}

impl UpdateRecord {
    pub(crate) fn new(
        sub_category_id: FieldUpdate<i32>,
        amount: FieldUpdate<i32>,
        memo: FieldUpdate<String>,
        target_dt: FieldUpdate<NaiveDateTime>,
        asset_id: FieldUpdate<i32>,
    ) -> Self {
        Self {
            sub_category_id,
            amount,
            memo,
            target_dt,
            asset_id,
        }
    }

    pub(crate) fn get_sub_category_id(&self) -> &FieldUpdate<i32> {
        &self.sub_category_id
    }
    pub(crate) fn get_amount(&self) -> &FieldUpdate<i32> {
        &self.amount
    }
    pub(crate) fn get_memo(&self) -> &FieldUpdate<String> {
        &self.memo
    }
    pub(crate) fn get_target_dt(&self) -> &FieldUpdate<NaiveDateTime> {
        &self.target_dt
    }
    pub(crate) fn get_asset_id(&self) -> &FieldUpdate<i32> {
        &self.asset_id
    }
}
