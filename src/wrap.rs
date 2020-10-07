use super::translate::{get_u32, get_bytes_u32, format_bytes};

use std::u8;
use std::convert::TryInto;

const PUSH_ONE: [u8;2]      = [0x6A, 0x01];
const PUSH_ONE_INS: &str    = "push   0x1";
const POP_EAX: [u8;1]       = [0x58];
const POP_EAX_INS: &str     = "pop    eax";
const XOR_AL: [u8;2]        = [0x34, 0x01];
const XOR_AL_INS: &str      = "xor    al,0x1";
const SUB: u8               = 0x2D;
const SUB_INS: &str         = "sub";
const ADD: u8               = 0x05;
const ADD_INS: &str         = "add";
const PUSH_EAX: [u8;1]      = [0x50];
const PUSH_EAX_INS: &str    = "push   eax";
const PUSH_VAL: u8       = 0x68;
const PUSH_VAL_INS: &str = "push";


#[derive(Debug, PartialEq, Clone, Copy)]
enum EncodeStyle {
    Sub,
    Add,
    XorSub,
    XorAdd,
}
#[derive(Debug, Clone)]
struct EncodeData {
    style:  EncodeStyle,
    values: Vec<u32>
}
impl EncodeData {
    ///checks which encoding style would be shortest
    fn check_encode(bytes: [u8;4], reg: [u8;4]) -> Self {
        let target = get_u32(bytes);
        let reg = get_u32(reg);
        let mut results = Vec::new();
        results.push(Self::check_add(target, reg));
        results.push(Self::check_sub(target, reg));
        results.push(Self::check_xor_add(target));
        results.push(Self::check_xor_sub(target));
        results.sort_by(|a,b| a.values.len().cmp(&b.values.len()));
        results[0].clone()
    }
    fn check_add(tar: u32, reg: u32) -> Self {
        let dif = if tar > reg {tar-reg} else {(0xFFFFFFFF-reg)+tar};
        Self {
            style: EncodeStyle::Add,
            values: Self::get_data(dif),
        }
    }
    fn check_xor_add(tar: u32) -> Self {
        Self {
            style: EncodeStyle::XorAdd,
            values: Self::get_data(tar),
        }
    }
    fn check_sub(tar: u32, reg: u32) -> Self {
        let dif = {
            if tar < reg {reg-tar} 
            else {
                0xFFFFFFFF_u32
                    .overflowing_sub(tar).0
                    .overflowing_add(1).0
                    .overflowing_add(reg).0
            }
        };
        Self {
            style: EncodeStyle::Sub,
            values: Self::get_data(dif),
        }
    }
    fn check_xor_sub(tar: u32) -> Self {
        let dif = (0xFFFFFFFF-tar).overflowing_add(1).0;
        Self {
            style: EncodeStyle::XorSub,
            values: Self::get_data(dif),
        }
    }

    fn get_data(dif: u32) -> Vec<u32> {
        if get_bytes_u32(dif)[0] == 0 {return vec!(0;10)}
        let (times, rem) = (dif/0x7F7F7F7F,dif%0x7F7F7F7F);
        let mut values = vec!(0x7F7F7F7F;times as usize);
        if rem!=0 {
            values.push(rem);
            //if remainder valid then its all good
            if get_bytes_u32(rem).iter().all(|b| *b<=0x7F&&*b!=0) {
                return values
            }
            //if only one val and val has null byte then bad
            if values.len()==1
            && get_bytes_u32(rem).iter().any(|b| *b==0) {
                return vec!(0;10)
            }
            //if remainder not valid start equalizing
            values = Self::equalize(values);
        }
        //values
        values
    }

    ///gets rid of zeros and invalid vals. adds lines as needed
    fn equalize(values: Vec<u32>) -> Vec<u32> {
        let mut lines = values.iter().map(|v| get_bytes_u32(*v)).collect::<Vec<[u8;4]>>();
        loop {
            for line in 0..lines.len() {
                for byte in 0..lines[line].len() {
                    //push overflow down
                    if lines[line][byte] > 0x7F {
                        if let None = lines.get(line+1) {lines.push([0;4])}
                        lines[line+1][byte] = lines[line][byte]-0x7F;
                        lines[line][byte] = 0x7f;
                    }
                    //steal from above to get rid of nulls
                    if lines[line][byte] == 0 {
                        lines[line-1][byte]-=1;
                        lines[line][byte]+=1;
                    }
                }
            }
            if lines.iter().all(|l|
                l.iter().all(|b| *b<=0x7F&&*b!=0)
            ) {break}
        }
        lines.iter().map(|v| get_u32(*v)).collect()
    }
}



