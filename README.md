erimulator

***

> [!WARNING]
> This is NOT a complete NES emulator. If you are looking to play games, not yet.

A NES/6502 emulator written in Rust.

91/151 official opcodes implemented

currently:
- [x] gui
  - [x] window + framebuffer texture
  - [x] open rom button
  - [ ] debugs (cpu register, memory viewer, pattern table and palette viewer)
- [x] cartridge loading, parsing
  - [x] ines header, prg/chr rom, trainer
  - [ ] chr ram (chr size 0)
- [ ] cpu emulation
  - [x] bus: ram + prg rom
  - [x] stack: jsr, rts, pha, pla, php, plp
  - [x] flags: sec, clc, sei, cli, sed, cld, clv
  - [x] branches
  - [x] loads, stores, transfers
  - [x] logic: and, ora, eor, bit
  - [x] arithmetic: adc, sbc, compares
  - [ ] inc/dec, shifts, rotates
  - [ ] all addressing modes
  - [ ] interrupts: reset, nmi, irq, brk, rti
  - [ ] nestest passes all official opcodes
  - [ ] unofficial opcodes (maybe)
- [ ] ppu emulation
  - [ ] registers ($2000-$2007)
  - [ ] vblank + nmi
  - [ ] background rendering
  - [ ] sprites + sprite 0 hit
  - [ ] scrolling
- [ ] controls
  - [ ] controller register ($4016)
  - [ ] keyboard mapping
- [ ] timing
- [ ] audio emulation
- [ ] mappers except mapper 0

## special thanks
- nesdev wiki: https://www.nesdev.org/
- retro6502: https://github.com/seasalim/retro6502/
- 100th coin: https://www.patreon.com/100th_Coin/posts/making-your-nes-137873901
- 6502.org: https://6502.org/users/obelisk/6502/reference.html

## ai usage disclosure
- ai was used to explain concepts, fix and explain bugs, and write tests. all actually running code is written by me.
