use std::mem::transmute;

/// converts a u32 into its corresponding bytes
pub fn get_bytes_u32(val: u32) -> [u8;4] {
    let bytes: [u8;4] = unsafe { transmute(val.to_le()) };
    bytes
}

/// gets the u32 number of a byte array
pub fn get_u32(bytes: [u8;4]) -> u32 {
    unsafe {transmute(bytes)}
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