//just creates an xor instruction
fn xor() -> Vec<Vec<u8>> {
    vec!(
        PUSH_ONE.to_vec(),
        POP_EAX.to_vec(),
        XOR_AL.to_vec()
    )
}

//creates the post instruction
fn post() -> Vec<u8> {
    PUSH_EAX.to_vec()
}

fn encode_action(op: u8, lines: Vec<u32>) -> Vec<Vec<u8>> {
    let mut instructions = Vec::new();
    for line in lines {
        let mut bytes = get_bytes_u32(line).to_vec();
        bytes.insert(0, op);
        instructions.push(bytes);
    }
    instructions
}


///displays the corresponding opcodes for a byte array
pub fn display_instructions(output: (Vec<Vec<u8>>, Vec<[u8;4]>)) {
    let (instructions, words) = output;
    let mut words = words.into_iter();
    for ins in instructions {
        println!("{:<24} //{}",
            format_bytes(&ins),
            match ins {
                i if i == PUSH_ONE       => PUSH_ONE_INS.to_string(),
                i if i == POP_EAX        => POP_EAX_INS.to_string(),
                i if i == XOR_AL         => XOR_AL_INS.to_string(),
                i if i == PUSH_EAX       => format!("{:<20}{:02X?}",
                    PUSH_EAX_INS.to_string(),
                    words.next().unwrap()
                ),
                i if i[0] == PUSH_VAL    => format!("{:<6} 0x{:08X}",
                    PUSH_VAL_INS,
                    get_u32([i[1], i[2], i[3], i[4]])
                ),
                i if i[0] == SUB => format!("{:<6} 0x{:08X}",
                    SUB_INS,
                    get_u32([i[1], i[2], i[3], i[4]])
                ),
                i if i[0] == ADD => format!("{:<6} 0x{:08X}",
                    ADD_INS,
                    get_u32([i[1], i[2], i[3], i[4]])
                ),
                _ => panic!("Error while formatting. Contact the Dev.")
            }
        );
    }
}




///gets dwords out of a byte array
pub fn get_dwords(bytes: &Vec<u8>) -> Vec<[u8;4]> {
    let mut words = bytes.chunks_exact(4).map(|b| b.try_into().unwrap())
        .collect::<Vec<[u8;4]>>();
    words.reverse();
    words
}





pub fn wrap(bytes: &Vec<u8>) -> (Vec<Vec<u8>>,  Vec<[u8;4]>) {
    let words = get_dwords(bytes);
    let mut output = vec!();
    output.extend(xor());
    let mut reg = [0x00_u8,0,0,0x00];
    for word in &words {
        if *word == reg {output.push(post())}
        else if word.iter().any(|b| *b>0x7F||*b==0) {
            use EncodeStyle::*;
            let action = EncodeData::check_encode(*word, reg);
            match action {
                e if e.style == Add    => {
                    output.extend(encode_action(ADD, e.values));
                },
                e if e.style == Sub    => {
                    output.extend(encode_action(SUB, e.values));
                },
                e if e.style == XorAdd => {
                    output.extend(xor());
                    output.extend(encode_action(ADD, e.values));
                },
                e if e.style == XorSub => {
                    output.extend(xor());
                    output.extend(encode_action(SUB, e.values));
                },
                _ => {panic!("Error while matching. Contact the dev")}
            }
            output.push(post());
            reg = *word;
        } else {
            let mut ins = word.to_vec();
            ins.insert(0, PUSH_VAL);
            output.push(ins)
        }
    }
    let xor_len = xor().len();
    if output[0..xor_len] == xor()[..]
    && output[xor_len..xor_len+xor_len] == xor()[..] {
        println!("Output had duplicate values. Its been adjusted, but please contact the dev");
        output.drain(0..xor_len);
    }
    (output, words)
}