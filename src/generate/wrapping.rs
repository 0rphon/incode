use incode_derive::Instructions;
use super::instructions::{Instruction, InstructionSet};
use super::translate::{get_bytes_u32, get_u32};

use std::u8;
/// handles the wrapping of non-ascii instructions in ascii shellcode
#[derive(Instructions, Debug, Clone)]
pub struct WrappedInstructions {
    /// a list of generated, ascii safe shellcode
    instructions: Vec<Instruction>,
    /// the length of the generated shellcode
    len: usize,
}
impl WrappedInstructions {
    /// generates WrappedInstructions for the given dword
    pub fn encode(dword: [u8;4], eax: [u8;4]) -> Self {
        let target = get_u32(dword);
        let eax = get_u32(eax);
        let mut results = Vec::new();
        results.push(Self::check_add(target, eax));
        results.push(Self::check_sub(target, eax));
        results.push(Self::new_zeroed().combine(Self::check_add(target, 0)));
        results.push(Self::new_zeroed().combine(Self::check_sub(target, 0)));
        results.sort_by(|a,b| a.len.cmp(&b.len));
        let best = results.remove(0);
        if best.instructions.len() > 10 {
            panic!("Couldn't encode payload! Please contact the dev with a sample so this issue can be fixed!")
        }
        best
    }

    ///creates a new WrappedInstruction instance with an xor eax, eax inside it
    fn new_zeroed() -> Self {
        let mut new = *Self::new();
        new.zero_eax();
        new
    }

    /// generates WrappedInstructions to add eax to the target value
    fn check_add(tar: u32, eax: u32) -> Self {
        let dif = if tar > eax {tar-eax} else {(0xFFFFFFFF-eax)+tar};
        let mut instructions = *Self::new();
        for val in Self::gen_values(dif){instructions.add_eax(val)}
        instructions
    }

    /// generates WrappedInstructions to sub eax to the target value
    fn check_sub(tar: u32, eax: u32) -> Self {
        let dif = {
            if tar < eax {eax-tar} 
            else {
                0_u32.overflowing_sub(tar).0
                    .overflowing_add(eax).0
            }
        };
        let mut instructions = *Self::new();
        for val in Self::gen_values(dif){instructions.sub_eax(val)}
        instructions
    }

    /// generates ascii safe values that add up to dif
    /// returns vec of 10 zeros if failed
    fn gen_values(dif: u32) -> Vec<u32> {
        let mut lines = vec!(get_bytes_u32(dif));
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
                        if line == 0 {return vec!(0;10)}
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