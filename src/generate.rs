mod translate;
mod instructions;
mod wrapping;
mod positioning;
use translate::{get_u32, get_dwords};
pub use instructions::InstructionSet;
use wrapping::WrappedInstructions;


/// generates ascii wrapped x86 shellcode for a byte array
pub fn wrap(bytes: &Vec<u8>) -> WrappedInstructions {
    let words = get_dwords(bytes.clone());
    let mut output = *WrappedInstructions::new();
    output.zero_eax();
    let mut eax = [0_u8,0,0,0];
    for word in &words {
        if *word == eax {output.push_eax(Some(get_u32(eax)))}
        else if word.iter().any(|b| *b>0x7F||*b==0) {
            let action = WrappedInstructions::encode(*word, eax);
            output.extend(action);
            eax = *word;
            output.push_eax(Some(get_u32(eax)));
        } else {
            output.push_u32(get_u32(*word));
        }
    }
    output
}


///Generates positioning code
pub fn position(esp: u32, eip: u32) {
    let mut output = positioning::PositioningInstructions::new();
    output.esp_to_eax();
    let dif = {
        if esp > eip {esp-eip}
        else {eip-esp}
    };
    output.eax_to_esp();
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