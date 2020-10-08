use super::translate::{get_bytes_u32, format_bytes};

use std::fmt;

/// handles translating values to instructions
#[derive(Debug, PartialEq, Clone, Copy)]
enum OpCode {
    Push8(u8),
    Push32(u32),
    PopEax,
    DecEax,
    SubEax(u32),
    AddEax(u32),
    PushEax(Option<u32>),
    PushEsp,
    PopEsp,
}
impl OpCode {
    /// generates the bytes and mnemonic for an instruction
    fn gen_instruction(&self) -> (Vec<u8>, String) {
        use OpCode::*;
        match self {
            Push8(v)   => (vec!(0x6A,*v),         format!("push   0x{:02X}",v)), 
            Push32(v)  => (Self::create(0x68,*v), format!("push   0x{:08X}",v)),
            PopEax     => (vec!(0x58),            format!("pop    eax")),
            DecEax     => (vec!(0x48),            format!("dec    eax")),
            SubEax(v)  => (Self::create(0x2D,*v), format!("sub    eax, 0x{:08X}",v)),
            AddEax(v)  => (Self::create(0x05,*v), format!("add    eax, 0x{:08X}",v)),
            PushEax(r) => (vec!(0x50),            format!("push   eax{}", 
                if let Some(v) = r {format!("              (pushed 0x{:08X})",v)} 
                else {String::new()})
            ),
            PushEsp    => (vec!(0x54),            format!("push   esp")),
            PopEsp     => (vec!(0x5C),            format!("pop    esp")),
        }
    }
    
    /// combines the bytes for an instruction
    fn create(op: u8, val: u32) -> Vec<u8> {
        let mut code = vec!(op);
        code.extend(&get_bytes_u32(val));
        code
    }
}

/// handles the creation of instructions
#[derive(Debug, Clone)]
pub struct Instruction {
    bytes: Vec<u8>,
    mnemonic: String,
}
impl Instruction {
    /// constructs an instruction for the given OpCode(val)
    fn construct(op: OpCode) -> Self {
        let (bytes, mnemonic) = op.gen_instruction();
        Self {
            bytes,
            mnemonic,
        }
    }
    
    //gets the len of the instruction
    fn len(&self) -> usize {
        self.bytes.len()
    } 
}
/// allows instructions to be easily displayed
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
            "{:<24} //{}",
            format_bytes(&self.bytes),
            self.mnemonic
        )
    }
}



/// enables the ability to build an instruction set
pub trait InstructionSet {
    fn new() -> Box<Self>;
    fn len(&self) -> usize;
    fn len_mut(&mut self) -> &mut usize;
    fn instructions(self) -> Vec<Instruction>;
    fn instructions_ref(&self) -> &Vec<Instruction>;
    fn instructions_mut(&mut self) -> &mut Vec<Instruction>;

    /// adds one set of InstructionSet onto another
    fn extend(&mut self, other: Self) {
        *self.len_mut()+=other.len();
        self.instructions_mut().extend(other.instructions());
    }

    /// combines two sets of InstructionSet
    fn combine(mut self, other: Self) -> Self where Self: std::marker::Sized {
        self.extend(other);
        self
    }

    /// adds an Instruction to the end of a InstructionSet instance
    fn push(&mut self, instruction: Instruction) {
        *self.len_mut()+=instruction.len();
        self.instructions_mut().push(instruction);
    }

    /// prints InstructionSet to screen
    fn display(&self) {
        println!("Payload size: {} bytes", self.len());
        for instruction in self.instructions_ref() {
            println!("{}", instruction)
        }
    }

    /// creates the ascii equivalent of xor eax,eax
    fn zero_eax(&mut self) {
        self.push_8(1);
        self.pop_eax();
        self.dec_eax();
    }

    fn esp_to_eax(&mut self) {
        self.push_esp();
        self.pop_eax();
    }

    fn eax_to_esp(&mut self) {
        self.push_eax(None);
        self.pop_esp();
    }

    /// creates instruction to push eax to the stack
    /// has the option to take the current eax value to display in the mnemonic
    fn push_eax(&mut self, eax: Option<u32>) {
        self.push(Instruction::construct(OpCode::PushEax(eax)))
    }

    /// creates instruction to add value to eax
    fn add_eax(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::AddEax(val)))
    }

    /// creates instruction to subtract value to eax
    fn sub_eax(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::SubEax(val)))
    }

    /// creates instruction to push the given value to the stack
    /// DOES NOT DO WRAPPING!
    fn push_u32(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::Push32(val)))
    }

    fn push_8(&mut self, val: u8) {
        self.push(Instruction::construct(OpCode::Push8(val)))
    }

    fn push_esp(&mut self) {
        self.push(Instruction::construct(OpCode::PushEsp))
    }

    fn pop_esp(&mut self) {
        self.push(Instruction::construct(OpCode::PopEsp))
    }

    fn pop_eax(&mut self) {
        self.push(Instruction::construct(OpCode::PopEax))
    }

    fn dec_eax(&mut self) {
        self.push(Instruction::construct(OpCode::DecEax))
    }
}