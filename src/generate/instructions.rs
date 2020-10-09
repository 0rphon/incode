mod wrapping;
mod positioning;

pub use wrapping::*;
pub use positioning::*;

use super::translate;

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
        code.extend(&translate::get_bytes_u32(val));
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
            translate::format_bytes(&self.bytes),
            self.mnemonic
        )
    }
}



#[derive(Debug, Clone)]
pub struct InstructionSet {
    /// a list of generated, ascii safe shellcode
    instructions: Vec<Instruction>,
    /// the length of the generated shellcode
    len: usize,
}


/// enables the ability to build an instruction set
impl InstructionSet {
    ///creates a new empty InstructionSet
    pub fn new() -> Self {
        Self {
            len: 0,
            instructions: Vec::new()
        }
    }

    /// adds one set of InstructionSet onto another
    pub fn extend(&mut self, other: Self) {
        self.len+=other.len;
        self.instructions.extend(other.instructions);
    }

    /// combines two sets of InstructionSet
    pub fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }

    /// adds an Instruction to the end of a InstructionSet instance
    pub fn push(&mut self, instruction: Instruction) {
        self.len+=instruction.len();
        self.instructions.push(instruction);
    }

    /// prints InstructionSet to screen
    pub fn display(&self) {
        println!("Payload size: {} bytes", self.len);
        for instruction in &self.instructions {
            println!("{}", instruction)
        }
    }

    /// creates the ascii equivalent of xor eax,eax
    pub fn zero_eax(&mut self) {
        self.push_8(1);
        self.pop_eax();
        self.dec_eax();
    }

    pub fn esp_to_eax(&mut self) {
        self.push_esp();
        self.pop_eax();
    }

    pub fn eax_to_esp(&mut self) {
        self.push_eax(None);
        self.pop_esp();
    }

    /// creates instruction to push eax to the stack
    /// has the option to take the current eax value to display in the mnemonic
    pub fn push_eax(&mut self, eax: Option<u32>) {
        self.push(Instruction::construct(OpCode::PushEax(eax)))
    }

    /// creates instruction to add value to eax
    pub fn add_eax(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::AddEax(val)))
    }

    /// creates instruction to subtract value to eax
    pub fn sub_eax(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::SubEax(val)))
    }

    /// creates instruction to push the given value to the stack
    /// DOES NOT DO WRAPPING!
    pub fn push_u32(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::Push32(val)))
    }

    pub fn push_8(&mut self, val: u8) {
        self.push(Instruction::construct(OpCode::Push8(val)))
    }

    pub fn push_esp(&mut self) {
        self.push(Instruction::construct(OpCode::PushEsp))
    }

    pub fn pop_esp(&mut self) {
        self.push(Instruction::construct(OpCode::PopEsp))
    }

    pub fn pop_eax(&mut self) {
        self.push(Instruction::construct(OpCode::PopEax))
    }

    pub fn dec_eax(&mut self) {
        self.push(Instruction::construct(OpCode::DecEax))
    }
}