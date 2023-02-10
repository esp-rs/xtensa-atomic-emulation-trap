#![doc = include_str!("../README.md")]
#![no_std]

pub const PLATFORM_REGISTER_LEN: usize = 16;

const SCOMPARE1_SR: u32 = 12;

const WSR_INSTRUCTION: u32 = 0b00010011_000000000000_0000;
const WSR_INSTRUCTION_MASK: u32 = 0b11111111_000000000000_1111;

const S32C1I_INSTRUCTION: u32 = 0b1110_00000000_0010;
const S32C1I_INSTRUCTION_MASK: u32 = 0b1111_00000000_1111;

static mut SCOMPARE1: u32 = 0;

#[inline(always)]
#[link_section = ".rwtext"]
pub unsafe fn atomic_emulation(pc: u32, save_frame: &mut [u32; PLATFORM_REGISTER_LEN]) -> bool {
    // deref the addr to find the instruction we trapped on.
    // if the PC address isn't word aligned, we need to read two words and capture the relevant instruction
    let insn = if pc % 4 != 0 {
        let prev_aligned = pc & !0x3;
        let offset = (pc - prev_aligned) as usize;

        let buffer = (*((prev_aligned + 4) as *const u32) as u64) << 32
            | (*(prev_aligned as *const u32) as u64); // read two words
        let buffer_bytes = buffer.to_le_bytes();

        u32::from_le_bytes([
            buffer_bytes[offset],
            buffer_bytes[offset + 1],
            buffer_bytes[offset + 2],
            0,
        ])
    } else {
        *(pc as *const u32)
    };

    // first check, is it a WSR instruction? RRR Format
    if (insn & WSR_INSTRUCTION_MASK) == WSR_INSTRUCTION {
        let target = (insn >> 4) & 0b1111;
        let sr = (insn >> 8) & 0b11111111;
        // is the dest register SCOMPARE1?
        if sr == SCOMPARE1_SR {
            // save value in our virtual register
            SCOMPARE1 = save_frame[target as usize];
            return true;
        }
    }

    // next check, is it the S32C1I instruction? RRI8 Format
    if (insn & S32C1I_INSTRUCTION_MASK) == S32C1I_INSTRUCTION {
        // decode the instruction
        let reg_mask = 0b1111;
        let target = (insn >> 4) & reg_mask;
        let source = (insn >> 8) & reg_mask;
        let offset = (insn >> 16) & 0b11111111;

        // get target value and source value (memory address)
        let target_value = save_frame[target as usize];
        let source_value = save_frame[source as usize];

        // get the value from memory
        let source_address = source_value + ((offset as u32) << 2);
        let memory_value = *(source_address as *const u32);

        if memory_value == SCOMPARE1 {
            // update the value in memory
            *(source_address as *mut u32) = target_value;
        }

        save_frame[target as usize] = memory_value;

        return true;
    }

    false
}
