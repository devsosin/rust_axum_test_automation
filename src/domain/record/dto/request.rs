use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use url::form_urlencoded::Serializer;

use crate::{
    domain::record::entity::{Record, Search, UpdateRecord},
    global::constants::FieldUpdate,
};

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct NewRecord {
    book_id: i32,
    sub_category_id: i32,
    amount: i32,
    memo: Option<String>,
    target_dt: NaiveDateTime,
    asset_id: Option<i32>,
    connect_ids: Option<Vec<i32>>,
}

impl NewRecord {
    pub fn new(
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

    pub fn to_entity(&self) -> Record {
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

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct SearchParams {
    start_dt: NaiveDate,
    period: String, // M, D
    base_id: Option<i16>,
    sub_id: Option<i32>,
}

impl SearchParams {
    pub fn new(
        start_dt: NaiveDate,
        period: String, // M, D
        base_id: Option<i16>,
        sub_id: Option<i32>,
    ) -> Self {
        Self {
            start_dt,
            period,
            base_id,
            sub_id,
        }
    }

    pub fn get_period(&self) -> &str {
        &self.period
    }

    pub fn encode_param(&self) -> String {
        let mut binding = Serializer::new(String::new());
        binding
            .append_pair("start_dt", &self.start_dt.format("%Y-%m-%d").to_string())
            .append_pair("period", &self.period);

        if let Some(base_id) = self.base_id {
            binding.append_pair("base_id", &base_id.to_string());
        }

        if let Some(sub_id) = self.sub_id {
            binding.append_pair("sub_id", &sub_id.to_string());
        }

        binding.finish()
    }

    pub fn to_query(&self) -> Search {
        let end_dt = match self.period.to_lowercase().as_str() {
            "m" => self.start_dt.checked_add_months(chrono::Months::new(1)),
            "d" => self.start_dt.checked_add_days(chrono::Days::new(1)),
            _ => None,
        };

        Search::new(self.start_dt, end_dt.unwrap(), self.base_id, self.sub_id)
    }
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq)]
pub struct EditRecord {
    sub_category_id: Option<i32>,
    amount: Option<i32>,
    memo: Option<String>,
    target_dt: Option<NaiveDateTime>,
    asset_id: Option<i32>,
}

impl EditRecord {
    pub fn new(
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

    pub fn to_update(self) -> UpdateRecord {
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
