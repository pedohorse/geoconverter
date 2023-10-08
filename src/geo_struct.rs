use std::collections::HashMap;

#[derive(Debug)]
pub enum ReaderElement {
    None,
    Bool(bool),
    Text(String),
    Int(i64),
    Float(f64),
    Array(Vec<ReaderElement>),
    KeyValueObject(HashMap<String, ReaderElement>)
}