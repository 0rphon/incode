mod wrapping;

pub use wrapping::*;

use super::translate;

use std::fmt;
pub const CONTACT_ME: &str = "Please contact the dev with a sample so this issue can be fixed!";
pub const JUMP: u8 = 0xE9;

//some adds
//05 7f 7f 7f 7f          add    eax,0x7f7f7f7f
//66 05 7f 7f             add    ax,0x7f7f
//04 7f                   add    al,0x7f 
//some subs
//2d 7f 7f 7f 7f          sub    eax,0x7f7f7f7f
//66 2d 7f 7f             sub    ax,0x7f7f
//2c 7f                   sub    al,0x7f 

/// handles translating values to instructions
#[derive(Debug, PartialEq, Clone, Copy)]
enum OpCode {
    Push8(u8),
    #[allow(unused)]
    Push16(u16),
    Push32(u32),
    PopEax,
    #[allow(unused)]
    DecEax,
    AddEax(u32),
    AddAx(u16),
    AddAl(u8),
    SubEax(u32),
    SubAx(u16),
    SubAl(u8),
    PushEax(Option<u32>),
    #[allow(unused)]
    PushAx(Option<u16>),
    PushEsp,
    PopEsp,
}
impl OpCode {
    /// generates the bytes and mnemonic for an instruction
    fn gen_instruction(&self) -> (Vec<u8>, String) {
        use OpCode::*;
        match self {
            Push8(v)   => (vec!(0x6A,*v),                   format!("push   0x{:02X}",v)), 
            Push16(v)  => (Self::create16(0x66, 68, *v),    format!("push   0x{:04X}",v)),
            Push32(v)  => (Self::create32(0x68,*v),         format!("push   0x{:08X}",v)),
            PopEax     => (vec!(0x58),                      format!("pop    eax")),
            DecEax     => (vec!(0x48),                      format!("dec    eax")),
            AddEax(v)  => (Self::create32(0x05,*v),         format!("add    eax, 0x{:08X}",v)),
            AddAx(v)   => (Self::create16(0x66, 0x05, *v),  format!("add    ax,  0x{:04X}",v)),
            AddAl(v)   => (vec!(0x04, *v),                  format!("add    al,  0x{:02X}",v)),
            SubEax(v)  => (Self::create32(0x2D,*v),         format!("sub    eax, 0x{:08X}",v)),
            SubAx(v)   => (Self::create16(0x66, 0x2D, *v),  format!("sub    ax,  0x{:04X}",v)),
            SubAl(v)   => (vec!(0x00, *v),                  format!("sub    al,  0x{:02X}",v)),
            PushEax(r) => (vec!(0x50),                      format!("push   eax{}", 
                if let Some(v) = r { format!("              (pushed 0x{:08X})",v)} else {String::new()})
            ),
            PushAx(r)   => (vec!(0x66,0x50),                format!("push   ax{}", 
                if let Some(v) = r { format!("              (pushed 0x{:04X})",v)} else {String::new()})
            ),
            PushEsp    => (vec!(0x54),                      format!("push   esp")),
            PopEsp     => (vec!(0x5C),                      format!("pop    esp")),
        }
    }
    
    /// combines the bytes for an instruction
    fn create32(op: u8, val: u32) -> Vec<u8> {
        let mut code = vec!(op);
        code.extend(&translate::get_full_bytes32(val));
        code
    }

    fn create16(op: u8, op2: u8, val: u16) -> Vec<u8> {
        let mut code = vec!(op, op2);
        code.extend(&translate::get_full_bytes16(val));
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

    pub fn len(&self) -> usize {
        self.len
    }

    /// prints InstructionSet to screen
    pub fn display(&self) {
        println!("Payload size: {} bytes", self.len);
        for instruction in &self.instructions {
            println!("{}", instruction)
        }
    }

    /// creates the ascii equivalent of mov eax, 1
    #[allow(unused)]
    pub fn zero_eax(&mut self) {
        self.push8(1);
        self.pop_eax();
        self.dec_eax();
    }

    /// creates the ascii equivalent of mov eax, 1
    pub fn one_eax(&mut self) {
        self.push8(1);
        self.pop_eax();
    }

    /// creates the unwrapped byte sequence for a far jump
    pub fn create_far_jump(jmp: u32, eip:u32) -> Vec<u8> {
        let offset = jmp.overflowing_sub(eip).0;
        let mut code = translate::get_full_bytes32(offset);
        code.insert(0, JUMP);
        code
    }

    /// creates instruction to push eax to the stack
    /// has the option to take the current eax value to display in the mnemonic
    pub fn push_eax(&mut self, eax: Option<u32>) {
        self.push(Instruction::construct(OpCode::PushEax(eax)))
    }

    #[allow(unused)]
    pub fn push_ax(&mut self, eax: Option<u16>) {
        self.push(Instruction::construct(OpCode::PushAx(eax)))
    }

    /// creates instruction to add value to eax
    pub fn add_eax(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::AddEax(val)))
    }

    /// creates instruction to add value to ax
    pub fn add_ax(&mut self, val: u16) {
        self.push(Instruction::construct(OpCode::AddAx(val)))
    }

    /// creates instruction to add value to al
    pub fn add_al(&mut self, val: u8) {
        self.push(Instruction::construct(OpCode::AddAl(val)))
    }

    /// creates instruction to subtract value to eax
    pub fn sub_eax(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::SubEax(val)))
    }

    /// creates instruction to add value to ax
    pub fn sub_ax(&mut self, val: u16) {
        self.push(Instruction::construct(OpCode::SubAx(val)))
    }

    /// creates instruction to add value to al
    pub fn sub_al(&mut self, val: u8) {
        self.push(Instruction::construct(OpCode::SubAl(val)))
    }

    /// creates instruction to push the given value to the stack
    /// DOES NOT DO WRAPPING!
    pub fn push32(&mut self, val: u32) {
        self.push(Instruction::construct(OpCode::Push32(val)))
    }

    #[allow(unused)]
    pub fn push16(&mut self, val: u16) {
        self.push(Instruction::construct(OpCode::Push16(val)))
    }

    pub fn push8(&mut self, val: u8) {
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