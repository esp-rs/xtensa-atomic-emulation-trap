# Xtensa atomic emulation trap handler

## Usage

### Additional RUSTFLAGS

Add the following rustflags to `.cargo/config.toml` in your project. Take care not to overwrite any existing ones.

```toml
rustflags = [
# enable the atomic codegen option for Xtensa
"-C", "target-feature=+s32c1i",

# tell the core library have atomics even though it's not specified in the target definition
"--cfg", 'target_has_atomic="8"',
"--cfg", 'target_has_atomic="16"',
"--cfg", 'target_has_atomic="32"',
"--cfg", 'target_has_atomic="ptr"',
]
```

and then call `atomic_emulation` in the exception handler.

## How it works

We build code for silicon that has the `s32c1i` feature, then when our target attempts to execute these instructions it throws an illegal instruction exception, at which point we can decode the instruction and emulate it in software.

There is only one atomic instruction to emulate in the Xtensa ISA, `S32C1I`.
However, compare values are written to the `SCOMPARE1` (Special Reg No. 12) register, so in silicon without this feature it won't exist. We need to emulate the instruction and the register for sucessful atomic emulation.

See 4.3.14.2 of the ISA RM for an example atomic compare swap loop.

| Instruction  | Format |    Instruction composition      |
| ------------ | ------ | ------------------------------- |
| WSR          | RSR    | `0001_0011_0000_0000_0000_0000` |
| S32C1I       | RRI8   | `XXXX_XXXX_1110_XXXX_XXXX_0010` |

To emulate the WSR instruction, we must first decode it and verify that the target register is 12, the `SCOMPARE1` register. Once that is confirmed, we can use this crates virtual `SCOMPARE1` to store the value.

Emulation of the `S32C1I` instruction is a little more complicated. First we decode the entire instruction to get the following values:

- `target register` - this contains the new value we wish to swap to
- `source register` - this conaints the address in memory of the current value
- `offset` - optional offset to add to the address in the source register

We deference the `source address` + `offset` to find the current value and compare it to the stored value inside our `SCOMPARE1` virtual register. If they are equal, the new target value is written to memory at the `source address` + `offset`. Regardless of whether the new value is written the old value is always written back into the target register.

## Usage with xtensa_lx_rt

An example of how to use this crate with xtensa-lx-rt can be found in [v0.3.1](https://github.com/esp-rs/xtensa-atomic-emulation-trap/tree/5b90a6073a2ed971915856652cf0ff1cdff112d0).

## I get linker errors when I build for debug

Follow the instructions [here](https://github.com/esp-rs/xtensa-lx-rt#i-get-linker-errors-when-i-build-for-debug), and also add the following.

```toml
[profile.dev.package.xtensa-atomic-emulation-trap]
opt-level = 'z'
```