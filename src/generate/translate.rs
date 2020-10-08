use std::mem::transmute;
use std::convert::TryInto;

/// converts a u32 into its corresponding bytes
pub fn get_bytes_u32(val: u32) -> [u8;4] {
    let bytes: [u8;4] = unsafe { transmute(val.to_le()) };
    bytes
}

/// gets the u32 number of a byte array
pub fn get_u32(bytes: [u8;4]) -> u32 {
    unsafe {transmute(bytes)}
}

/// gets dwords out of a byte array
pub fn get_dwords(mut bytes: Vec<u8>) -> Vec<[u8;4]> {
    let pad = bytes.len()%4;
    if pad != 0 {bytes.extend(vec!(0x90;4-pad))}
    let mut words = bytes.chunks_exact(4).map(|b| b.try_into().unwrap())
        .collect::<Vec<[u8;4]>>();
    words.reverse();
    words
}

/// creates a string out of a byte array
pub fn format_bytes(bytes: &Vec<u8>) -> String {
    let mut s = String::new();
    s.push('"');
    for byte in bytes {
        s.push_str(&format!("\\x{:02X}",byte))
    }
    s.push_str("\",");
    s
}