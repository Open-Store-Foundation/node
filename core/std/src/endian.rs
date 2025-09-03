use byteorder::{ByteOrder, LittleEndian};

pub fn get_u16_le(slice: &[u8]) -> u16  {
    return LittleEndian::read_u16(slice)
}

pub fn get_u64_le(slice: &[u8]) -> u64 {
    return LittleEndian::read_u64(slice)
}

pub fn get_u32_le(slice: &[u8]) -> u32 {
    return LittleEndian::read_u32(slice)
}

pub fn put_u32_le(buf: &mut [u8], n: u32) {
    LittleEndian::write_u32(buf, n);
}