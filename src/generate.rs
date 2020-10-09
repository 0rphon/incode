mod instructions;
mod translate;
use translate::{get_dwords, get_full_bytes32};
use instructions::InstructionSet;


/// generates ascii wrapped x86 shellcode for a byte array
pub fn wrap(bytes: &Vec<u8>) -> InstructionSet {
    let words = get_dwords(bytes);
    let mut output = InstructionSet::new();
    output.one_eax();
    let mut eax = get_full_bytes32(1);
    for dword in words {
        output.encode(&dword, &mut eax);
    }
    output
}


///Generates positioning code
pub fn position(esp: u32, eip: u32) -> InstructionSet {
    let mut output = InstructionSet::new();
    output.esp_to_eax();
    output.generate_positional(esp, eip);
    output.eax_to_esp();
    output
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