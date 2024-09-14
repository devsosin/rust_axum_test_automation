// Production version
#[cfg(not(test))]
pub fn get_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

// Test version
#[cfg(test)]
pub fn get_uuid() -> uuid::Uuid {
    uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
}
