use std::io::Read;

use crate::geo_struct::ReaderElement;
use crate::f16_half::{half_from_be_bytes, half_from_le_bytes};
use std::collections::HashMap;

#[derive(Debug)]
enum ReaderState {
    Off,
    Keyword,
    Text,
    Number,
    Array,
    KeyValueObject,
}

struct BuffChannel<'a> {
    buff: Vec<u8>,
    chunk_size: usize,
    start: usize,
    i: usize,
    src: &'a mut dyn std::io::Read,
}

impl<'a> BuffChannel<'a> {
    pub fn new(
        file: &'a mut dyn std::io::Read,
        buff_size: usize,
        chunk_size: usize,
    ) -> BuffChannel<'a> {
        let buffer = Vec::with_capacity(buff_size);
        let chunk_size = if chunk_size == 0 {
            buff_size
        } else {
            chunk_size
        };
        BuffChannel {
            buff: buffer,
            chunk_size: chunk_size,
            start: 0,
            i: 0,
            src: file,
        }
    }

    fn buffer_moar(&mut self) -> Option<()> {
        let chunk = self.chunk_size;
        let old_size = self.buff.len();
        self.buff.resize(old_size + chunk, 0);

        let read_bytes = self
            .src
            .read(&mut self.buff[old_size..])
            .expect("read error");
        self.buff.resize(old_size + read_bytes, 0);

        let mut offset = 0;
        for i in old_size..self.buff.len() {
            if self.buff[i].is_ascii_whitespace() {  // TODO: i'm an idiot - this eats spaces from string constants
                offset += 1;
                continue;
            }
            self.buff[i - offset] = self.buff[i];
        }
        self.buff.resize(self.buff.len() - offset, 0);

        if read_bytes == 0 {
            return None;
        }
        Some(())
    }

    pub fn peak(&mut self) -> Option<u8> {
        //println!("{} {} {:?}", self.start, self.i, self.buff);
        if self.i == self.buff.len() {
            // buffer is empty
            if self.buffer_moar() == None {
                return None;
            }
        };
        return Some(self.buff[self.i]);
    }

    pub fn consume(&mut self) -> Option<()> {
        if self.i == self.buff.len() {
            if self.buffer_moar() == None {
                return None;
            };
        };
        self.i += 1;
        Some(())
    }

    pub fn get(&mut self) -> Option<u8> {
        let x = self.peak();
        self.i += 1;
        x
    }

    pub fn buffer(&self) -> &[u8] {
        return &self.buff[self.start..self.i];
    }

    pub fn reset_buffer(&mut self) {
        if self.i == self.buff.len() {
            self.buff.clear();
            self.i = 0;
            self.start = 0;
        } else {
            // otherwise we move start, but don't clear buffer (we could truncate it tho, but me lazy)
            self.start = self.i;
        }
    }

    pub fn peak_skip_whitespaces(&mut self) -> Option<u8> {
        loop {
            match self.peak() {
                Some(x) if x.is_ascii_whitespace() => { self.consume(); }
                Some(x) => { return Some(x) }
                None => { return None }
            };
        };
    }

    pub fn get_skip_whitespaces(&mut self) -> Option<u8> {
        let x = self.peak_skip_whitespaces();
        self.i += 1;
        x
    }

}

