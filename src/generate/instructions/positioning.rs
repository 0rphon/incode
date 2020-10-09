use super::InstructionSet;
use super::translate::get_bytes_u32;

#[derive(Debug)]
enum Direction {
    Up,
    Down,
}

impl InstructionSet {
    pub fn generate_positional(&self, esp: u32, eip: u32) {
        let (dif, dir) = Self::get_difference(esp, eip);
        let bytes = get_bytes_u32(dif);
        println!("{:02X?}",bytes);
        if bytes.iter().any(|b| *b>0x7F||*b==0) {
            println!("bad! {:?}", dir)
        } else {
            println!("good! {:?}", dir)   
        }
    }

    fn get_difference(esp: u32, eip: u32) -> (u32, Direction) {
        if eip > esp {(eip-esp, Direction::Up)}
        else {(esp-eip, Direction::Down)}
    }
}


// "\x54",                 //push    esp
// "\x58",                 //pop     eax
// "\x66\x05\x2B\x08",     //add     ax,0x82B
// "\x50",                 //push    eax
// "\x5c",                 //pop     esp


//probably need to rewrite equalize and move it its own function something
//need to equalize but nt care about leading zeros if more than one leading zero

//trim leading zeros
//check for >0x7f || ==0 && len!=3
//  equalize

// 0:  05 7f 7f 7f 7f          add    eax,0x7f7f7f7f
// 5:  66 05 7f 7f             add    ax,0x7f7f
// 9:  04 7f                   add    al,0x7f
// b:  2d 7f 7f 7f 7f          sub    eax,0x7f7f7f7f
// 10: 66 2d 7f 7f             sub    ax,0x7f7f
// 14: 2c 7f                   sub    al,0x7f 



//why the fuck did i set up my structs this way???
//just have everything under InstructionSet and split its different impls into modules
//get rid of the derive macro lol

//fist step tomorrow is combining all this stuff