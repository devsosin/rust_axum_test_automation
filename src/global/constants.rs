#[derive(Clone, PartialEq, Debug)]
pub enum FieldUpdate<T> {
    Set(T),
    SetNone,
    NoChange,
}

#[derive(Debug, sqlx::FromRow)]
pub struct UpdateResult {
    exists_check: bool,
    authorized_check: bool,
    duplicated_check: bool,
    update_count: i64,
}

impl UpdateResult {
    pub fn get_exist(&self) -> bool {
        self.exists_check
    }
    pub fn get_authorized(&self) -> bool {
        self.authorized_check
    }
    pub fn get_duplicated(&self) -> bool {
        self.duplicated_check
    }
    pub fn get_count(&self) -> i64 {
        self.update_count
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DeleteResult {
    exists_check: bool,
    authorized_check: bool,
    delete_count: i64,
}

impl DeleteResult {
    pub fn get_exist(&self) -> bool {
        self.exists_check
    }
    pub fn get_authorized(&self) -> bool {
        self.authorized_check
    }
    pub fn get_count(&self) -> i64 {
        self.delete_count
    }
}