fn parse_one_element(chan: &mut BuffChannel) -> ReaderElement {
    let mut state = ReaderState::Off;

    let mut value = ReaderElement::None;

    match chan.peak_skip_whitespaces() {
        Some(b'[') => {
            state = ReaderState::Array;
        }
        Some(c) if c.is_ascii_digit() || c == b'.' || c == b'-' || c == b'+' => {
            state = ReaderState::Number;
        }
        Some(c) if c.is_ascii_alphabetic() => {
            state = ReaderState::Keyword;
        }
        Some(b'{') => {
            state = ReaderState::KeyValueObject;
        }
        Some(b'"') => {
            state = ReaderState::Text;
        }
        Some(smth) => {
            panic!("unexpected file structure! got {:?}", smth);
        } // TODO: add position information
        None => {} // normal channel closure, we exit
    }
    // println!("reading {:?}", state);

    match state {
        ReaderState::Off => {} // nothing, though it should happen only on empty input
        ReaderState::Number => {
            chan.reset_buffer();
            loop {
                let c = chan.peak().expect("unexpected end of file");
                if !c.is_ascii_digit() {
                    if c == b'.' {
                        // we should ensure there's one dot, but meh
                    } else if c == b'-' || c == b'+' {
                        // we should ensure it's in front, but meh
                    } else if c == b'e' {
                        // we should properly check exp, but meh
                    } else {
                        // no parsing exp part etc for now
                        break;
                    }
                }
                chan.consume().expect("unexpected end of file");
            }
            match std::str::from_utf8(chan.buffer())
                .expect("bad number")
                .parse::<i64>()
            {
                Ok(number) => {
                    value = ReaderElement::Int(number);
                }
                Err(_) => {
                    match std::str::from_utf8(chan.buffer())
                        .expect("bad number")
                        .parse::<f64>()
                    {
                        Ok(number) => {
                            value = ReaderElement::Float(number);
                        }
                        Err(_) => {
                            panic!("bad number !! '{:?}'", chan.buffer())
                        }
                    }
                }
            }
        }
        ReaderState::Array => {
            let mut arr: Vec<ReaderElement> = Vec::new();
            if chan.get().expect("internal error") != b'[' {
                panic!("internal error");
            }
            if chan.peak_skip_whitespaces().expect("unable to parse file") != b']' {
                // empty array
                loop {
                    let arr_value = parse_one_element(chan);
                    arr.push(arr_value);
                    match chan.get_skip_whitespaces() {
                        Some(b',') => continue,
                        Some(b']') => break,
                        x => panic!(
                            "bad array!, saw {:?}, arr {:?}",
                            char::from_u32(x.expect("") as u32),
                            arr
                        ),
                    }
                }
            } else {
                chan.consume();
            }; // eat that ]
            value = ReaderElement::Array(arr);
        }
        ReaderState::KeyValueObject => {
            let mut hmap: HashMap<String, ReaderElement> = HashMap::new();
            if chan.get().expect("internal error") != b'{' {
                panic!("internal error");
            }
            if chan.peak_skip_whitespaces().expect("unable to parse file") != b'}' {
                // empty map
                loop {
                    let mkey = parse_one_element(chan);
                    if chan.get_skip_whitespaces().expect("internal error") != b':' {
                        panic!("internal error");
                    }
                    let mval = parse_one_element(chan);
                    if let ReaderElement::Text(text) = mkey {
                        hmap.insert(text, mval);
                    } else {
                        panic!("non-string keys are not yet supported")
                    }
                    match chan.get_skip_whitespaces() {
                        Some(b',') => continue,
                        Some(b'}') => break,
                        _ => panic!("bad dict!"),
                    }
                }
            } else {
                chan.consume();
            }; // eat that }
            value = ReaderElement::KeyValueObject(hmap);
        }
        ReaderState::Text => {
            if chan.get().expect("internal error") != b'"' {
                panic!("internal error");
            }
            chan.reset_buffer();

            let mut next_escaped = false;
            loop {
                let char = chan.peak().expect("unexpected end of file");
                if next_escaped {
                    next_escaped = false;
                } else if char == b'\\' {
                    next_escaped = true;
                    chan.consume();
                    continue;
                } else if char == b'"' {
                    break;
                };
                chan.consume();
            }
            value = ReaderElement::Text(String::from_utf8_lossy(chan.buffer()).to_string());
            chan.consume(); // eat closing "
        }
        ReaderState::Keyword => {
            if !chan.peak().expect("internal error").is_ascii_alphabetic() {
                panic!("internal error");
            }
            chan.reset_buffer();

            loop {
                let char = chan.peak().expect("unexpected end of file");
                if !char.is_ascii_alphanumeric() {
                    break;
                };
                chan.consume();
            }
            match chan.buffer() {
                b"true" => {
                    value = ReaderElement::Bool(true);
                }
                b"false" => {
                    value = ReaderElement::Bool(false);
                }
                _ => {
                    panic!(
                        "unknown keyword '{}'",
                        String::from_utf8_lossy(chan.buffer())
                    );
                }
            }
        }
    };
    return value;
}


