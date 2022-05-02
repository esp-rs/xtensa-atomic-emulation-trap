#![no_std]
#![feature(asm_experimental_arch)]

use xtensa_lx_rt::exception::{Context, ExceptionCause};

use core::fmt::Write;

// Only one instruction to emulate, `S32C1I`
// Compare values are written to the `SCOMPARE1` (Special Reg No. 12) register, but on chips without this feature it won't exist
// We'll need to emulate the instruction and the register for sucessful atomic emulation

// See 4.3.14.2 of the ISA RM for an example atomic compare swap loop

// Instruction  | Format  
// WSR          | RSR
// S32C1I       | RRI8

#[no_mangle] // TODO #[xtensa_lx_rt::exception] doesn't work
#[link_section = ".rwtext"]
unsafe fn __exception(cause: ExceptionCause, save_frame: &mut Context) {
    match cause {
        ExceptionCause::Illegal if atomic_emulation(save_frame) => {
            save_frame.PC += core::mem::size_of::<usize>() as u32;
        },
        _ => {
            todo!("Allow a custom execption forwarding here")
        }
    }
}

const SCOMPARE1_SR: usize = 12;

static mut SCOMPARE1: u32 = 0;

pub fn test_print() {
    writeln!(Uart, "About to deref").ok();
}

#[link_section = ".rwtext"]
pub unsafe fn atomic_emulation(save_frame: &Context) -> bool {
    // writeln!(Uart, "About to deref = {}", save_frame.PC).ok();
    // deref the addr to find the instruction we trapped on.
    // because the data is in the cache, we need to use special instruction to read from the cache memories
   
    let insn: usize;
    let pc = save_frame.PC;

    let insn = if pc % 4 != 0 {
        let prev_aligned = pc & !0x3;
        let offset = (pc - prev_aligned) as usize;

        let buffer = (*((prev_aligned + 4) as *const u32) as u64) << 32 | (*(prev_aligned as *const u32) as u64); // read two words
        let buffer_bytes = buffer.to_le_bytes();

        usize::from_le_bytes([buffer_bytes[offset], buffer_bytes[offset + 1], buffer_bytes[offset + 2], 0])
    } else {
        *(pc as *const usize)
    };

    writeln!(Uart, "Instruction (word): {}", insn).ok();

    // first check, is it a WSR instruction? RRR Format
    if (insn & 0b11111111_000000000000_1111) == 0b00010011_000000000000_0000 {
        let target = (insn >> 4) & 0b1111;
        let sr = (insn >> 8) & 0b11111111;
        writeln!(Uart, "Emulating WSR, target reg = {}, special reg = {}, value = {}", target, sr, register_value_from_index(target, save_frame)).ok();
        if sr == SCOMPARE1_SR { // is the dest register SCOMPARE1
            // write the source _value_ into our emulated SCOMPARE1
            let target_value = register_value_from_index(target, save_frame);
            writeln!(Uart, "Writing {} to SCOMPARE1 register", target_value).ok();
            SCOMPARE1 = target_value;
            return true
        }
    }
        

    // next check, is it the S32C1I instruction? RRI8 Format
    if (insn & 0b1111_00000000_1111) == 0b1110_00000000_0011 {
        let reg_mask = 0b1111;
        let dest = (insn >> 4) & reg_mask;
        let source = (insn >> 8) & reg_mask;
        let offset = (insn >> 16) & 0b11111111;
    }

    false
}

extern "C" {
    fn uart_tx_one_char(c: u8);
}

struct Uart;

impl core::fmt::Write for Uart {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().iter().for_each(|&c| unsafe { uart_tx_one_char(c) });

        Ok(())
    }
}


fn register_value_from_index(index: usize, save_frame: &Context) -> u32{
    match index {
        0 => save_frame.A0,
        1 => save_frame.A1,
        2 => save_frame.A2,
        3 => save_frame.A3,
        4 => save_frame.A4,
        5 => save_frame.A5,
        6 => save_frame.A6,
        7 => save_frame.A7,
        8 => save_frame.A8,
        9 => save_frame.A9,
        10 => save_frame.A10,
        11 => save_frame.A11,
        12 => save_frame.A12,
        13 => save_frame.A13,
        14 => save_frame.A14,
        15 => save_frame.A15,
        _ => unreachable!()
    }
}

// #[cfg(test)]
// mod test_super {
//     use super::*;

//     const WSR_A0_SCOMPARE1: u32 = 0x000c13;

//     #[test]
//     fn test_wsr_emulation() {
//         let instruction_memory = WSR_A0_SCOMPARE1; // asm!("wsr a0, SCOMPARE1")

//         let context = Context {
//             PC: &instruction_memory as *const _ as _, // address of "instruction_memory" aka where our program halted
//             ..Default::default()
//         };
        
//     }
// }
