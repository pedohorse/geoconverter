/// for now it implements only little endian bgeos
/// not sure why or when would i want to save big endians instead

use std::io::Write;
use crate::geo_struct::{ReaderElement, UniformArrayType};
use crate::bgeo_constants::*;

pub fn to_bjson(element: &ReaderElement, output: &mut dyn Write) {
    let mut buff = [0_u8; 5];
    buff[0] = JID_MAGIC;
    buff[1..5].copy_from_slice(&BINARY_MAGIC_SWAP);
    output.write(&buff).expect(ERRMSG);

    write_element(output, element);
}

//
const ERRMSG: &str = "unexpected end of stream while reading length";


fn write_length(output: &mut dyn Write, len: usize) {
    let mut buff = [0_u8; 9];
    match len {
        x if x < 0xf1 => {
            buff[0] = x as u8;
            output.write(&buff[..1]).expect(ERRMSG);
        }
        x if x <= 0xffff => {
            buff[0] = 0xf0 + 2;
            buff[1..3].copy_from_slice(&(x as u16).to_le_bytes());
            output.write(&buff[..3]).expect(ERRMSG);
        }
        x if x <= 0xffffffff => {
            buff[0] = 0xf0 + 4;
            buff[1..5].copy_from_slice(&(x as u32).to_le_bytes());
            output.write(&buff[..5]).expect(ERRMSG);
        }
        x if x <= 0xffffffffffffffff => {
            buff[0] = 0xf0 + 8;
            buff[1..9].copy_from_slice(&(x as u64).to_le_bytes());
            output.write(&buff[..9]).expect(ERRMSG);
        }
        _ => {
            panic!("len is too be to be saved!");
        }
    }
}

fn write_string(output: &mut dyn Write, string: &String) {
    let bytes = string.as_bytes();
    write_length(output, bytes.len());
    output.write(bytes).expect(ERRMSG);
}

// TODO make into a trait that both geo/bgeo serializers implement
fn write_element(output: &mut dyn Write, elem: &ReaderElement) {
    let mut buf = [0_u8; 9];

    match elem {
        ReaderElement::Array(arr) => {
            buf[0] = JID_ARRAY_BEGIN;
            buf[1] = JID_ARRAY_END;
            output.write(&buf[..1]).expect(ERRMSG);  // write begin
            for subelem in arr {
                write_element(output, subelem)
            }
            output.write(&buf[1..2]).expect(ERRMSG);  // write end
        }
        ReaderElement::KeyValueObject(kvo) => {
            buf[0] = JID_MAP_BEGIN;
            buf[1] = JID_MAP_END;
            buf[2] = JID_STRING;
            output.write(&buf[..1]).expect(ERRMSG);  // write begin
            for (key, val) in kvo.iter() {
                output.write(&buf[2..3]).expect(ERRMSG);  // write string jid
                write_string(output, key);
                write_element(output, val);
            }
            output.write(&buf[1..2]).expect(ERRMSG);  // write end
        }
        ReaderElement::UniformArray(uarr) => {
            buf[0] = JID_UNIFORM_ARRAY;
            output.write(&buf[..1]).expect(ERRMSG);
            match uarr {
                UniformArrayType::UniformArrayTu8(vec) => {
                    buf[1] = JID_UINT8;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &u8::to_le_bytes);
                }
                UniformArrayType::UniformArrayTu16(vec) => {
                    buf[1] = JID_UINT16;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &u16::to_le_bytes);
                }
                UniformArrayType::UniformArrayTi8(vec) => {
                    buf[1] = JID_INT8;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &i8::to_le_bytes);
                }
                UniformArrayType::UniformArrayTi16(vec) => {
                    buf[1] = JID_INT16;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &i16::to_le_bytes);
                }
                UniformArrayType::UniformArrayTi32(vec) => {
                    buf[1] = JID_INT32;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &i32::to_le_bytes);
                }
                UniformArrayType::UniformArrayTi64(vec) => {
                    buf[1] = JID_INT64;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &i64::to_le_bytes);
                }
                UniformArrayType::UniformArrayTf16(vec) => {
                    // TODO: implement actual f16 writing !
                    buf[1] = JID_REAL32;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &f32::to_le_bytes);
                }
                UniformArrayType::UniformArrayTf32(vec) => {
                    buf[1] = JID_REAL32;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &f32::to_le_bytes);
                }
                UniformArrayType::UniformArrayTf64(vec) => {
                    buf[1] = JID_REAL64;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_uniform_array(output, vec, &f64::to_le_bytes);
                }
                UniformArrayType::UniformArrayTbool(vec) => {
                    buf[1] = JID_BOOL;
                    output.write(&buf[1..2]).expect(ERRMSG);
                    write_length(output, vec.len());
                    let mut packed = 0_u32;
                    let mut i = 0;
                    for val in vec {
                        packed |= (*val as u32) << i;
                        i += 1;
                        if i == 32 {
                            output.write(&packed.to_le_bytes()).expect(ERRMSG);
                            i = 0;
                            packed = 0;
                        }
                    }
                    if i != 32 {  // so if last packed was not written
                        output.write(&packed.to_le_bytes()).expect(ERRMSG);
                    }
                }
            }
        }
        ReaderElement::Int(i) => {
            buf[0] = JID_INT64;
            buf[1..9].copy_from_slice(&i.to_le_bytes());
            output.write(&buf[..9]).expect(ERRMSG);
        }
        ReaderElement::Float(f) => {
            buf[0] = JID_REAL64;
            buf[1..9].copy_from_slice(&f.to_le_bytes());
            output.write(&buf[..9]).expect(ERRMSG);
        }
        ReaderElement::Text(t) => {
            buf[0] = JID_STRING;
            output.write(&buf[..1]).expect(ERRMSG);
            write_string(output, t);
        }
        ReaderElement::Bool(b) => {
            buf[0] = JID_BOOL;
            buf[1] = *b as u8;
            output.write(&buf[..2]).expect(ERRMSG);
        }
        ReaderElement::None => {
            buf[0] = JID_NULL;
            output.write(&buf[..1]).expect(ERRMSG);
        }
    };
}


fn write_uniform_array<T: Copy, const N: usize>(output: &mut dyn Write, array: &Vec<T>, tobytes_func: & dyn Fn(T) -> [u8; N]) {
    write_length(output, array.len());
    for el in array {
        output.write(&(tobytes_func(*el))).expect(ERRMSG);
    }
}