use std::io::Read;

use crate::geo_struct::{ReaderElement, UniformArrayType};
use crate::f16_half::{half_from_be_bytes, half_from_le_bytes};
use crate::bgeo_constants::*;
use std::collections::HashMap;


#[derive(Debug)]
enum ReaderElementOption {
    Some(ReaderElement),
    ArrayEndToken,
    MapEndToken,
    ValueSeparatorToken
}

struct BgeoParser<'a> {
    chan: &'a mut dyn Read,
    tokens: HashMap<usize, String>,
    u16_from_bytes: &'static dyn Fn([u8; 2]) -> u16,
    u32_from_bytes: &'static dyn Fn([u8; 4]) -> u32,
    u64_from_bytes: &'static dyn Fn([u8; 8]) -> u64,
    i16_from_bytes: &'static dyn Fn([u8; 2]) -> i16,
    i32_from_bytes: &'static dyn Fn([u8; 4]) -> i32,
    i64_from_bytes: &'static dyn Fn([u8; 8]) -> i64,
    f16_32_from_bytes: &'static dyn Fn([u8; 2]) -> f32,  // let's not work with halfs
    f32_from_bytes: &'static dyn Fn([u8; 4]) -> f32,
    f64_from_bytes: &'static dyn Fn([u8; 8]) -> f64,
}

impl<'a> BgeoParser<'a> {
    fn new_be(channel: &'a mut dyn Read) -> BgeoParser {
        BgeoParser {
            chan: channel,
            tokens: HashMap::new(),
            u16_from_bytes: &u16::from_be_bytes,
            u32_from_bytes: &u32::from_be_bytes,
            u64_from_bytes: &u64::from_be_bytes,
            i16_from_bytes: &i16::from_be_bytes,
            i32_from_bytes: &i32::from_be_bytes,
            i64_from_bytes: &i64::from_be_bytes,
            f16_32_from_bytes: &half_from_be_bytes,
            f32_from_bytes: &f32::from_be_bytes,
            f64_from_bytes: &f64::from_be_bytes
        }
    }

    fn new_le(channel: &'a mut dyn Read) -> BgeoParser {
        BgeoParser {
            chan: channel,
            tokens: HashMap::new(),
            u32_from_bytes: &u32::from_le_bytes,
            u64_from_bytes: &u64::from_le_bytes,
            u16_from_bytes: &u16::from_le_bytes,
            i16_from_bytes: &i16::from_le_bytes,
            i32_from_bytes: &i32::from_le_bytes,
            i64_from_bytes: &i64::from_le_bytes,
            f16_32_from_bytes: &half_from_le_bytes,
            f32_from_bytes: &f32::from_le_bytes,
            f64_from_bytes: &f64::from_le_bytes
        }
    }

    fn parse_read_length(&mut self) -> usize {
        const ERRMSG: &str = "unexpected end of stream while reading length";
        let mut buff: [u8; 8] = [0; 8];
        self.chan.read_exact(&mut buff[..1]).expect(ERRMSG);

        match buff[0] {
            x if x<0xf1 => x as usize, 
            x if x == 0xf0 + 2 => {
                self.chan.read_exact(&mut buff[..2]).expect(ERRMSG);
                (self.u16_from_bytes)(buff[..2].try_into().expect("")) as usize
            }
            x if x == 0xf0 + 4 => {
                self.chan.read_exact(&mut buff[..4]).expect(ERRMSG);
                (self.u32_from_bytes)(buff[..4].try_into().expect("")) as usize
            }
            x if x == 0xf0 + 8 => {
                self.chan.read_exact(&mut buff[..8]).expect(ERRMSG);
                (self.u64_from_bytes)(buff[..8].try_into().expect("")) as usize
            }
            _ => panic!("unexpected length format")
        }
    }

    fn parse_string(&mut self) -> String {
        let len = self.parse_read_length();
        let mut buff: Vec<u8> = vec![0; len];
        self.chan.read_exact(&mut buff[..]).expect("unexpected end of buffer while reading string");

        String::from_utf8(buff).expect("malformed utf8 string!")
    }

    fn parse_u8(&mut self) -> u8 {
        let mut buff = [0_u8; 1];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading u8");
        buff[0]
    }

    fn parse_u16(&mut self) -> u16 {
        let mut buff = [0_u8; 2];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading u16");
        (self.u16_from_bytes)(buff)
    }

    // currently not needed
    // 
    // fn parse_u32(&mut self) -> u32 {
    //     let mut buff = [0_u8; 4];
    //     self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading u32");
    //     (self.u32_from_bytes)(buff)
    // }

    // fn parse_u64(&mut self) -> u64 {
    //     let mut buff = [0_u8; 8];
    //     self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading u64");
    //     (self.u64_from_bytes)(buff)
    // }

    fn parse_i8(&mut self) -> i8 {
        let mut buff = [0_u8; 1];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading i8");
        buff[0] as i8
    }

    fn parse_i16(&mut self) -> i16 {
        let mut buff = [0_u8; 2];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading i16");
        (self.i16_from_bytes)(buff)
    }

