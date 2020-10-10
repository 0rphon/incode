mod instructions;
mod translate;
use translate::get_dwords;
use instructions::InstructionSet;

use std::io::{self, Write};


/// generates ascii wrapped x86 shellcode for a byte array
pub fn wrap(bytes: &Vec<u8>) -> InstructionSet {
    println!("Encoding {} bytes: {:02X?}", bytes.len(), bytes);
    let words = get_dwords(bytes);
    let mut output = InstructionSet::new();
    output.one_eax();
    let mut eax = 1;
    for (i, dword) in words.iter().enumerate() {
        print!("\rProg: {}/{}",i*4,bytes.len());
        io::stdout().flush().unwrap();
        output.encode(*dword, &mut eax);
    }
    print!("\r");
    output
}


///Generates positioning code
pub fn position(esp: u32, eip: u32) -> InstructionSet {
    println!("Generating positional code for 0x{:08X} -> 0x{:08X}", esp, eip);
    let mut output = InstructionSet::new();
    output.position(esp, eip, 0);
    output
}


pub fn position_wrap(bytes: &Vec<u8>, esp: u32, eip: u32) -> InstructionSet {
    println!("Encoding {} bytes: {:02X?}", bytes.len(), bytes);
    let words = get_dwords(bytes);
    let mut payload = InstructionSet::new();
    payload.one_eax();
    let mut eax = 1;
    for (i, dword) in words.iter().enumerate() {
        print!("\rProg: {}/{}",i*4,bytes.len());
        io::stdout().flush().unwrap();
        payload.encode(*dword, &mut eax);
    }

    let unpack_len = words.len()*4;
    let mut positional = InstructionSet::new();
    print!("\rGenerating positional code");
    positional.position(esp, eip, payload.len()+unpack_len);
    positional.extend(payload);
    println!("\rGenerated positional code for 0x{:08X} -> 0x{:08X}", esp, eip+(positional.len()+unpack_len) as u32);
    positional
}

// "\x54",                 //push    esp
// "\x58",                 //pop     eax
// "\x66\x05\x2B\x08",     //add     ax,0x82C
// "\x50",                 //push    eax
// "\x5c",                 //pop     esp


//some adds
//05 7f 7f 7f 7f          add    eax,0x7f7f7f7f
//66 05 7f 7f             add    ax,0x7f7f
//04 7f                   add    al,0x7f 
//some subs
//2d 7f 7f 7f 7f          sub    eax,0x7f7f7f7f
//66 2d 7f 7f             sub    ax,0x7f7f
//2c 7f                   sub    al,0x7f 



//jump target 19D588

//position && jump:
//  gen jump code
//  gen position code
//  set last len
//  while last len != cur len
//  re-gen jump code based on last len
//  regen position code based on last len