const JID_NULL: u8 = 0x00;
const JID_MAP_BEGIN: u8 = 0x7b;
const JID_MAP_END: u8 = 0x7d;
const JID_ARRAY_BEGIN: u8 = 0x5b;
const JID_ARRAY_END: u8 = 0x5d;
const JID_BOOL: u8 = 0x10;
const JID_INT8: u8 = 0x11;
const JID_INT16: u8 = 0x12;
const JID_INT32: u8 = 0x13;
const JID_INT64: u8 = 0x14;
const JID_REAL16: u8 = 0x18;
const JID_REAL32: u8 = 0x19;
const JID_REAL64: u8 = 0x1a;
const JID_UINT8: u8 = 0x21;
const JID_UINT16: u8 = 0x22;
const JID_STRING: u8 = 0x27;
const JID_FALSE: u8 = 0x30;
const JID_TRUE: u8 = 0x31;
const JID_TOKENDEF: u8 = 0x2b;
const JID_TOKENREF: u8 = 0x26;
const JID_TOKENUNDEF: u8 = 0x2d;
const JID_UNIFORM_ARRAY: u8 = 0x40;
const JID_KEY_SEPARATOR: u8 = 0x3a;
const JID_VALUE_SEPARATOR: u8 = 0x2c;
const JID_MAGIC: u8 = 0x7f;

const BINARY_MAGIC: [u8; 4] = [0x62, 0x4a, 0x53, 0x4e];
const BINARY_MAGIC_SWAP: [u8; 4] = [0x4e, 0x53, 0x4a, 0x62];

#[derive(Debug)]
enum ReaderElementOption {
    Some(ReaderElement),
    ArrayEndToken,
    MapEndToken,
    ValueSeparatorToken,
    None
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

    fn parse_u32(&mut self) -> u32 {
        let mut buff = [0_u8; 4];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading u32");
        (self.u32_from_bytes)(buff)
    }

    fn parse_u64(&mut self) -> u64 {
        let mut buff = [0_u8; 8];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading u64");
        (self.u64_from_bytes)(buff)
    }

    fn parse_i8(&mut self) -> i8 {
        let mut buff = [0_u8; 1];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading i8");
        buff[0] as i8
    }

    fn parse_i16(&mut self) -> i16 {
        let mut buff = [0_u8; 2];
        self.chan.read_exact(&mut buff).expect("unexpected end of buffer while reading i16");;
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
                let vec_el: ReaderElement = ReaderElement::Array(match array_type {
                    JID_INT8 => {
                        self.parse_uniform_array(array_len, &Self::parse_i8)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Int(*x as i64) }).collect()
                    }
                    JID_INT16 => {  // TODO: make more effective, less repetative
                        self.parse_uniform_array(array_len, &Self::parse_i16)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Int(*x as i64) }).collect()
                    }
                    JID_INT32 => {
                        self.parse_uniform_array(array_len, &Self::parse_i32)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Int(*x as i64) }).collect()
                    }
                    JID_INT64 => {
                        self.parse_uniform_array(array_len, &Self::parse_i64)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Int(*x as i64) }).collect()
                    }
                    JID_UINT8 => {
                        self.parse_uniform_array(array_len, &Self::parse_u8)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Int(*x as i64) }).collect()
                    }
                    JID_UINT16 => {
                        self.parse_uniform_array(array_len, &Self::parse_u16)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Int(*x as i64) }).collect()
                    }
                    JID_REAL16 => {
                        self.parse_uniform_array(array_len, &Self::parse_f16)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Float(*x as f64) }).collect()
                    }
                    JID_REAL32 => {
                        self.parse_uniform_array(array_len, &Self::parse_f32)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Float(*x as f64) }).collect()
                    }
                    JID_REAL64 => {
                        self.parse_uniform_array(array_len, &Self::parse_f64)
                            .iter()
                            .map(|x| -> ReaderElement { ReaderElement::Float(*x as f64) }).collect()
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


pub fn parse_ascii(input: &mut dyn std::io::Read) -> ReaderElement {
    //let mut buf = String::new();

    let mut chan = BuffChannel::new(input, 1024 * 128, 0);
    return parse_one_element(&mut chan);
}


pub fn parse_binary(input: &mut dyn std::io::Read) -> ReaderElement {
    let mut buf = [0_u8; 4];
    input.read_exact(&mut buf[..1]).expect("failed to read magic");

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
