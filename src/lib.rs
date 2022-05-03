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
#[link_section = ".rwtext"]
unsafe fn __exception(cause: ExceptionCause, save_frame: &mut Context) {
    match cause {
        ExceptionCause::Illegal if atomic_emulation(save_frame) => {
            save_frame.PC += 3; // 24bit instructions
            return;
        },
        _ => {
            // todo!("Allow a custom execption forwarding here")
        }
    }

    write!(Uart, "ERROR unrecoverable exception").ok();

    loop {
    }
}

const SCOMPARE1_SR: usize = 12;

static mut SCOMPARE1: u32 = 0;

pub fn test_print() {
    writeln!(Uart, "About to deref").ok();
}

#[link_section = ".rwtext"]
pub unsafe fn atomic_emulation(save_frame: &mut Context) -> bool {
    writeln!(Uart, "About to deref = {}", save_frame.PC).ok();
    // deref the addr to find the instruction we trapped on.
   
    let pc = save_frame.PC;

    // if the PC address isn't word aligned, we need to read two words and capture the relevant instruction
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
        // writeln!(Uart, "Emulating WSR, target reg = {}, special reg = {}, value = {}", target, sr, register_value_from_index(target, save_frame)).ok();
        if sr == SCOMPARE1_SR { // is the dest register SCOMPARE1
            // write the source _value_ into our emulated SCOMPARE1
            let target_value = register_value_from_index(target, save_frame);
            writeln!(Uart, "Writing {} to SCOMPARE1 register", target_value).ok();
            SCOMPARE1 = target_value;
            return true
        }
    }

    // next check, is it the S32C1I instruction? RRI8 Format
    if (insn & 0b1111_00000000_1111) == 0b1110_00000000_0010 {
        let reg_mask = 0b1111;
        let target = (insn >> 4) & reg_mask;
        let source = (insn >> 8) & reg_mask;
        let offset = (insn >> 16) & 0b11111111;

        let target_value = register_value_from_index(target, save_frame);
        let source_value = register_value_from_index(source, save_frame);
        // let source_address = source_value + ((offset as u32) << 2);

        let source_address = source_value; // TODO use offset when not zero

        // writeln!(Uart, "E S32, target = {}, source = {}, offset = {}", target_value, source_value, offset).ok();

        let memory_value = *(source_address as *const u32);

        writeln!(Uart, "E S32, mem value = {}, scompare: {}", memory_value, SCOMPARE1).ok();

        if memory_value == SCOMPARE1 {
            *(source_address as *mut u32) = target_value;
        }

        let target_value_mut = register_value_mut_from_index(target, save_frame);
        *target_value_mut = memory_value;

        return true;
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

fn register_value_mut_from_index(index: usize, save_frame: &mut Context) -> &mut u32 {
    match index {
        0 => &mut save_frame.A0,
        1 => &mut save_frame.A1,
        2 => &mut save_frame.A2,
        3 => &mut save_frame.A3,
        4 => &mut save_frame.A4,
        5 => &mut save_frame.A5,
        6 => &mut save_frame.A6,
        7 => &mut save_frame.A7,
        8 => &mut save_frame.A8,
        9 => &mut save_frame.A9,
        10 => &mut save_frame.A10,
        11 => &mut save_frame.A11,
        12 => &mut save_frame.A12,
        13 => &mut save_frame.A13,
        14 => &mut save_frame.A14,
        15 => &mut save_frame.A15,
        _ => unreachable!()
    }
}
