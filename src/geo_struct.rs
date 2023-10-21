use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ReaderElement {
    None,
    Bool(bool),
    Text(String),
    Int(i64),
    Float(f64),
    Array(Vec<ReaderElement>),
    KeyValueObject(HashMap<String, ReaderElement>),
    UniformArray(UniformArrayType),
}

#[derive(Debug, Clone)]
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
    UniformArrayTbool(Vec<bool>),
}

/// this guy is supposed to provide information how to locate a certain key
/// within ReaderElement structure
#[derive(Clone)]
pub struct ReaderElementPointer {
    path: Vec<ReaderElementPointerEntry>,
}

#[derive(Clone)]
enum ReaderElementPointerEntry {
    ArrayIndex(usize),
    MapKey(String),
}

impl ReaderElementPointer {
    pub fn new() -> ReaderElementPointer {
        ReaderElementPointer {
            path: Vec::new()
        }
    }

    pub fn add_array_index(&mut self, idx: usize) {
        self.path.push(ReaderElementPointerEntry::ArrayIndex(idx));
    }

    pub fn add_map_index(&mut self, key: String) {
        self.path.push(ReaderElementPointerEntry::MapKey(key));
    }

    pub fn locate_key_in<'a>(&self, elem: &'a ReaderElement) -> Option<&'a ReaderElement> {
        let mut curr = elem;
        for entry in self.path.iter() {
            match (entry, curr) {
                (ReaderElementPointerEntry::ArrayIndex(idx), ReaderElement::Array(next_array)) => {
                    curr = next_array.get(*idx)?;
                }
                (ReaderElementPointerEntry::MapKey(key), ReaderElement::KeyValueObject(next_map)) => {
                    curr = next_map.get(key)?;
                }
                _ => {
                    return None;
                }
            }
        };
        Some(curr)
    }

    pub fn locate_key_in_mut<'a>(&self, elem: &'a mut ReaderElement) -> Option<&'a mut ReaderElement> {
        let mut curr = elem;
        for entry in self.path.iter() {
            match (entry, curr) {
                (ReaderElementPointerEntry::ArrayIndex(idx), ReaderElement::Array(next_array)) => {
                    curr = next_array.get_mut(*idx)?;
                }
                (ReaderElementPointerEntry::MapKey(key), ReaderElement::KeyValueObject(next_map)) => {
                    curr = next_map.get_mut(key)?;
                }
                _ => {
                    return None;
                }
            }
        };
        Some(curr)
    }
}
