use std::collections::HashMap;

#[derive(Debug)]
pub enum ReaderElement {
    None,
    Bool(bool),
    Text(String),
    Int(i64),
    Float(f64),
    Array(Vec<ReaderElement>),
    KeyValueObject(HashMap<String, ReaderElement>),
    UniformArray(UniformArrayType)
}

#[derive(Debug)]
pub enum UniformArrayType {
    UniformArrayTu8(Vec<u8>),
    UniformArrayTu16(Vec<u16>),
    UniformArrayTi8(Vec<i8>),
    UniformArrayTi16(Vec<i16>),
    UniformArrayTi32(Vec<i32>),
    UniformArrayTi64(Vec<i64>),
    UniformArrayTf16(Vec<f32>),
    UniformArrayTf32(Vec<f32>),
    UniformArrayTf64(Vec<f64>),
    UniformArrayTbool(Vec<bool>)
}
