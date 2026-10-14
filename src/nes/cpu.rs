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
            0x86 => { // store stx (zero page addressing Mode)
                let address = self.bus.read(self.pc) as u16; 
                self.bus.write(address, self.x);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 3;
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
            0xEA => { // NOP
                // literally just do nothing
                self.cycles += 2;
            }
            0x38 => { // SEC
                self.status |= 0x01; // set carry flag
                self.cycles += 2;
            }
            0x18 => { // CLC
                self.status &= !0x01; // clear carry flag
                self.cycles += 2;
            }
            0xA9 => { // LDA immediate
                let value = self.bus.read(self.pc);
                self.a = value;
                self.set_zn(value);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 2;
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
    use crate::nes::cartridge::Cartridge;

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
}
