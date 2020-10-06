mod wrap;
mod translate;
use wrap::{wrap, display_instructions};
use translate::{parse_bytes, get_dwords};

use std::env::args;

fn main() {
    let input = {
        if let Some(arg) = args().nth(1) {arg}
        else {panic!("No arg passed!")}
    };
    let bytes = parse_bytes(&input).unwrap();
    println!("Parsed {} bytes: {:02X?}", bytes.len(), bytes);
    let words = get_dwords(&bytes);

    let output = wrap(words);
    println!("Payload size: {} bytes", output.iter().flatten().count());
    display_instructions(output);
}


//tools
//25 7f 7f 7f 7f          and    eax,0x7f7f7f7f
//35 7f 7f 7f 7f          xor    eax,0x7f7f7f7f
//05 7f 7f 7f 7f          add    eax,0x7f7f7f7f
//smallest way to zero reg
//6a 01                   push   0x1
//58                      pop    eax
//34 01                   xor    al,0x1


//ADJUSTING
//specify esp location
//specify your location
//generate esp code
//generate unpack
//regenerate esp code
//make sure new esp code == last esp code

//esp code should be able to go all the way up to FFFF FFFF

//if jump mode specified and jump address given, then at the end adjust the jmp for new bytes, store the byte len, redo calcs, then check if equal



//prog.exe --wrap "\x33\x00\x90\x01\xFF" --esp 3b8eff20 --eip 3b8ef030                 ENCODE INSTRUCTIONS
//prog.exe --jump 3b8ef330 --esp 3b8eff20 --eip 3b8ef030                        JUMP
//prog.exe --wrap "\x33\x00\x90\x01\xFF"                                        JUST CREATE THE WRAP CODE
//prog.exe --esp 3b8eff20 --eip 3b8ef030