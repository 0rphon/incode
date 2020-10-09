use super::InstructionSet;
use super::translate::{get32, get_full_bytes32, get_bytes32, strip_trailing_zero};

use std::u8;

const CONTACT_ME: &str = "Please contact the dev with a sample so this issue can be fixed!";

impl InstructionSet {
    /// generates InstructionSets for the given dword
    pub fn encode(&mut self, dword: &Vec<u8>, eax: &mut Vec<u8>) {
        assert_eq!(dword.len(),4);
        assert_eq!(eax.len(),4);
        let dword_addr = get32(dword);
        let bytes = strip_trailing_zero(dword);
        println!("{:02X?} {:02X?} {:02X?}\n",dword, eax, bytes);
        if dword == eax {self.push_eax(Some(dword_addr))}
        else if bytes.iter().any(|b| *b>0x7f||*b==0) || bytes.len()==3 {
            let mut output = Self::wrap(dword, eax);
            if output.instructions.len() >= 10 {
                println!("had to dec!");
                self.dec_eax();
                *eax = get_full_bytes32(get32(eax).overflowing_sub(1).0);
                output = Self::wrap(dword, eax);
            }
            self.extend(output);
            self.push_eax(Some(dword_addr));
            *eax = dword.clone();
        }
        else {
            match bytes.len() {
                4 => self.push32(dword_addr),
                2 => self.push16(dword_addr as u16),
                1 => self.push8(dword_addr as u8),
                _ => panic!("Couldnt push 0x{:08X}! {}",dword_addr,CONTACT_ME)  //if passed 00000000
            }
        }
    }

    fn wrap(dword: &Vec<u8>, eax: &Vec<u8>) -> Self {
        let target = get32(dword);
        let eax = get32(eax);
        let mut results = Vec::new();
        results.push(Self::check_add(target, eax));
        results.push(Self::check_sub(target, eax));
        results.push(Self::new_oned().combine(Self::check_add(target, 1)));
        results.push(Self::new_oned().combine(Self::check_sub(target, 1)));
        results.sort_by(|a,b| a.len.cmp(&b.len));
        results.remove(0)
    }

    ///creates a new InstructionSet instance with push 1, pop eax
    fn new_oned() -> Self {
        let mut new = Self::new();
        new.one_eax();
        new
    }

    /// generates InstructionSets to add eax to the target value
    fn check_add(tar: u32, eax: u32) -> Self {
        let dif = if tar > eax {tar-eax} else {(0xFFFFFFFF-eax)+tar};
        let mut instructions = Self::new();
        if dif == 0 {return instructions}
        for val in Self::gen_values(dif) {
            match get_bytes32(val) {
                b if b.len() == 4 => instructions.add_eax(val),
                b if b.len() == 2 => instructions.add_ax(val as u16),
                b if b.len() == 1 => instructions.add_al(val as u8),
                b => {panic!("Cant create add instruction for generated value {:08X?}! {}",b,CONTACT_ME)}
            }
        }
        instructions
    }

    /// generates InstructionSets to sub eax to the target value
    fn check_sub(tar: u32, eax: u32) -> Self {
        let dif = {
            if tar < eax {eax-tar} 
            else {
                0_u32.overflowing_sub(tar).0
                    .overflowing_add(eax).0
            }
        };
        let mut instructions = Self::new();
        if dif == 0 {return instructions}
        for val in Self::gen_values(dif) {
            match get_bytes32(val) {
                b if b.len() == 4 => instructions.sub_eax(val),
                b if b.len() == 2 => instructions.sub_ax(val as u16),
                b if b.len() == 1 => instructions.sub_al(val as u8),
                b => {panic!("Cant create sub instruction for generated value {:08X?}! {}",b,CONTACT_ME)}
            }
        }
        instructions
    }

    /// generates ascii safe values that add up to dif
    fn gen_values(dif: u32) -> Vec<u32> {
        let mut lines = vec!(get_full_bytes32(dif));
        while lines.iter().any(|line|strip_trailing_zero(line).iter().any(|b| *b>0x7f||*b==0)) && lines.len() < 10{
            Self::equalize(&mut lines);
        }
        for line in &lines {
            if strip_trailing_zero(line).iter().any(|b| *b>0x7f||*b==0) {
                println!("bad val {:02X?}",line)
            }
        }
        lines.iter().map(|v| get32(v)).collect()
    }

    /// recursively propagates the value at lines[index] until every line is ascii safe
    fn equalize(lines: &mut Vec<Vec<u8>>) {
        //remove ascii-safe zeros
        for line in 0..lines.len() {
            let stripped = strip_trailing_zero(&lines[line]);
            for (i, byte) in stripped.iter().enumerate() {
                //push overflow down. creates new line if needed
                if *byte > 0x7F {
                    if let None = lines.get(line+1) {lines.push(vec!(0;lines[line].len()))}
                    lines[line+1][i] = byte-0x7F;
                    lines[line][i] = 0x7f;
                }
                //steal from above to get rid of nulls. if first line then returns bad instruction
                if *byte == 0 {
                    if line == 0 {lines.extend(vec!(vec!(1;lines[line].len());10));return}         //NEED TO HANDLE THIS BETTER
                    lines[line-1][i]-=1;
                    lines[line][i]+=1;
                }
            }
            println!("{:02X?}->{:02X?}", stripped, lines[line]);
        }
    }
}



// Payload size: 17 bytes  jmp -492  e9 10 fe ff ff
// "\x6A\x01",              //push   0x01
// "\x58",                  //pop    eax
// "\x2D\x02\x6F\x6F\x6F",  //sub    eax, 0x6F6F6F02
// "\x50",                  //push   eax              (pushed 0x909090FF)
// "\x05\x7F\x7F\x6D\x6F",  //add    eax, 0x6F6D7F7F
// "\x04\x6B",              //add    al,  0x6B
// "\x50",                  //push   eax              (pushed 0xFFFE10E9)


//get rid of nop padding unless chunk len is 3
//only look at target.len() of eax for push
//equalize should take any size value
//match the final push to the instruction len