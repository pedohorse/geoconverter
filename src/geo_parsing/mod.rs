mod parsing_ascii;
mod parsing_bin;

use crate::geo_struct::ReaderElement;
use crate::bgeo_constants::JID_MAGIC;
use parsing_ascii::parse_ascii_first_byte_separately;
use parsing_bin::parse_binary_first_byte_separately;

pub use parsing_ascii::parse_ascii;
pub use parsing_bin::parse_binary;


pub fn parse(input: &mut dyn std::io::Read) -> ReaderElement {
    let mut buf = [0_u8; 1];
    input.read_exact(&mut buf).expect("failed to read magic header");

    if buf[0] == JID_MAGIC {
        parse_binary_first_byte_separately(buf[0], input)
    } else {
        parse_ascii_first_byte_separately(buf[0], input)
    }
}
