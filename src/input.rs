use dynerr::*;

use std::{fmt, error};
use std::env::args;

use itertools::Itertools;
use regex::Regex;
use std::num;


const CODE: &str = "--code";
const ESP: &str  = "--esp";
const EIP: &str  = "--eip";
const JUMP: &str = "--jump";
const HELP: &str = "--help";
pub const SEE_HELP: &str = "Try --help for usage.";
pub const HELP_MESSAGE: &str = "InCode is an ASCII encoder for x86 shellcode. It has tools to handle wrapping, positioning, and jumping.
This is a tool I wrote for personal security research. I obviously accept no responsibility for how other
people use it.

Usage:
    --code [bytes]:     Wrap an instruction in x86 ascii shellcode that gets decoded to [esp] at runtime.

    --esp [addr] --eip [addr]:
                        What the addresses of ESP and EIP will be at the first byte of this payload. While 
                        these values wont stay the same between runs, their difference will. Use this if 
                        you want the payload to handle unpacking on its own.

    --jump [addr]:      Generate a wrapped far jump from [eip] to [addr] that gets decoded to [esp] at 
                        runtime. Requires --esp and --eip to be set. It will handle esp positioning for 
                        you.

    --help:             Show this screen and exit.

Examples:
    Generate ASCII wrapped payload that decodes given values in memory:
        incode.exe \\xF3\\xE9\\xB8\\x00\\x33\\x4A\\x41

    (UNIMPLEMENTED) Generate shellcode to position esp at your location:                                  
        incode.exe --esp 45D308 --eip 457B00

    (UNIMPLEMENTED) Generate [positioning code]+[wrapped payload]:                                        
        incode.exe --code F3E9B800334A41 --esp 45D308 --eip 457B00

    (UNIMPLEMENTED) Generate [positioning code]+[wrapped far jump]:                                       
        incode.exe --jump 463303 --esp 45D308 --eip 457B00

    (UNIMPLEMENTED) Generate [positioning code]+[wrapped payload]+[wrapped far jump]:                       
        incode.exe --code \"0xF3 0xE9 0xB8 0x00 0x33 0x4A 0x41\" --jump 463303 --esp 45D308 --eip 457B00";

use InputError::*;
///a custom error type
#[derive(Debug)]
pub enum InputError {
    BadArg(String),
    MissingArg(String),
    NoArgs,
    ParseError(String, String),
    BadBytes(String),
    BadAddress(String),
    InvalidAddress(String),
}
//impl display formatting for error
impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BadArg(s)         => write!(f, "InputError::BadArg: Failed to parse arg: \"{}\". {}",s, SEE_HELP),
            Self::MissingArg(s)     => write!(f, "InputError::MissingArg: Must specify value after {}. {}",s, SEE_HELP),
            Self::NoArgs            => write!(f, "InputError::NoArgs: No arguments supplied! {}", SEE_HELP),
            Self::ParseError(s,e)   => write!(f, "InputError::ParseError: Attempt to parse bytes \"{}\" returned: {}. {}",s, e, SEE_HELP),
            Self::BadBytes(s)       => write!(f, "InputError::BadBytes: Parser found zero valid characters in given bytes \"{}\". {}",s,SEE_HELP),
            Self::BadAddress(s)     => write!(f, "InputError::BadAddress: Parser found zero valid characters in given address \"{}\". {}",s,SEE_HELP),
            Self::InvalidAddress(s) => write!(f, "InputError::InvalidAddress: Given address not within 32bit address range \"{}\". {}",s,SEE_HELP),
        }
    }
}
//impl error conversion for error
impl error::Error for InputError {}






///stores user input
#[derive(Debug)]
pub struct UserInput {
    pub code: Option<Vec<u8>>,
    pub esp:  Option<u32>,
    pub eip:  Option<u32>,
    pub jump: Option<u32>,
    pub help: bool,
}

impl UserInput {
    fn new_empty() -> Self {
        UserInput {
            code: None,
            esp: None,
            eip: None,
            jump: None,
            help: false,
        }
    }
}

///strips non hex characters from a string
fn strip_hex(input: &str) -> String {
    Regex::new(r"(0x)|[^A-Fa-f0-9]").unwrap()
        .replace_all(&input, "").to_string()
}

///parses input for its hex bytes
fn parse_bytes(input: &str) -> DynResult<Vec<u8>> {
    let mut parsed = strip_hex(input);
    if parsed.len() == 0 {dynerr!(BadBytes(input.to_string()))}
    if parsed.len()%2 != 0 {parsed.insert(0,'0')}
    let bytes = parsed.chars().chunks(2).into_iter()
        .map(|b| Ok(u8::from_str_radix(&b.collect::<String>(), 16)?))
        .collect::<DynResult<Vec<u8>>>()?;
    Ok(bytes)
}

///parses input for hex u32 value
fn parse_addr(input: &str) -> DynResult<u32> {
    let parsed = strip_hex(input);
    if parsed.len() == 0 {dynerr!(BadAddress(input.to_string()))}
    match u32::from_str_radix(&parsed, 16) {
        Ok(addr) => Ok(addr),
        Err(e) if *e.kind() == num::IntErrorKind::Overflow => {
            dynerr!(InvalidAddress(input.to_string()))
        },
        Err(e)   => {
            dynerr!(ParseError(input.to_string(), e.to_string()))
        }
    }
}

///parses user args
pub fn get_input() -> DynResult<UserInput> {
    let mut args = args().skip(1);
    let mut input = UserInput::new_empty();
    //no args
    if args.len() == 0 {dynerr!(NoArgs)}
    //one arg then try parsing it for bytes to wrap
    else if args.len() == 1 {
        match args.next().unwrap() {
            s if s==HELP => {input.help = true; return Ok(input)},
            s if s.starts_with("-") => dynerr!(BadArg(s)),
            byte_string  => match parse_bytes(&byte_string) {
                Ok(bytes) => input.code = Some(bytes),
                Err(e)    => dynmatch!(e,
                    type InputError {
                        arm BadBytes(_) => return Err(e),
                        _ => dynerr!(ParseError(byte_string.to_string(), e.to_string()))
                    },  _ => dynerr!(ParseError(byte_string.to_string(), e.to_string()))
                ),
            }
        }
        return Ok(input)
    }
    //else do a full arg match
    while let Some(arg) = args.next() {
        match &arg {
            s if s == CODE => {
                match args.next() {
                    Some(b) => input.code = Some(parse_bytes(&b)?),
                    None    => dynerr!(MissingArg(CODE.to_string()))
                }
            },
            s if s == ESP  => {
                match args.next() {
                    Some(b) => input.esp = Some(parse_addr(&b)?),
                    None    => dynerr!(MissingArg(ESP.to_string()))
                }
            },
            s if s == EIP  => {
                match args.next() {
                    Some(b) => input.eip = Some(parse_addr(&b)?),
                    None    => dynerr!(MissingArg(EIP.to_string()))
                }
            },
            s if s == JUMP => {
                match args.next() {
                    Some(b) => input.jump = Some(parse_addr(&b)?),
                    None    => dynerr!(MissingArg(JUMP.to_string()))
                }
            },
            s              => dynerr!(BadArg(s.to_string()))
        }
    }
    Ok(input)
}