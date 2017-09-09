pub fn uint32_from_bytes(bytes : [u8; 4]) -> u32 {
    return ((bytes[0] as u32) << 24) +
        ((bytes[1] as u32) << 16) +
        ((bytes[2] as u32) << 8) +
        bytes[3] as u32;
}

pub fn uint16_from_bytes(bytes : [u8; 2]) -> u16 {
    return ((bytes[0] as u16) << 8) + bytes[1] as u16;
}
