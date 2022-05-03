# Xtensa atomic emulation trap handler

## Usage

### The target

You'll need to modify your target to enable the xtensa atomic feature, `s32c1i`.

Example target json for the esp32s2, which doesn't have hardward atomic CAS:

```jsonc
// esp32s2-atomic.json
{
   "arch": "xtensa",
   "cpu": "esp32-s2",
   "data-layout": "e-m:e-p:32:32-i8:8:32-i16:16:32-i64:64-n32",
   "emit-debug-gdb-scripts": false,
   "executables": true,
   "features": "+s32c1i", // <-- Explicitly enable the atomic feature of Xtensa silicon
   "linker": "xtensa-esp32s2-elf-gcc",
   "llvm-target": "xtensa-none-elf",
   "panic-strategy": "abort",
   "relocation-model": "static",
   "target-pointer-width": "32",
   "max-atomic-width": 32,
   "vendor": ""
}
```

### Building

Include this crate somewhere in your code:

```rust
use xtensa_atomic_emulation_trap as _;
```

Then just build with `--target esp32s2-atomic.json` instead of the usual target.

## How it works

We build code for silicon that has the `s32c1i` feature, then when our target attempts to execute these instructions it throws an illegal instruction exception, at which point we can decode the instruction and emulate it in software.

There is only one atomic instruction to emulate in the Xtensa ISA, `S32C1I`.
However, compare values are written to the `SCOMPARE1` (Special Reg No. 12) register, so in silicon without this feature it won't exist. We need to emulate the instruction and the register for sucessful atomic emulation.

See 4.3.14.2 of the ISA RM for an example atomic compare swap loop.

| Instruction  | Format |    Instruction composition    |
| ------------ | ------ | ----------------------------- |
| WSR          | RSR    | 0001_0011_0000_0000_0000_0000 |
| S32C1I       | RRI8   | XXXX_XXXX_1110_XXXX_XXXX_0010 |

To emulate the WSR instruction, we must first decode it and verify that the target register is 12, the `SCOMPARE1` register. Once that is confirmed, we can use this crates virtual `SCOMPARE1` to store the value.

Emulation of the `S32C1I` instruction is a little more complicated. First we decode the entire instruction to get the following values:

- `target register` - this contains the new value we wish to swap to
- `source register` - this conaints the address in memory of the current value
- `offset` - optional offset to add to the address in the source register

We deference the `source address` + `offset` to find the current value and compare it to the stored value inside our `SCOMPARE1` virtual register. If they are equal, the new target value is written to memory at the `source address` + `offset`. Regardless of whether the new value is written the old value is always written back into the target register.
