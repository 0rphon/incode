use dynerr::*;

use std::{fmt, error};
use std::env::args;

use itertools::Itertools;
use regex::Regex;


const CODE: &str = "--code";
const ESP: &str  = "--esp";
const EIP: &str  = "--eip";
const JUMP: &str = "--jump";
const HELP: &str = "--help";
const SEE_HELP: &str = "Use --help for usage.";
const HELP_MESSAGE: &str = "InCode is an ASCII encoder for x86 shellcode. It has tools to handle wrapping, positioning, and jumping.
This is a tool I wrote for personal security research. I obviously accept no responsibility for how other 
people use it. Stay safe and stop breaking the law kiddos.

Usage:
    --help:             Show this screen and exit.

    --code [bytes]:     Encode an instruction in x86 ascii shellcode that gets decoded to [esp] at runtime.

    --esp [addr] --eip [addr]:
                        What the addresses of ESP and EIP will be at the start of this payload. While these 
                        values wont stay the same between runs, their difference will. Use this if you want 
                        the payload to handle unpacking and jumping on its own.

    --jump [addr]:      Generate an encoded far jump instruction from your current location. Requires --esp 
                        and --eip to be set. It will handle esp positioning for you.
    
    Examples:
        TODO";

use InputError::*;
///a custom error type
#[derive(Debug)]
pub enum InputError {
    BadArg(String),
    MissingArg(String),
    NoArgs,
    ParseError(String, String),
    HelpMessage,
}
//impl display formatting for error
impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BadArg(s)       => write!(f, "BadArg: Failed to parse arg: {}. {}",s, SEE_HELP),
            Self::MissingArg(s)   => write!(f, "MissingArg: Must specify value after {}. {}",s, SEE_HELP),
            Self::NoArgs          => write!(f, "NoArgs: No arguments supplied! {}", SEE_HELP),
            Self::ParseError(s,e) => write!(f, "ParseError: Attempt to parse bytes {} returned {}. {}",s, e, SEE_HELP),
            Self::HelpMessage     => write!(f, "{}", HELP_MESSAGE),
        }
    }
}
//impl error conversion for error
impl error::Error for InputError {}






pub struct UserInput {
    pub code: Option<Vec<u8>>,
    pub esp: Option<u32>,
    pub eip: Option<u32>,
    pub jump: Option<u32>,
}

impl UserInput {
    fn new_empty() -> Self {
        UserInput {
            code: None,
            esp: None,
            eip: None,
            jump: None,
        }
    }
}

fn strip_hex(input: &str) -> String {
    Regex::new(r"(0x)|[^A-Fa-f0-9]").unwrap()
        .replace_all(&input, "").to_string()
}

///parses input for its hex values
fn parse_bytes(input: &str) -> DynResult<Vec<u8>> {
    let mut parsed = strip_hex(input);
    if parsed.len()%2 != 0 {parsed.insert(0,'0')}
    let mut bytes = parsed.chars().chunks(2).into_iter()
        .map(|b| Ok(u8::from_str_radix(&b.collect::<String>(), 16)?))
        .collect::<DynResult<Vec<u8>>>()?;
    let pad = bytes.len()%4;
    if pad != 0 {bytes.extend(vec!(0x90;4-pad))}
    Ok(bytes)
}

fn parse_addr(input: &str) -> DynResult<u32> {
    let parsed = strip_hex(input);
    Ok(u32::from_str_radix(&parsed, 16)?)
}

pub fn get_input() -> DynResult<UserInput> {
    let mut args = args().skip(1);
    let mut input = UserInput::new_empty();
    //no args
    if args.len() == 0 {dynerr!(NoArgs)}
    //one arg then try parsing it for bytes to wrap
    else if args.len() == 1 {
        match args.next().unwrap() {
            s if s==HELP => dynerr!(HelpMessage),
            byte_string  => {
                match parse_bytes(&byte_string) {
                    Ok(bytes) => input.code = Some(bytes),
                    Err(e)    => dynerr!(ParseError(byte_string.to_string(), e.to_string())),
                };
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

//prog.exe --wrap "\x33\x00\x90\x01\xFF" --esp 3b8eff20 --eip 3b8ef030                  ENCODE INSTRUCTIONS
//prog.exe --jump 3b8ef330 --esp 3b8eff20 --eip 3b8ef030                                JUMP
//prog.exe --wrap "\x33\x00\x90\x01\xFF"                                                JUST CREATE THE WRAP CODE
//prog.exe --esp 3b8eff20 --eip 3b8ef030

//--wrap (bytes)
//--esp (addr)
//--eip (addr)
//--jump (addr)

//nothing specified just bytes then wrap

//--esp --eip           adjust
//--wrap --esp --eip    adjust then wrap
//--wrap                wrap code
//--jump                wrap jump
//--jump --esp --eip    adjust then wrap jump