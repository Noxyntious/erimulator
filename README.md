<p align="center">
  <img width="750" height="128" src="assets/erimulatorlogo.png">
</p>

***

> [!WARNING]
> This is NOT a complete NES emulator. If you are looking to play games, not yet.

A NES/6502 emulator written in Rust.

Implemented:
- [x] GUI
  - [x] window + framebuffer texture
  - [x] open rom button
  - [ ] debugs (cpu register, memory viewer, pattern table and palette viewer)
- [x] Cartridge loading, parsing
  - [x] ines header, prg/chr rom, trainer
  - [ ] chr ram (chr size 0)
- [x] CPU emulation
  - [x] bus: ram + prg rom
  - [x] stack: jsr, rts, pha, pla, php, plp
  - [x] flags: sec, clc, sei, cli, sed, cld, clv
  - [x] branches
  - [x] loads, stores, transfers
  - [x] logic: and, ora, eor, bit
  - [x] arithmetic: adc, sbc, compares
  - [x] inc/dec, shifts, rotates
  - [x] all addressing modes
  - [x] interrupts: reset, nmi, irq, brk, rti
  - [x] nestest passes all official opcodes
  - [ ] unofficial opcodes (maybe)
- [ ] PPU emulation
  - [ ] registers ($2000-$2007)
  - [ ] vblank + nmi
  - [ ] background rendering
  - [ ] sprites + sprite 0 hit
  - [ ] scrolling
- [ ] Controls
  - [ ] controller register ($4016)
  - [ ] keyboard mapping
- [ ] Timing
- [ ] Audio emulation
- [ ] Mappers except NROM

## Special thanks
- NESDev wiki: https://www.nesdev.org/
- Retro6502: https://github.com/seasalim/retro6502/
- 100th Coin: https://www.patreon.com/100th_Coin/posts/making-your-nes-137873901
- 6502.org: https://6502.org/users/obelisk/6502/reference.html
- TetaNES: https://github.com/lukexor/tetanes/

## AI usage disclosure
- AI was used to explain concepts, fix and explain bugs, and write tests. All actually running code is written by me.
