use super::InstructionSet;
use super::translate::{get_full_bytes32, get_bytes32};

use std::u8;

const CONTACT_ME: &str = "Please contact the dev with a sample so this issue can be fixed!";

impl InstructionSet {

    /// generates code to position esp at the end of the payload
    pub fn position(&mut self, esp: u32, eip: u32, payload_size: usize) {
        self.push_esp();
        self.pop_eax();
        let mut last_size = 0;
        loop {
            println!("sp{:08X} ip{:08X}", esp, eip);
            println!("{:08X}+{:08X}+{:08X}+{:08X}",eip, last_size, 4, payload_size);
            println!("{:08X}",eip+last_size as u32+4+payload_size as u32);
            let position = InstructionSet::wrap(eip+last_size as u32+4+payload_size as u32, esp);
            if position.len == last_size {self.extend(position);break}
            else {last_size = position.len} 
        }
        self.push_eax(None);
        self.pop_esp();
    }

    /// generates InstructionSets for the given dword
    pub fn encode(&mut self, dword: u32, eax: &mut u32) {
        let dword_bytes = get_bytes32(dword);
        if dword == *eax {self.push_eax(Some(dword))}
        else if dword_bytes.iter().any(|b| *b>0x7f||*b==0) {
            self.extend(Self::wrap(dword, *eax));
            self.push_eax(Some(dword));
            *eax = dword.clone();
        }else {self.push32(dword)}
    }

    fn wrap(dest: u32, src: u32) -> Self {
        let mut results = Vec::new();
        results.push(Self::check_add(dest, src));
        results.push(Self::check_sub(dest, src));
        results.push(Self::new_oned().combine(Self::check_add(dest, 1)));
        results.push(Self::new_oned().combine(Self::check_sub(dest, 1)));
        for i in (0x1..0x7f7f7f7f).step_by(0x00001111) {
            let b = get_bytes32(i);
            if b.len() != 3 && !b.into_iter().any(|b| b>0x7f||b==0) {
                results.push(Self::new_pre_sub(i)
                    .combine(Self::check_add(dest, src.overflowing_sub(i).0)));
                results.push(Self::new_pre_add(i)
                    .combine(Self::check_sub(dest, src.overflowing_add(i).0)));
            }
        }
        results.sort_by(|a,b| a.len.cmp(&b.len));
        results.remove(0)
    }

    fn new_pre_add(val: u32) -> Self {
        let mut new = Self::new();
        match get_bytes32(val).len() {
            4 => new.add_eax(val),
            2 => new.add_ax(val as u16),
            1 => new.add_al(val as u8),
            _ => {panic!("Cant create add instruction for pre-add value {:08X?}! {}",val,CONTACT_ME)}
        }
        new
    }

    fn new_pre_sub(val: u32) -> Self {
        let mut new = Self::new();
        match get_bytes32(val).len() {
            4 => new.sub_eax(val),
            2 => new.sub_ax(val as u16),
            1 => new.sub_al(val as u8),
            _ => {panic!("Cant create sub instruction for pre-sub value {:08X?}! {}",val,CONTACT_ME)}
        }
        new
    }

    ///creates a new InstructionSet instance with push 1, pop eax
    fn new_oned() -> Self {
        let mut new = Self::new();
        new.one_eax();
        new
    }

    /// generates InstructionSets to add eax to the target value
    fn check_add(dest: u32, src: u32) -> Self {
        let dif = {
            if dest > src {dest-src} 
            else {
                let mut dif: u32 = 0;
                for (i, (t, r)) in get_full_bytes32(dest)
                    .into_iter().zip(get_full_bytes32(src)).enumerate() {
                    if r != 0 {dif += (0xFF_u8.overflowing_sub(r).0 as u32)<<i*8}
                    dif += (t as u32)<<i*8;
                }
                dif+1
            }
        };
        let mut instructions = Self::new();
        if dif == 0 {return instructions}
        for val in Self::gen_values(dif) {
            match get_bytes32(val).len() {
                4 => instructions.add_eax(val),
                2 => instructions.add_ax(val as u16),
                1 => instructions.add_al(val as u8),
                _ => {panic!("Cant create add instruction for generated value {:08X?}! {}",val,CONTACT_ME)}
            }
        }
        instructions
    }

