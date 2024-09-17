#[derive(Clone, PartialEq, Debug)]
pub enum FieldUpdate<T> {
    Set(T),
    SetNone,
    NoChange,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UpdateResult {
    is_exist: bool,
    is_authorized: bool,
    is_duplicated: bool,
    update_count: i64,
}

impl UpdateResult {
    pub fn get_exist(&self) -> bool {
        self.is_exist
    }
    pub fn get_authorized(&self) -> bool {
        self.is_authorized
    }
    pub fn get_duplicated(&self) -> bool {
        self.is_duplicated
    }
    pub fn get_count(&self) -> i64 {
        self.update_count
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DeleteResult {
    is_exist: bool,
    is_authorized: bool,
    delete_count: i64,
}

impl DeleteResult {
    pub fn get_exist(&self) -> bool {
        self.is_exist
    }
    pub fn get_authorized(&self) -> bool {
        self.is_authorized
    }
    pub fn get_count(&self) -> i64 {
        self.delete_count
    }
}
