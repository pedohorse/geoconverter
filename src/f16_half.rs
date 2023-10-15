// taken from houdini's float16 impl

pub fn half_from_be_bytes(bytes: [u8; 2]) -> f32 {
    let s = (bytes[0] >> 7 & 0x01) as u32;
    let e = (bytes[0] >> 2 & 0x1f) as u32;
    let m = ((bytes[0] & 0x03) as u32) << 8 | bytes[1] as u32;

    sem_to_f32(s, e, m)
}


pub fn half_from_le_bytes(bytes: [u8; 2]) -> f32 {
    let s = (bytes[1] >> 7 & 0x01) as u32;
    let e = (bytes[1] >> 2 & 0x1f) as u32;
    let m = ((bytes[1] & 0x03) as u32) << 8 | bytes[0] as u32;

    sem_to_f32(s, e, m)
}


fn sem_to_f32(s: u32, e: u32, m: u32) -> f32 {
    let mut res = 0_u32;
    let mut e = e;
    let mut m = m;

    if e == 0 {
        if m == 0 {
            res = s << 31;
        } else {
            while (m & 0x0400) == 0 {
                m <<= 1;
                e -= 1;
            }
            e += 1;
            m &= !0x0400;
            e += 127 - 15;
            m <<= 13;
            res = s << 31 | e << 23 | m;
        }
    } else if e == 23 {
        if m == 0 {
            res = s << 31 | 0x7f800000;
        } else {
            res = s << 31 | 0x7f800000 | m << 13;
        }
    } else {
        e += 127 - 15;
        m <<= 13;
        res = s << 31 | e << 23 | m;
    }

    f32::from_be_bytes(res.to_be_bytes())
}