    /// generates InstructionSets to sub eax to the target value
    fn check_sub(dest: u32, src: u32) -> Self {
        let dif = {
            if dest < src {src-dest} 
            else {
                let mut dif: u32 = 0;
                for (i, (t, r)) in get_full_bytes32(dest)
                    .into_iter().zip(get_full_bytes32(src)).enumerate() {
                    if t != 0 {dif += (0xFF_u8.overflowing_sub(t).0 as u32)<<i*8}
                    dif += (r as u32)<<i*8;
                }
                dif+1
            }
        };
        let mut instructions = Self::new();
        if dif == 0 {return instructions}
        for val in Self::gen_values(dif) {
            match get_bytes32(val).len() {
                4 => instructions.sub_eax(val),
                2 => instructions.sub_ax(val as u16),
                1 => instructions.sub_al(val as u8),
                _ => {panic!("Cant create sub instruction for generated value {:08X?}! {}",val,CONTACT_ME)}
            }
        }
        instructions
    }

    /// generates ascii safe values that add up to dif
    fn gen_values(dif: u32) -> Vec<u32> {
        let mut values = vec!(dif);
        while values.iter().any(|v| 
            get_bytes32(*v).iter().any(|b| 
                *b>0x7f||*b==0)) 
        && values.len() < 10 {
            Self::equalize(&mut values);
        }
        values
    }

    /// recursively propagates the value at lines[index] until every line is ascii safe
    fn equalize(values: &mut Vec<u32>) {
        //remove ascii-safe zeros
        for val in 0..values.len() {
            let stripped = get_bytes32(values[val]);
            for (i, byte) in stripped.iter().enumerate() {
                //push overflow down. creates new line if needed
                if *byte > 0x7F {
                    if let None = values.get(val+1) {values.push(0)}
                    values[val+1] += (*byte as u32-0x7F)<<(i*8);
                    values[val] -= 0x7F<<(i*8);
                }
                //steal from above to get rid of nulls. if first line then returns bad instruction
                if *byte == 0 {
                    if val == 0 {values.extend(vec!(1;10));return}         //NEED TO HANDLE THIS BETTER
                    values[val-1]-=0x01<<(i*8);
                    values[val]+=0x01<<(i*8);
                }
            }
        }
    }
}



// Payload size: 17 bytes  jmp -492  e9 10 fe ff ff
// "\x6A\x01",              //push   0x01
// "\x58",                  //pop    eax
// "\x2D\x02\x6F\x6F\x6F",  //sub    eax, 0x6F6F6F02
// "\x50",                  //push   eax              (pushed 0x909090FF)
// "\x05\x7F\x7F\x6D\x6F",  //add    eax, 0x6F6D7F7F
// "\x04\x6B",              //add    al,  0x6B
// "\x50",                  //push   eax              (pushed 0xFFFE10E9)


//get rid of nop padding unless chunk len is 3
//only look at target.len() of eax for push
//equalize should take any size value
//match the final push to the instruction len



//holy shit the way ive set this up, adding u64's would be so easy
//get rid of all size dependencies and make everything go up to u64


// Encoding 5 bytes: [E9, 10, FE, FF, FF]
// Payload size: 17 bytes
// "\x6A\x01",              //push   0x01
// "\x58",                  //pop    eax
// "\x2D\x02\x6F\x6F\x6F",  //sub    eax, 0x6F6F6F02
// "\x50",                  //push   eax              (pushed 0x909090FF)
// "\x05\x6B\x7F\x6D\x6F",  //add    eax, 0x6F6D7F6B
// "\x04\x6B",              //add    al,  0x6B
// "\x50",                  //push   eax              (pushed 0xFFFE10E9)