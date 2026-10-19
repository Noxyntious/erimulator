use crate::nes::bus::Bus;

pub struct Cpu {
    pub a: u8, // accumulator
    pub x: u8, // x register
    pub y: u8, // y register
    pub sp: u8, // stack pointer
    pub status: u8, // status register
    pub pc: u16, // program counter
    pub cycles : u64, // cycle count
    pub bus: Bus, // ambatubus
}

impl Cpu {
    pub fn new(bus: Bus) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            status: 0x24,
            pc: 0,
            cycles: 7,
            bus,
        }
    }

    fn set_zn(&mut self, value: u8) {
        if value == 0 {
            self.status |= 0x02; // set zero flag
        } else {
            self.status &= !0x02; // clear zero flag
        }

        if value & 0x80 != 0 {
            self.status |= 0x80; // set negative flag
        } else {
            self.status &= !0x80; // clear negative flag
        }
    }

    fn push(&mut self, value: u8) {
        let address = 0x0100 | (self.sp as u16);
        self.bus.write(address, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pull(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let address = 0x0100 | (self.sp as u16);
        self.bus.read(address)
    }

    fn branch(&mut self, condition: bool) { // branch helper so that i dont have to write the same shit 8 times
        let offset = self.bus.read(self.pc) as i8;
        self.pc = self.pc.wrapping_add(1);
        if condition {
            let old_pc = self.pc;
            self.pc = self.pc.wrapping_add(offset as u16);
            self.cycles += 1;
            if (old_pc & 0xFF00) != (self.pc & 0xFF00) {
                self.cycles += 1;
            }
        }
        self.cycles += 2;
    }

    fn adc(&mut self, value: u8) {
        let carry = if self.status & 0x01 != 0 { 1 } else { 0 };
        let sum = self.a as u16 + value as u16 + carry as u16;
        let result = sum as u8;

        if sum > 0xFF {
            self.status |= 0x01; // set carry flag
        } else {
            self.status &= !0x01; // unset carry flag
        }

        if ((self.a ^ result) & (value ^ result) & 0x80) != 0 {
            self.status |= 0x40; // set overflow flag
        } else {
            self.status &= !0x40; // unset overflow flag
        }

        self.a = result;
        self.set_zn(self.a);
    }

    fn sbc(&mut self, value: u8) {
        self.adc(!value);
    }

    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);
        if register >= value {
            self.status |= 0x01;
        } else {
            self.status &= !0x01;
        }
        self.set_zn(result);
    }

    pub fn step(&mut self) {
        // fetch instruction
        let opcode = self.bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        match opcode {
            // branches (might as well have these at the top)
            0xB0 => self.branch(self.status & 0x01 != 0), // BCS
            0x90 => self.branch(self.status & 0x01 == 0), // BCC
            0xF0 => self.branch(self.status & 0x02 != 0), // BEQ
            0xD0 => self.branch(self.status & 0x02 == 0), // BNE
            0x30 => self.branch(self.status & 0x80 != 0), // BMI
            0x10 => self.branch(self.status & 0x80 == 0), // BPL
            0x70 => self.branch(self.status & 0x40 != 0), // BVS
            0x50 => self.branch(self.status & 0x40 == 0), // BVC

            // flag sets/clears
            0x38 => { // SEC - set carry
                self.status |= 0x01; 
                self.cycles += 2;
            }
            0x18 => { // CLC - clear carry
                self.status &= !0x01;
                self.cycles += 2;
            }
            0x78 => { // SEI - set interrupt disable
                self.status |= 0x04;
                self.cycles += 2;
            }
            0x58 => { // CLI - clear interrupt disable
                self.status &= !0x04;
                self.cycles += 2;
            }
            0xF8 => { // SED - set decimal mode
                self.status |= 0x08;
                self.cycles += 2;
            }
            0xD8 => { // CLD - clear decimal mode
                self.status &= !0x08;
                self.cycles += 2;
            }
            0xB8 => { // CLV - clear overflow
                self.status &= !0x40;
                self.cycles += 2;
            }

            // stack
            0x08 => { // PHP - push processor status
                self.push(self.status | 0x30);
                self.cycles += 3;
            }
            0x28 => { // PLP - pull processor status
                self.status = (self.pull() & !0x10) | 0x20;
                self.cycles += 4;
            }
            0x48 => { // PHA - push accumulator
                self.push(self.a);
                self.cycles += 3;
            }
            0x68 => { // PLA - pull accumulator
                self.a = self.pull();
                self.set_zn(self.a);
                self.cycles += 4;
            }
            0x20 => { // JSR absolute
                let target = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let return_addr = self.pc.wrapping_sub(1);
                self.push((return_addr >> 8) as u8);
                self.push(return_addr as u8);
                self.pc = target;
                self.cycles += 6;
            }
            0x60 => { // RTS
                let low = self.pull() as u16;
                let high = self.pull() as u16;
                let return_addr = (high << 8) | low;
                self.pc = return_addr.wrapping_add(1);
                self.cycles += 6;
            }

            // logic
            0x29 => { // AND immediate
                let value = self.bus.read(self.pc);
                self.a &= value;
                self.set_zn(self.a);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0x09 => { // ORA immediate
                let value = self.bus.read(self.pc);
                self.a |= value;
                self.set_zn(self.a);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0x49 => { // EOR immediate
                let value = self.bus.read(self.pc);
                self.a ^= value;
                self.set_zn(self.a);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }

            0x4C => { // JMP absolute
                let address = self.bus.read_u16(self.pc);
                self.pc = address;
                self.cycles += 3;
            }
            0xA2 => { // LDX immediate
                let value = self.bus.read(self.pc);
                self.x = value;
                self.set_zn(value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0xA0 => { // LDY immediate
                let value = self.bus.read(self.pc);
                self.y = value;
                self.set_zn(value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0x86 => { // store stx (zero page addressing Mode)
                let address = self.bus.read(self.pc) as u16; 
                self.bus.write(address, self.x);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 3;
            }
            0x85 => { // store sta (zero page addressing Mode)
                let address = self.bus.read(self.pc) as u16; 
                self.bus.write(address, self.a);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 3;
            }
            0xEA => { // NOP
                // literally just do nothing
                self.cycles += 2;
            }
            0xA9 => { // LDA immediate
                let value = self.bus.read(self.pc);
                self.a = value;
                self.set_zn(value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0x69 => { // ADC immediate
                let value = self.bus.read(self.pc);
                self.adc(value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0xE9 => { // SBC immediate
                let value = self.bus.read(self.pc);
                self.sbc(value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0xC9 => { // CMP immediate
                let value = self.bus.read(self.pc);
                self.compare(self.a, value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0xE0 => { // CPX immediate
                let value = self.bus.read(self.pc);
                self.compare(self.x, value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0xC0 => { // CPY immediate
                let value = self.bus.read(self.pc);
                self.compare(self.y, value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
            }
            0x24 => { // BIT zero page
                let address = self.bus.read(self.pc) as u16;
                let value = self.bus.read(address);
                let result = self.a & value;
                if value & 0x80 != 0 {
                    self.status |= 0x80; // set negative flag
                } else {
                    self.status &= !0x80; // clear negative flag
                }
                if value & 0x40 != 0 {
                    self.status |= 0x40; // set overflow flag
                } else {
                    self.status &= !0x40; // clear overflow flag
                }
                if result == 0 {
                    self.status |= 0x02; // set zero flag
                } else {
                    self.status &= !0x02; // clear zero flag
                }
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 3;
            }
            0xC8 => { // INY
                self.y = self.y.wrapping_add(1);
                self.set_zn(self.y);
                self.cycles += 2;
            }
            0xE8 => { // INX
                self.x = self.x.wrapping_add(1);
                self.set_zn(self.x);
                self.cycles += 2;
            }
            0x88 => { // DEY
                self.y = self.y.wrapping_sub(1);
                self.set_zn(self.y);
                self.cycles += 2;
            }
            0xCA => { // DEX
                self.x = self.x.wrapping_sub(1);
                self.set_zn(self.x);
                self.cycles += 2;
            }
            0xA8 => { // TAY
                self.y = self.a;
                self.set_zn(self.y);
                self.cycles += 2;
            }
            0xAA => { // TAX
                self.x = self.a;
                self.set_zn(self.x);
                self.cycles += 2;
            }
            0x98 => { // TYA
                self.a = self.y;
                self.set_zn(self.a);
                self.cycles += 2;
            }
            0x8A => { // TXA
                self.a = self.x;
                self.set_zn(self.a);
                self.cycles += 2;
            }
            0xBA => { // TSX
                self.x = self.sp;
                self.set_zn(self.x);
                self.cycles += 2;
            }
            0x9A => { // TXS
                self.sp = self.x;
                self.cycles += 2;
            }
            0x8E => { // STX absolute
                let address = self.bus.read_u16(self.pc);
                self.bus.write(address, self.x);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 4;
            }
            0xAE => { // LDX absolute
                let address = self.bus.read_u16(self.pc);
                self.x = self.bus.read(address);
                self.set_zn(self.x);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 4;
            }
            0xAD => { // LDA absolute
                let address = self.bus.read_u16(self.pc);
                self.a = self.bus.read(address);
                self.set_zn(self.a);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 4;
            }
            
            _ => {
                panic!("!!! Unknown opcode: {:02X} at {:04X}", opcode, self.pc.wrapping_sub(1)); // real nes hardware wraps around on overflow, so we wanna use wrapping_sub here and everywhere to emulate that behavior. if we used -, rust would panic (well, everywhere except the cycle counter as i dont think that really matters)
            }
        }
    }
}

//tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::nes::cartridge::{Cartridge, Header, Mirroring};

    fn hex_field(line: &str, label: &str) -> u8 {
        let start = line.find(label).unwrap() + label.len();
        u8::from_str_radix(&line[start..start + 2], 16).unwrap()
    }

    #[test]
    fn test_nestest_rom() {
        let rom_bytes = include_bytes!("../../roms/nestest.nes");
        let log = include_str!("../../roms/nestest.log");
        let cartridge = Cartridge::from_bytes(rom_bytes).unwrap();
        let bus = Bus::new(cartridge);
        let mut cpu = Cpu::new(bus);

        cpu.pc = 0xC000;

        for (index, line) in log.lines().enumerate() {
            let number = index + 1;

            let pc = u16::from_str_radix(&line[0..4], 16).unwrap();
            let cycles_start = line.find("CYC:").unwrap() + 4;
            let cycles: u64 = line[cycles_start..].trim().parse().unwrap();

            assert_eq!(cpu.pc, pc, "!!! PC mismatch at line {number}: {line}");
            assert_eq!(cpu.a, hex_field(line, "A:"), "!!! A mismatch at line {number}: {line}");
            assert_eq!(cpu.x, hex_field(line, "X:"), "!!! X mismatch at line {number}: {line}");
            assert_eq!(cpu.y, hex_field(line, "Y:"), "!!! Y mismatch at line {number}: {line}");
            assert_eq!(cpu.status, hex_field(line, "P:"), "!!! P mismatch at line {number}: {line}");
            assert_eq!(cpu.sp, hex_field(line, "SP:"), "!!! SP mismatch at line {number}: {line}");
            assert_eq!(cpu.cycles, cycles, "!!! CYC mismatch at line {number}: {line}");

            cpu.step();
        }
    }
    // a cpu with a blank 16kb rom, for testing instructions without nestest
    fn blank_cpu() -> Cpu {
        let cartridge = Cartridge {
            header: Header {
                prg_rom_size: 1,
                chr_rom_size: 0,
                mapper: 0,
                mirroring: Mirroring::Horizontal,
                has_trainer: false,
            },
            prg_rom: vec![0; 16384],
            chr_rom: vec![],
        };
        Cpu::new(Bus::new(cartridge))
    }

    const CARRY: u8 = 0x01;
    const ZERO: u8 = 0x02;
    const OVERFLOW: u8 = 0x40;
    const NEGATIVE: u8 = 0x80;

    // checks one flag and says which case failed
    fn assert_flag(cpu: &Cpu, flag: u8, expected: bool, name: &str, case: &str) {
        assert_eq!(cpu.status & flag != 0, expected, "{name} flag wrong for {case}");
    }

    #[test]
    fn test_adc() {
        // (a, value, carry in, result, carry out, overflow)
        let cases = [
            (0x50, 0x10, false, 0x60, false, false), // 80 + 16 = 96, fits
            (0x50, 0x50, false, 0xA0, false, true),  // 80 + 80 = 160, signed overflow
            (0x50, 0xD0, false, 0x20, true, false),  // 80 + -48 = 32, unsigned wrap only
            (0xD0, 0x90, false, 0x60, true, true),   // -48 + -112 = -160, both wrap
            (0xFF, 0x01, false, 0x00, true, false),  // wraps to zero
            (0x10, 0x10, true, 0x21, false, false),  // carry in adds 1
        ];

        for (a, value, carry_in, result, carry_out, overflow) in cases {
            let case = format!("ADC {a:02X} + {value:02X} (carry in {carry_in})");
            let mut cpu = blank_cpu();
            cpu.a = a;
            cpu.status = if carry_in { CARRY } else { 0 };

            cpu.adc(value);

            assert_eq!(cpu.a, result, "result wrong for {case}");
            assert_flag(&cpu, CARRY, carry_out, "carry", &case);
            assert_flag(&cpu, OVERFLOW, overflow, "overflow", &case);
            assert_flag(&cpu, ZERO, result == 0, "zero", &case);
            assert_flag(&cpu, NEGATIVE, result & 0x80 != 0, "negative", &case);
        }
    }

    #[test]
    fn test_sbc() {
        // (a, value, carry in, result, carry out, overflow)
        // carry in = true means no borrow, which is how games always call it (SEC first)
        let cases = [
            (0x50, 0x30, true, 0x20, true, false),  // 80 - 48 = 32
            (0x30, 0x50, true, 0xE0, false, false), // 48 - 80 = -32, borrow
            (0x50, 0xB0, true, 0xA0, false, true),  // 80 - -80 = 160, signed overflow
            (0x50, 0x50, true, 0x00, true, false),  // equal values give zero
            (0x50, 0x30, false, 0x1F, true, false), // carry clear subtracts 1 more
        ];

        for (a, value, carry_in, result, carry_out, overflow) in cases {
            let case = format!("SBC {a:02X} - {value:02X} (carry in {carry_in})");
            let mut cpu = blank_cpu();
            cpu.a = a;
            cpu.status = if carry_in { CARRY } else { 0 };

            cpu.sbc(value);

            assert_eq!(cpu.a, result, "result wrong for {case}");
            assert_flag(&cpu, CARRY, carry_out, "carry", &case);
            assert_flag(&cpu, OVERFLOW, overflow, "overflow", &case);
            assert_flag(&cpu, ZERO, result == 0, "zero", &case);
            assert_flag(&cpu, NEGATIVE, result & 0x80 != 0, "negative", &case);
        }
    }

    #[test]
    fn test_compare() {
        // (register, value, carry, zero, negative)
        let cases = [
            (0x10, 0x10, true, true, false),  // equal
            (0x20, 0x10, true, false, false), // register bigger
            (0x10, 0x20, false, false, true), // register smaller, 0x10 - 0x20 = 0xF0
            (0x00, 0xFF, false, false, false), // unsigned compare: 0 < 255, 0x00 - 0xFF = 0x01
        ];

        for (register, value, carry, zero, negative) in cases {
            let case = format!("compare {register:02X} with {value:02X}");
            let mut cpu = blank_cpu();
            cpu.a = 0x42;
            cpu.status = OVERFLOW; // compare must not touch overflow

            cpu.compare(register, value);

            assert_flag(&cpu, CARRY, carry, "carry", &case);
            assert_flag(&cpu, ZERO, zero, "zero", &case);
            assert_flag(&cpu, NEGATIVE, negative, "negative", &case);
            assert_flag(&cpu, OVERFLOW, true, "overflow", &case);
            assert_eq!(cpu.a, 0x42, "compare changed A for {case}");
        }
    }
}
