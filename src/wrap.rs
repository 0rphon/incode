use super::translate::{get_u32, get_bytes_u32, format_bytes};

use std::u8;
use std::convert::TryInto;
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
struct Instruction {
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

    /// creates the ascii equivalent of xor eax,eax
    fn zero_eax() -> Vec<Self> {
        vec!(
            Self::construct(OpCode::Push8(1)),
            Self::construct(OpCode::PopEax),
            Self::construct(OpCode::DecEax),
        )
    }

    /// pushes eax to the stack
    /// has the option to take the current eax value to display in the mnemonic
    fn push_eax(eax: Option<u32>) -> Self {
        Self::construct(OpCode::PushEax(eax))
    }

    /// creates instruction to push the given value to the stack
    /// DOES NOT DO WRAPPING!
    fn push_u32(value: u32) -> Self {
        Self::construct(OpCode::Push32(value))
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

/// handles the wrapping of non-ascii instructions in ascii shellcode
#[derive(Debug, Clone)]
pub struct WrappedInstructions {
    /// a list of generated, ascii safe shellcode
    instructions: Vec<Instruction>,
    /// the length of the generated shellcode
    len: usize,
}
impl WrappedInstructions {
    /// creates a new WrappedInstructions instance from a list of Instructions
    fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            len: instructions.iter().map(|i| i.bytes.len()).sum::<usize>(),
            instructions,
        }
    }

    /// adds one set of WrappedInstructions onto another
    fn extend(&mut self, other: Self) {
        self.instructions.extend(other.instructions);
        self.len+=other.len;
    }

    /// combines two sets of WrappedInstructions
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }

    /// adds an Instruction to the end of a WrappedInstructions instance
    fn push(&mut self, instruction: Instruction) {
        self.len+=instruction.bytes.len();
        self.instructions.push(instruction);
    }

    /// generates WrappedInstructions for the given dword
    fn wrap(dword: [u8;4], eax: [u8;4]) -> Self {
        let target = get_u32(dword);
        let eax = get_u32(eax);
        let mut results = Vec::new();
        results.push(Self::check_add(target, eax));
        results.push(Self::check_sub(target, eax));
        results.push(Self::new(Instruction::zero_eax())
            .combine(Self::check_add(target, 0)));
        results.push(Self::new(Instruction::zero_eax())
            .combine(Self::check_sub(target, 0)));
        results.sort_by(|a,b| a.len.cmp(&b.len));
        let best = results.remove(0);
        if best.instructions.len() > 10 {
            panic!("Couldn't encode payload! Please contact the dev with a sample so this issue can be fixed!")
        }
        best
    }

    /// generates WrappedInstructions to add eax to the target value
    fn check_add(tar: u32, eax: u32) -> Self {
        let dif = if tar > eax {tar-eax} else {(0xFFFFFFFF-eax)+tar};
        let instructions = Self::gen_values(dif).into_iter().map(|val|
            Instruction::construct(OpCode::AddEax(val))
        ).collect::<Vec<Instruction>>();
        Self::new(instructions)
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
        let instructions = Self::gen_values(dif).into_iter().map(|val|
            Instruction::construct(OpCode::SubEax(val))
        ).collect::<Vec<Instruction>>();
        Self::new(instructions)
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

    /// prints WrappedInstructions to screen
    pub fn display(&self) {
        println!("Payload size: {} bytes", self.len);
        for instruction in &self.instructions {
            println!("{}", instruction)
        }
    }
}

/// gets dwords out of a byte array
pub fn get_dwords(mut bytes: Vec<u8>) -> Vec<[u8;4]> {
    let pad = bytes.len()%4;
    if pad != 0 {bytes.extend(vec!(0x90;4-pad))}
    let mut words = bytes.chunks_exact(4).map(|b| b.try_into().unwrap())
        .collect::<Vec<[u8;4]>>();
    words.reverse();
    words
}

/// generates ascii wrapped x86 shellcode for a byte array
pub fn wrap(bytes: &Vec<u8>) -> WrappedInstructions {
    let words = get_dwords(bytes.clone());
    let mut output = WrappedInstructions::new(Instruction::zero_eax());
    let mut eax = [0_u8,0,0,0];
    for word in &words {
        if *word == eax {output.push(Instruction::push_eax(Some(get_u32(eax))))}
        else if word.iter().any(|b| *b>0x7F||*b==0) {
            let action = WrappedInstructions::wrap(*word, eax);
            output.extend(action);
            eax = *word;
            output.push(Instruction::push_eax(Some(get_u32(eax))));
        } else {
            output.push(Instruction::push_u32(get_u32(*word)));
        }
    }
    output
}