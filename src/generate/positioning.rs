use super::instructions::{Instruction, InstructionSet};
use incode_derive::Instructions;

#[derive(Instructions)]
pub struct PositioningInstructions {
    instructions: Vec<Instruction>,
    len: usize
}

impl PositioningInstructions {
    fn generate_jump(esp: u32, eax: u32) {

    }
}


// "\x54",                 //push    esp
// "\x58",                 //pop     eax
// "\x66\x05\x2B\x08",     //add     ax,0x82C
// "\x50",                 //push    eax
// "\x5c",                 //pop     esp