#![feature(int_error_matching)]

mod wrap;
mod translate;
mod input;
use wrap::{wrap, display_instructions};
use dynerr::*;

use std::process::exit;

const WELCOME_MESSAGE: &str = "InCode: for encoding your code in code to run in other peoples code!\nCreated by 0rphon\n";


fn main() {
    println!("{}",WELCOME_MESSAGE);

    let input = input::get_input().unwrap_or_else(|e| {
        println!("{}",e);
        dynmatch!(e,
            type input::InputError {
                arm input::InputError::HelpMessage => exit(0),
                _   => exit(1)
            },  _   => exit(2)
        )
    });
    let (esp, eip, jump, code) = (
        input.esp.is_some(),
        input.eip.is_some(),
        input.jump.is_some(),
        input.code.is_some()
    );

    if esp != eip {
        println!("ArgsError: You must set both --esp and --eip. {}", 
            input::SEE_HELP);
    }
    else if jump && (!esp||!eip) {
        println!("ArgsError: You must set both --esp and --eip to use --jump. {}", 
            input::SEE_HELP);
    }
    else if esp && eip && code && jump {do_position_code_jump(input)}
    else if esp && eip && jump {do_position_jump(input)}
    else if esp && eip && code {do_position_code(input)}
    else if esp && eip {do_position(input)}
    else if code {do_code(input)}
    else {println!("ArgsError: failed to match arguments. Please notify the dev. {}",
        input::SEE_HELP);
    }
}

fn do_code(input: input::UserInput) {
    let bytes = input.code.unwrap();
    println!("Encoding {} bytes: {:02X?}", bytes.len(), bytes);
    let output = wrap(&bytes);
    println!("Payload size: {} bytes", output.0.iter().flatten().count());
    display_instructions(output);
}
fn do_position(input: input::UserInput) {println!("Not Implemented yet. Sorry! {:02X?}", input)}
fn do_position_code(input: input::UserInput) {println!("Not Implemented yet. Sorry! {:02X?}", input)}
fn do_position_jump(input: input::UserInput) {println!("Not Implemented yet. Sorry! {:02X?}", input)}
fn do_position_code_jump(input: input::UserInput) {println!("Not Implemented yet. Sorry! {:02X?}", input)}



//tools
//25 7f 7f 7f 7f          and    eax,0x7f7f7f7f
//35 7f 7f 7f 7f          xor    eax,0x7f7f7f7f
//05 7f 7f 7f 7f          add    eax,0x7f7f7f7f
//smallest way to zero reg
//6a 01                   push   0x1
//58                      pop    eax
//34 01                   xor    al,0x1


//positionING
//specify esp location
//specify your location
//generate esp code
//generate unpack
//regenerate esp code
//make sure new esp code == last esp code

//esp and code should be able to go all the way up to FFFF FFFF

//if jump mode specified and jump address given, then at the end position the jmp for new bytes, store the byte len, redo calcs, then check if equal

//TEST COMMAND
//[pos]+[add esp,0x300]+[jmp]
//incode.exe --esp 19CF54 --eip 197758 --code "81 c4 00 03 00 00" --jump 19D588
//then after that ill need some code to put esp in ecx


//TODO
//idea: make a mini toolkit just for generating ascii safe versions of common commands without wrapping
//also a jump calculator etc etc
//make it so --code is optional if bytes were the first thing specified