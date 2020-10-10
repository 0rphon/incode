#![feature(int_error_matching, unsized_locals)]

mod generate;
mod input;

use std::fmt::{Display, Debug};
use std::process::exit;

const WELCOME_MESSAGE: &str = "Incode: for encoding your code in code to run in other peoples code!\nCreated by 0rphon\n";

pub trait FormattedUnwrap<T> {
    /// if its debug mode it prints the full panic, if its release it prints the formatted error
    fn unwrap_or_fmt(self) -> T;
}
/// implementing the method on every Result
impl<T, E: Display+Debug> FormattedUnwrap<T> for Result<T, E> {
    fn unwrap_or_fmt(self) -> T {
        if cfg!(debug_assertions) {
            self.unwrap()
        } else {
            self.unwrap_or_else(|e| {
                println!("{}", e);
                exit(0)
            })
        }
    }
}

fn main() {
    println!("{}",WELCOME_MESSAGE);

    let input = input::get_input().unwrap_or_fmt();
    let (esp, eip, jump, wrap) = (
        input.esp.is_some(),
        input.eip.is_some(),
        input.jump.is_some(),
        input.wrap.is_some()
    );

    if input.help {println!("{}",input::HELP_MESSAGE)}
    else if esp != eip {
        println!("ArgsError: You must set both --esp and --eip. {}", 
            input::SEE_HELP);
    }
    else if jump && (!esp||!eip) {
        println!("ArgsError: You must set both --esp and --eip to use --jump. {}", 
            input::SEE_HELP);
    }
    else if esp && eip && wrap && jump {do_position_wrap_jump(input)}
    else if esp && eip && jump {do_position_jump(input)}
    else if esp && eip && wrap {do_position_wrap(input)}
    else if esp && eip {do_position(input)}
    else if wrap {do_wrap(input)}
    else {println!("ArgsError: failed to match arguments. Please notify the dev. {}",
        input::SEE_HELP);
    }
}

fn do_wrap(input: input::UserInput) {
    let bytes = input.wrap.unwrap();
    let output = generate::wrap(&bytes);
    output.display();
}
fn do_position(input: input::UserInput) {
    let esp = input.esp.unwrap();
    let eip = input.eip.unwrap();
    let output = generate::position(esp, eip);
    output.display();
}
fn do_position_wrap(input: input::UserInput) {
    let bytes = input.wrap.unwrap();
    let esp = input.esp.unwrap();
    let eip = input.eip.unwrap();
    let output = generate::position_wrap(&bytes, esp, eip);
    output.display();
}
fn do_position_jump(input: input::UserInput) {println!("Not Implemented yet. Sorry! {:02X?}", input)}
fn do_position_wrap_jump(input: input::UserInput) {println!("Not Implemented yet. Sorry! {:02X?}", input)}



//tools
//25 7f 7f 7f 7f          and    eax,0x7f7f7f7f
//35 7f 7f 7f 7f          xor    eax,0x7f7f7f7f
//05 7f 7f 7f 7f          add    eax,0x7f7f7f7f
//smallest way to zero reg
//6a 01                   push   0x1
//58                      pop    eax
//34 01                   xor    al,0x1


//generate positional wrap
//generate wrapped wrap
//generate jump wrap WITHOUT xor_eax

//TEST COMMAND
//[pos]+[add esp,0x300]+[jmp]
//incode.exe --esp 19CF54 --eip 19D758 --wrap "81 c4 00 03 00 00" --jump 19D588
//then after that ill need some code to put esp in ecx


//TODO
//idea: make a mini toolkit just for generating ascii safe versions of common commands without wrapping
//also a jump calculator etc etc
//make it so --wrap is optional if bytes were the first thing specified




//append the add/sub instruction during the calculation step to get rid of the junk
//add xor/or/and modes
//xor = 0x35
//and = 0x25
//or  = 0x0D



//if you change the wrapper so that it adds/subs based on null bytes at the beginning of the val then it would save room
//so if it just needs 7f7f more dont make a whole new instruction, just add ax
//same with al for 7f