    fn parse_i32(&mut self) -> i32 {
        let mut buff = [0_u8; 4];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading i32");
        (self.i32_from_bytes)(buff)
    }

    fn parse_i64(&mut self) -> i64 {
        let mut buff = [0_u8; 8];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading i64");
        (self.i64_from_bytes)(buff)
    }

    fn parse_f16(&mut self) -> f32 {
        let mut buff = [0_u8; 2];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading f32");
        (self.f16_32_from_bytes)(buff)
    }

    fn parse_f32(&mut self) -> f32 {
        let mut buff = [0_u8; 4];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading f32");
        (self.f32_from_bytes)(buff)
    }

    fn parse_f64(&mut self) -> f64 {
        let mut buff = [0_u8; 8];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading f64");
        (self.f64_from_bytes)(buff)
    }

    fn parse_token_def_binary(&mut self) {
        let token_id = self.parse_read_length();
        let s = self.parse_string();
        self.tokens.insert(token_id, s);
    }

    fn parse_token_undef_binary(&mut self) {
        let token_id = self.parse_read_length();
        self.tokens.remove(&token_id).expect(format!("token {:?} was not defiend", token_id).as_str());
    }

    fn parse_one_element_binary(&mut self) -> ReaderElementOption {
        let mut buff = [0_u8; 8];

        self.chan.read_exact(&mut buff[..1]).expect("unexpected end of buffer while reading token type");
        let mut next_type_byte = buff[0];

        loop {
            match next_type_byte {
                JID_TOKENDEF => {
                    self.parse_token_def_binary();     
                },
                JID_TOKENUNDEF => {
                    self.parse_token_undef_binary();
                },
                _ =>  break
            }
            self.chan.read_exact(&mut buff[..1]).expect("unexpected end of buffer while reading token type");
            next_type_byte = buff[0];
        }

        match next_type_byte {
            JID_MAP_BEGIN => {
                let mut map = HashMap::new();
                loop {
                    let key = match self.parse_one_element_binary() {
                        ReaderElementOption::Some(ReaderElement::Text(key))  => {
                            key
                        }
                        ReaderElementOption::MapEndToken => {
                            break;
                        }
                        // ReaderElementOption::ValueSeparatorToken => continue,
                        t => {
                            panic!("unexpected token in map: {:?}", t);
                        }
                    };
                    // self.chan.read_exact(&mut buff[..1]).expect("unexpected end of buffer while reading key separator");
                    // if buff[0] != JID_KEY_SEPARATOR {
                    //     panic!("expected key separator after key, got {:?}, key={:?}", buff[0], key);
                    // }
                    let val = match self.parse_one_element_binary() {
                        ReaderElementOption::Some(x) => { x }
                        _ => { panic!("unexpected end of map"); }
                    };
                    map.insert(key, val);
                }
                return ReaderElementOption::Some(ReaderElement::KeyValueObject(map));
            }
            JID_ARRAY_BEGIN => {
                let mut arr = Vec::new();
                loop {
                    let val = match self.parse_one_element_binary() {
                        ReaderElementOption::Some(x) => x,
                        ReaderElementOption::ArrayEndToken => {
                            break;
                        }
                        // ReaderElementOption::ValueSeparatorToken => continue,
                        _ => { panic!("unexpected end of array"); }
                    };
                    arr.push(val);
                }
                return ReaderElementOption::Some(ReaderElement::Array(arr));
            }
            JID_BOOL => {
                self.chan.read_exact(&mut buff[..1]).expect("unexpected end of buffer while reading bool");
                return ReaderElementOption::Some(ReaderElement::Bool(buff[0] != 0));
            }
            JID_FALSE => {
                return ReaderElementOption::Some(ReaderElement::Bool(false));
            }
            JID_TRUE => {
                return ReaderElementOption::Some(ReaderElement::Bool(true));
            }
            token @ (JID_INT8 | JID_INT16 | JID_INT32 | JID_INT64) => {
                return ReaderElementOption::Some(ReaderElement::Int(match token {
                    JID_INT8 => self.parse_i8() as i64,
                    JID_INT16 => self.parse_i16() as i64,
                    JID_INT32 => self.parse_i32() as i64,
                    JID_INT64 => self.parse_i64(),
                    _ => { panic!("very unexpected type of int {:?}", token); }
                }));
            }
            token @ (JID_UINT8 | JID_UINT16) => {
                return ReaderElementOption::Some(ReaderElement::Int(match token {
                    JID_UINT8 => self.parse_u8() as i64,
                    JID_UINT16 => self.parse_u16() as i64,
                    _ => { panic!("very unexpected type of int {:?}", token); }
                }));
            }
            token @ (JID_REAL16 | JID_REAL32 | JID_REAL64 ) => {
                return ReaderElementOption::Some(ReaderElement::Float(match token {
                    JID_REAL16 => self.parse_f16() as f64,
                    JID_REAL32 => self.parse_f32() as f64,
                    JID_REAL64 => self.parse_f64(),
                    _ => { panic!("very unexpected type of int {:?}", token); }
                }));
            }
            JID_STRING => {
                return ReaderElementOption::Some(ReaderElement::Text(self.parse_string()));
            }
            JID_TOKENREF => {
                let token_id = self.parse_read_length();
                // we duplicate all tokens at this stage, do we care?
                return ReaderElementOption::Some(ReaderElement::Text(self.tokens.get(&token_id).expect(format!("referenced token {} was not defined", token_id).as_str()).to_owned()));
            }
            JID_UNIFORM_ARRAY => {
                self.chan.read_exact(&mut buff[..1]).expect("unexpected end of buffer while reading key uniform array type");
                let array_type = buff[0];
                let array_len = self.parse_read_length();
                let vec_el: ReaderElement = ReaderElement::UniformArray(match array_type {
                    JID_INT8 => {
                        UniformArrayType::UniformArrayTi8(self.parse_uniform_array(array_len, &Self::parse_i8))
                    }
                    JID_INT16 => {  // TODO: make more effective, less repetative
                        UniformArrayType::UniformArrayTi16(self.parse_uniform_array(array_len, &Self::parse_i16))
                    }
                    JID_INT32 => {
                        UniformArrayType::UniformArrayTi32(self.parse_uniform_array(array_len, &Self::parse_i32))
                    }
                    JID_INT64 => {
                        UniformArrayType::UniformArrayTi64(self.parse_uniform_array(array_len, &Self::parse_i64))
                    }
                    JID_UINT8 => {
                        UniformArrayType::UniformArrayTu8(self.parse_uniform_array(array_len, &Self::parse_u8))
                    }
                    JID_UINT16 => {
                        UniformArrayType::UniformArrayTu16(self.parse_uniform_array(array_len, &Self::parse_u16))
                    }
                    JID_REAL16 => {
                        UniformArrayType::UniformArrayTf16(self.parse_uniform_array(array_len, &Self::parse_f16))
                    }
                    JID_REAL32 => {
                        UniformArrayType::UniformArrayTf32(self.parse_uniform_array(array_len, &Self::parse_f32))
                    }
                    JID_REAL64 => {
                        UniformArrayType::UniformArrayTf64(self.parse_uniform_array(array_len, &Self::parse_f64))
                    }
                    JID_BOOL => {
                        // packed blocks of 32 bits
                        let mut remaining_len = array_len;
                        let mut buff = [0_u8; 4];
                        let mut vec = Vec::with_capacity(array_len);
                        while remaining_len > 0 {
                            let nbits = 32.min(remaining_len);

                            self.chan.read_exact(&mut buff).expect("failed to read buffer");
                            let sample = (self.u32_from_bytes)(buff.try_into().expect("failed to convert bits"));
                            for i in 0..nbits {
                                vec.push(sample & (1 << i) != 0);
                            }
                            remaining_len -= nbits;
                        }
                        UniformArrayType::UniformArrayTbool(vec)
                    }
                    JID_TOKENREF => {
                        panic!("not yet implemented!");
                    }
                    JID_STRING => {
                        panic!("not yet implemented!");
                    }
                    _ => panic!("unknown unified array type {}", array_type)
                });
                // we convert uniform array into simple array... is it good enough? 
                // imagine voxel data array - even one byte overhead from enum is a pain,
                // but converint it later in schema parsers is even more of a unnecessary overhead, so
                // TODO: add uniform array to ReaderElement
                return ReaderElementOption::Some(vec_el);
            }
            JID_NULL => {
                return ReaderElementOption::Some(ReaderElement::None);
            }
            JID_MAP_END => {
                return ReaderElementOption::MapEndToken;
            }
            JID_ARRAY_END => {
                return ReaderElementOption::ArrayEndToken;
            }
            JID_VALUE_SEPARATOR => {
                return ReaderElementOption::ValueSeparatorToken;
            }
            x => {
                panic!("unexpected token: {:?}", x);
            }
        }
    }

    fn parse_uniform_array<T>(&mut self, array_len: usize, parse_func: & dyn Fn(&mut Self) -> T) -> Vec<T> {
        let mut vec = Vec::with_capacity(array_len);
        for _ in 0..array_len {
            vec.push(parse_func(self));
        }
        vec
    }
}


pub fn parse_binary_first_byte_separately(first_byte: u8, input: &mut dyn std::io::Read) -> ReaderElement {
    if first_byte != JID_MAGIC {
        panic!("bad magic header!");
    }
    let mut buf = [0_u8; 4];
    input.read_exact(&mut buf[..4]).expect("failed to endian magic");

    let mut parser = match buf {
        BINARY_MAGIC => BgeoParser::new_be(input),
        BINARY_MAGIC_SWAP => BgeoParser::new_le(input),
        _ => panic!("unrecognized binary magic!"),
    };
    return match parser.parse_one_element_binary() {
        ReaderElementOption::Some(x) => x,
        _ => panic!("failed to parse file")
    };
}


pub fn parse_binary(input: &mut dyn std::io::Read) -> ReaderElement {
    let mut buf = [0_u8; 1];
    input.read_exact(&mut buf[..1]).expect("failed to read magic");

    return parse_binary_first_byte_separately(buf[0], input);
}
