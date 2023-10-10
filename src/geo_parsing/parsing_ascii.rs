use crate::geo_struct::ReaderElement;
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

    pub fn peek(&mut self) -> Option<u8> {
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
        let x = self.peek();
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

    pub fn peek_skip_whitespaces(&mut self) -> Option<u8> {
        loop {
            match self.peek() {
                Some(x) if x.is_ascii_whitespace() => { self.consume(); }
                Some(x) => { return Some(x) }
                None => { return None }
            };
        };
    }

    pub fn get_skip_whitespaces(&mut self) -> Option<u8> {
        let x = self.peek_skip_whitespaces();
        self.i += 1;
        x
    }

}

fn parse_one_element(chan: &mut BuffChannel) -> ReaderElement {
    let mut state = ReaderState::Off;

    let mut value = ReaderElement::None;

    match chan.peek_skip_whitespaces() {
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
                let c = chan.peek().expect("unexpected end of file");
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
            if chan.peek_skip_whitespaces().expect("unable to parse file") != b']' {
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
            if chan.peek_skip_whitespaces().expect("unable to parse file") != b'}' {
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
                let char = chan.peek().expect("unexpected end of file");
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
            if !chan.peek().expect("internal error").is_ascii_alphabetic() {
                panic!("internal error");
            }
            chan.reset_buffer();

            loop {
                let char = chan.peek().expect("unexpected end of file");
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


pub fn parse_ascii(input: &mut dyn std::io::Read) -> ReaderElement {
    //let mut buf = String::new();

    let mut chan = BuffChannel::new(input, 1024 * 128, 0);
    return parse_one_element(&mut chan);
}
