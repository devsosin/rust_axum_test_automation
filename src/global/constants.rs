#[derive(Clone, PartialEq, Debug)]
pub(crate) enum FieldUpdate<T> {
    Set(T),
    SetNone,
    NoChange,
}
