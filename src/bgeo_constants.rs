
pub const JID_NULL: u8 = 0x00;
pub const JID_MAP_BEGIN: u8 = 0x7b;
pub const JID_MAP_END: u8 = 0x7d;
pub const JID_ARRAY_BEGIN: u8 = 0x5b;
pub const JID_ARRAY_END: u8 = 0x5d;
pub const JID_BOOL: u8 = 0x10;
pub const JID_INT8: u8 = 0x11;
pub const JID_INT16: u8 = 0x12;
pub const JID_INT32: u8 = 0x13;
pub const JID_INT64: u8 = 0x14;
pub const JID_REAL16: u8 = 0x18;
pub const JID_REAL32: u8 = 0x19;
pub const JID_REAL64: u8 = 0x1a;
pub const JID_UINT8: u8 = 0x21;
pub const JID_UINT16: u8 = 0x22;
pub const JID_STRING: u8 = 0x27;
pub const JID_FALSE: u8 = 0x30;
pub const JID_TRUE: u8 = 0x31;
pub const JID_TOKENDEF: u8 = 0x2b;
pub const JID_TOKENREF: u8 = 0x26;
pub const JID_TOKENUNDEF: u8 = 0x2d;
pub const JID_UNIFORM_ARRAY: u8 = 0x40;
pub const JID_KEY_SEPARATOR: u8 = 0x3a;
pub const JID_VALUE_SEPARATOR: u8 = 0x2c;
pub const JID_MAGIC: u8 = 0x7f;

pub const BINARY_MAGIC: [u8; 4] = [0x62, 0x4a, 0x53, 0x4e];
pub const BINARY_MAGIC_SWAP: [u8; 4] = [0x4e, 0x53, 0x4a, 0x62];