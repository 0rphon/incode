use std::mem::transmute;


/// strips trailing zeros from a byte array
/// [00, FF, 00, 00] -> [00, FF]
/// 0x0000FF00 -> 0xFF00
pub fn strip_trailing_zero(bytes: &Vec<u8>) -> Vec<u8> {
    let mut bytes = bytes.clone().into_iter().rev().skip_while(|x| *x == 0_u8).collect::<Vec<u8>>();
    bytes.reverse();
    if bytes.len()==3 {bytes.push(0)} 
    bytes
}

/// converts a u32 into its full dword
pub fn get_full_bytes32(val: u32) -> Vec<u8> {
    let bytes: [u8;4] = unsafe { transmute(val.to_le()) };
    bytes.to_vec()
}

/// converts a u32 into bytes
pub fn get_bytes32(val: u32) -> Vec<u8> {
    strip_trailing_zero(&get_full_bytes32(val))
}

/// converts a u16 into its full word
pub fn get_full_bytes16(val: u16) -> Vec<u8> {
    let bytes: [u8;2] = unsafe { transmute(val.to_le()) };
    bytes.to_vec()
}


/// adds 0x00's to a byte array to create its dword
pub fn to_dword(bytes: &mut Vec<u8>) {
    let pad = bytes.len()%4;
    if pad != 0 {bytes.extend(vec!(0x90;4-pad))}
}

/// gets the first dword of a byte array
pub fn get32(bytes: &Vec<u8>) -> u32 {
    let mut bytes = bytes.clone();
    to_dword(&mut bytes);
    let b: [u8;4] = [bytes[0], bytes[1], bytes[2], bytes[3]];
    let val: u32 = unsafe {transmute(b)};
    val
}

/// gets dwords out of a byte array
pub fn get_dwords(bytes: &Vec<u8>) -> Vec<u32> {
    let mut bytes = bytes.clone();
    to_dword(&mut bytes);
    let mut words = bytes.chunks_exact(4).map(|b| get32(&b.to_vec()))
        .collect::<Vec<u32>>();
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