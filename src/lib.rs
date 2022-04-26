#![no_std]

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
unsafe fn __exception(cause: ExceptionCause, save_frame: &mut Context) {
    writeln!(Uart, "Exception: {:?}", cause).ok();
    writeln!(Uart, "PC = {}", save_frame.PC).ok();
    writeln!(Uart, "A3 = {}", save_frame.A3).ok();
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

pub unsafe fn atomic_emulation(save_frame: &Context) -> bool {
    
    // deref the addr to find the instruction we trapped on.
    let insn: usize = *(save_frame.PC as *const _);

    writeln!(Uart, "After").ok();

    // first check, is it a WSR instruction? RRR Format
    if (insn & 0b11111111_000000000000_1111) != 0b00010011_000000000000_0000 {
        let target = (insn >> 4) & 0b1111;
        let sr = (insn >> 8) & 0b11111111;
        writeln!(Uart, "Emulating WSR, target reg = {}, special reg = {}", target, sr).ok();
        if sr == SCOMPARE1_SR { // is the dest register SCOMPARE1
            // write the source _value_ into our emulated SCOMPARE1
        }
    }
        

    // next check, is it the S32C1I instruction? RRI8 Format
    if (insn & 0b1111_00000000_1111) != 0b1110_00000000_0011 {
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
