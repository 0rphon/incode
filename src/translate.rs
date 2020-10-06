use dynerr::*;

use std::convert::TryInto;
use itertools::Itertools;
use std::mem::transmute;
use regex::Regex;


///parses input for its hex values
pub fn parse_bytes(input: &str) -> DynResult<Vec<u8>> {
    let mut parsed = Regex::new(r"(0x)|[^A-Fa-f0-9]")?
        .replace_all(&input, "").to_string();
    if parsed.len()%2 != 0 {parsed.insert(0,'0')}
    let mut bytes = parsed.chars().chunks(2).into_iter()
        .map(|b| Ok(u8::from_str_radix(&b.collect::<String>(), 16)?))
        .collect::<DynResult<Vec<u8>>>()?;
    let pad = bytes.len() % 4;
    if pad != 0 {bytes.extend(vec!(0x90;4-pad))}
    Ok(bytes)
}

pub fn get_words(bytes: &Vec<u8>) -> Vec<[u8;4]> {
    let mut words = bytes.chunks_exact(4).map(|b| b.try_into().unwrap())
        .collect::<Vec<[u8;4]>>();
    words.reverse();
    words
}

///ONLY WORKS FOR X86
pub fn get_bytes_u32(val: u32) -> [u8;4] {
    let bytes: [u8;4] = unsafe { transmute(val.to_le()) };
    bytes
}

///ONLY WORKS FOR X86
/// will be backwards from the passed bytes obviously
pub fn get_u32(bytes: [u8;4]) -> u32 {
    unsafe {transmute(bytes)}
}


pub fn format_bytes(bytes: &Vec<u8>) -> String {
    let mut s = String::new();
    s.push('"');
    for byte in bytes {
        s.push_str(&format!("\\x{:02X}",byte))
    }
    s.push_str("\",");
    s
}