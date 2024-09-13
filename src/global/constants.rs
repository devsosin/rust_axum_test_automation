#[derive(Clone, PartialEq, Debug)]
pub enum FieldUpdate<T> {
    Set(T),
    SetNone,
    NoChange,
}
