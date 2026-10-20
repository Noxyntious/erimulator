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

pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY
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

    fn effective_address(&mut self, mode: AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Immediate => {
                let address = self.pc;
                self.pc = self.pc.wrapping_add(1);
                (address, false)
            }
            AddressingMode::ZeroPage => {
                let address = self.bus.read(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                (address, false)
            }
            AddressingMode::ZeroPageX => {
                let address = self.bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let address = address.wrapping_add(self.x) as u16;
                (address, false)
            }
            AddressingMode::ZeroPageY => {
                let address = self.bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let address = address.wrapping_add(self.y) as u16;
                (address, false)
            }
            AddressingMode::Absolute => {
                let address = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                (address, false)
            }
            AddressingMode::AbsoluteX => {
                let base = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let address = base.wrapping_add(self.x as u16);
                let crossed = (base & 0xFF00) != (address & 0xFF00);
                (address, crossed)
            }
            AddressingMode::AbsoluteY => {
                let base = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let address = base.wrapping_add(self.y as u16);
                let crossed = (base & 0xFF00) != (address & 0xFF00);
                (address, crossed)
            }
            AddressingMode::IndirectX => {
                let base = self.bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let address = base.wrapping_add(self.x);
                let effective_address_lo = self.bus.read(address as u16);
                let effective_address_hi = self.bus.read(address.wrapping_add(1) as u16);
                let effective_address = u16::from_le_bytes([effective_address_lo, effective_address_hi]);
                (effective_address, false)
            }
            AddressingMode::IndirectY => {
                let base = self.bus.read(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let address_lo = self.bus.read(base as u16);
                let address_hi = self.bus.read(base.wrapping_add(1) as u16);
                let address = u16::from_le_bytes([address_lo, address_hi]);
                let effective_address = address.wrapping_add(self.y as u16);
                let crossed = (address & 0xFF00) != (effective_address & 0xFF00);
                (effective_address, crossed)
            }
        }
    }

    fn lda(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.a = self.bus.read(address);
        self.set_zn(self.a);
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }
    fn ldx(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.x = self.bus.read(address);
        self.set_zn(self.x);
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }
    fn ldy(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.y = self.bus.read(address);
        self.set_zn(self.y);
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }

    fn sta(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, _) = self.effective_address(mode);
        self.bus.write(address, self.a);
        self.cycles += cycles;
    }
    fn stx(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, _) = self.effective_address(mode);
        self.bus.write(address, self.x);
        self.cycles += cycles;
    }
    fn sty(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, _) = self.effective_address(mode);
        self.bus.write(address, self.y);
        self.cycles += cycles;
    }

    fn ora(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.a |= self.bus.read(address);
        self.set_zn(self.a);
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }

    fn and(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.a &= self.bus.read(address);
        self.set_zn(self.a);
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }

    fn eor(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.a ^= self.bus.read(address);
        self.set_zn(self.a);
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }

    fn cmp(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.compare(self.a, self.bus.read(address));
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }

    fn cpx(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.compare(self.x, self.bus.read(address));
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }

    fn cpy(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        self.compare(self.y, self.bus.read(address));
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
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

    fn adc2(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        let value = self.bus.read(address);
        self.adc(value);
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
    }

    fn sbc2(&mut self, mode: AddressingMode, cycles: u64) {
        let (address, page_crossed) = self.effective_address(mode);
        let value = self.bus.read(address);
        self.sbc(value);
        self.cycles += cycles;
        if page_crossed == true {
            self.cycles += 1;
        }
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

    fn lsr(&mut self, value: u8) -> u8 {
        if value & 0x01 != 0 {
            self.status |= 0x01;
        } else {
            self.status &= !0x01;
        }
        let result = value >> 1;
        self.set_zn(result);
        result
    }

    fn asl(&mut self, value: u8) -> u8 {
        if value & 0x80 != 0 {
            self.status |= 0x01;
        } else {
            self.status &= !0x01;
        }
        let result = value << 1;
        self.set_zn(result);
        result
    }

    fn rol(&mut self, value: u8) -> u8 {
        let old_carry = self.status & 0x01;
        if value & 0x80 != 0 {
            self.status |= 0x01;
        } else {
            self.status &= !0x01;
        }
        let result = (value << 1) | old_carry;
        self.set_zn(result);
        result
    }

    fn ror(&mut self, value: u8) -> u8 {
        let old_carry = self.status & 0x01;
        if value & 0x01 != 0 {
            self.status |= 0x01;
        } else {
            self.status &= !0x01;
        }
        let result = (value >> 1) | (old_carry << 7);
        self.set_zn(result);
        result
    }

    fn interrupt(&mut self, vec: u16, brk: bool) {
        self.push((self.pc >> 8) as u8); // pushing high
        self.push(self.pc as u8); // pushing low
        let flags = if brk == true { self.status | 0x30 } else { (self.status & !0x10) | 0x20 };
        self.push(flags);
        self.status |= 0x04;
        self.pc = self.bus.read_u16(vec);
        self.cycles += 7;
    }

    pub fn nmi(&mut self) {
        self.interrupt(0xFFFA, false);
    }

    pub fn irq(&mut self) {
        if self.status & 0x04 == 0 {
            self.interrupt(0xFFFE, false);
        } // if I is clear
    }

    pub fn reset(&mut self) {
        self.pc = self.bus.read_u16(0xFFFC) // point to the games reset vector
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

            // increments
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
            0xE6 => { // INC zero page
                let address = self.bus.read(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let value = self.bus.read(address).wrapping_add(1);
                self.bus.write(address, value);
                self.set_zn(value);
                self.cycles += 5;
            }
            0xF6 => { // INC zero page,X
                let base_address = self.bus.read(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let address = base_address.wrapping_add(self.x as u16) & 0xFF;
                let value = self.bus.read(address).wrapping_add(1);
                self.bus.write(address, value);
                self.set_zn(value);
                self.cycles += 6;
            }
            0xEE => { // INC absolute
                let address = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let value = self.bus.read(address).wrapping_add(1);
                self.bus.write(address, value);
                self.set_zn(value);
                self.cycles += 6;
            }
            0xFE => { // INC absolute,X
                let base_address = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let address = base_address.wrapping_add(self.x as u16);
                let value = self.bus.read(address).wrapping_add(1);
                self.bus.write(address, value);
                self.set_zn(value);
                self.cycles += 7;
            }

            // decrements
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
            0xC6 => { // DEC zero page
                let address = self.bus.read(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let value = self.bus.read(address).wrapping_sub(1);
                self.bus.write(address, value);
                self.set_zn(value);
                self.cycles += 5;
            }
            0xD6 => { // DEC zero page,X
                let base_address = self.bus.read(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let address = base_address.wrapping_add(self.x as u16) & 0xFF;
                let value = self.bus.read(address).wrapping_sub(1);
                self.bus.write(address, value);
                self.set_zn(value);
                self.cycles += 6;
            }
            0xCE => { // DEC absolute
                let address = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let value = self.bus.read(address).wrapping_sub(1);
                self.bus.write(address, value);
                self.set_zn(value);
                self.cycles += 6;
            }
            0xDE => { // DEC absolute,X
                let base_address = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let address = base_address.wrapping_add(self.x as u16);
                let value = self.bus.read(address).wrapping_sub(1);
                self.bus.write(address, value);
                self.set_zn(value);
                self.cycles += 7;
            }

            // transfers
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
            0x40 => { // RTI
                self.status = (self.pull() & !0x10) | 0x20;
                let low = self.pull() as u16;
                let high = self.pull() as u16;
                self.pc = (high << 8) | low;
                self.cycles += 6;
            }

            // logic
            0x29 => self.and(AddressingMode::Immediate, 2),
            0x25 => self.and(AddressingMode::ZeroPage, 3),
            0x35 => self.and(AddressingMode::ZeroPageX, 4),
            0x2D => self.and(AddressingMode::Absolute, 4),
            0x3D => self.and(AddressingMode::AbsoluteX, 4),
            0x39 => self.and(AddressingMode::AbsoluteY, 4),
            0x21 => self.and(AddressingMode::IndirectX, 6),
            0x31 => self.and(AddressingMode::IndirectY, 5),

            0x09 => self.ora(AddressingMode::Immediate, 2),
            0x05 => self.ora(AddressingMode::ZeroPage, 3),
            0x15 => self.ora(AddressingMode::ZeroPageX, 4),
            0x0D => self.ora(AddressingMode::Absolute, 4),
            0x1D => self.ora(AddressingMode::AbsoluteX, 4),
            0x19 => self.ora(AddressingMode::AbsoluteY, 4),
            0x01 => self.ora(AddressingMode::IndirectX, 6),
            0x11 => self.ora(AddressingMode::IndirectY, 5),

            0x49 => self.eor(AddressingMode::Immediate, 2),
            0x45 => self.eor(AddressingMode::ZeroPage, 3),
            0x55 => self.eor(AddressingMode::ZeroPageX, 4),
            0x4D => self.eor(AddressingMode::Absolute, 4),
            0x5D => self.eor(AddressingMode::AbsoluteX, 4),
            0x59 => self.eor(AddressingMode::AbsoluteY, 4),
            0x41 => self.eor(AddressingMode::IndirectX, 6),
            0x51 => self.eor(AddressingMode::IndirectY, 5),

            // loads
            0xA9 => self.lda(AddressingMode::Immediate, 2),
            0xAD => self.lda(AddressingMode::Absolute, 4),
            0xB5 => self.lda(AddressingMode::ZeroPageX, 4),
            0xA1 => self.lda(AddressingMode::IndirectX, 6),
            0xB1 => self.lda(AddressingMode::IndirectY, 5),
            0xBD => self.lda(AddressingMode::AbsoluteX, 4),
            0xA5 => self.lda(AddressingMode::ZeroPage, 3),
            0xB9 => self.lda(AddressingMode::AbsoluteY, 4),

            0xA2 => self.ldx(AddressingMode::Immediate, 2),
            0xAE => self.ldx(AddressingMode::Absolute, 4),
            0xA6 => self.ldx(AddressingMode::ZeroPage, 3),
            0xB6 => self.ldx(AddressingMode::ZeroPageY, 4),
            0xBE => self.ldx(AddressingMode::AbsoluteY, 4),

            0xA0 => self.ldy(AddressingMode::Immediate, 2),
            0xAC => self.ldy(AddressingMode::Absolute, 4),
            0xA4 => self.ldy(AddressingMode::ZeroPage, 3),
            0xB4 => self.ldy(AddressingMode::ZeroPageX, 4),
            0xBC => self.ldy(AddressingMode::AbsoluteX, 4),
            
            // stores
            0x8D => self.sta(AddressingMode::Absolute, 4),
            0x85 => self.sta(AddressingMode::ZeroPage, 3),
            0x95 => self.sta(AddressingMode::ZeroPageX, 4),
            0x9D => self.sta(AddressingMode::AbsoluteX, 5),
            0x99 => self.sta(AddressingMode::AbsoluteY, 5),
            0x81 => self.sta(AddressingMode::IndirectX, 6),
            0x91 => self.sta(AddressingMode::IndirectY, 6),

            0x8E => self.stx(AddressingMode::Absolute, 4),
            0x86 => self.stx(AddressingMode::ZeroPage, 3),
            0x96 => self.stx(AddressingMode::ZeroPageY, 4),

            0x8C => self.sty(AddressingMode::Absolute, 4),
            0x84 => self.sty(AddressingMode::ZeroPage, 3),
            0x94 => self.sty(AddressingMode::ZeroPageX, 4),


            // jumps
            0x4C => { // JMP absolute
                let address = self.bus.read_u16(self.pc);
                self.pc = address;
                self.cycles += 3;
            }
            0x6C => { // JMP indirect
                let address = self.bus.read_u16(self.pc);
                let effective_address_lo = self.bus.read(address);
                let effective_address_hi = if (address & 0x00FF) == 0x00FF {
                    self.bus.read(address & 0xFF00)
                } else {
                    self.bus.read(address.wrapping_add(1))
                };
                let effective_address = u16::from_le_bytes([effective_address_lo, effective_address_hi]);
                self.pc = effective_address;
                self.cycles += 5; 
            }

            // shifts - asl
            0x0A => { // ASL A
                self.a = self.asl(self.a);
                self.cycles += 2;
            }
            0x06 => { // ASL zero page
                let address = self.bus.read(self.pc) as u16;
                let value = self.bus.read(address);
                let result = self.asl(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 5;
            }
            0x16 => { // ASL zero page,X
                let address = self.bus.read(self.pc).wrapping_add(self.x) as u16;
                let value = self.bus.read(address);
                let result = self.asl(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 6;
            }
            0x0E => { // ASL absolute
                let address = self.bus.read_u16(self.pc);
                let value = self.bus.read(address);
                let result = self.asl(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 6;
            }
            0x1E => { // ASL absolute,X
                let address = self.bus.read_u16(self.pc).wrapping_add(self.x as u16);
                let value = self.bus.read(address);
                let result = self.asl(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 7;
            }

            // shifts - lsr
            0x4A => { // LSR A
                self.a = self.lsr(self.a);
                self.cycles += 2;
            }
            0x46 => { // LSR zero page
                let address = self.bus.read(self.pc) as u16;
                let value = self.bus.read(address);
                let result = self.lsr(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 5;
            }
            0x56 => { // LSR zero page,X
                let address = self.bus.read(self.pc).wrapping_add(self.x) as u16;
                let result = self.lsr(self.bus.read(address));
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 6;
            }
            0x4E => { // LSR absolute
                let address = self.bus.read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let result = self.lsr(self.bus.read(address));
                self.bus.write(address, result);
                self.cycles += 6;
            }
            0x5E => { // LSR absolute,X
                let address = self.bus.read_u16(self.pc).wrapping_add(self.x as u16);
                self.pc = self.pc.wrapping_add(2);
                let result = self.lsr(self.bus.read(address));
                self.bus.write(address, result);
                self.cycles += 7;
            }
            
            // shifts - rol
            0x2A => { // ROL A
                self.a = self.rol(self.a);
                self.cycles += 2;
            }
            0x26 => { // ROL zero page
                let address = self.bus.read(self.pc) as u16;
                let value = self.bus.read(address);
                let result = self.rol(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 5;
            }
            0x36 => { // ROL zero page,X
                let address = self.bus.read(self.pc).wrapping_add(self.x) as u16;
                let value = self.bus.read(address);
                let result = self.rol(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 6;
            }
            0x2E => { // ROL absolute
                let address = self.bus.read_u16(self.pc);
                let value = self.bus.read(address);
                let result = self.rol(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 6;
            }
            0x3E => { // ROL absolute,X
                let address = self.bus.read_u16(self.pc).wrapping_add(self.x as u16);
                let value = self.bus.read(address);
                let result = self.rol(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 7;
            }

            // shifts - ror
            0x6A => { // ROR A
                self.a = self.ror(self.a);
                self.cycles += 2;
            }
            0x66 => { // ROR zero page
                let address = self.bus.read(self.pc) as u16;
                let value = self.bus.read(address);
                let result = self.ror(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 5;
            }
            0x76 => { // ROR zero page,X
                let address = self.bus.read(self.pc).wrapping_add(self.x) as u16;
                let value = self.bus.read(address);
                let result = self.ror(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(1);
                self.cycles += 6;
            }
            0x6E => { // ROR absolute
                let address = self.bus.read_u16(self.pc);
                let value = self.bus.read(address);
                let result = self.ror(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 6;
            }
            0x7E => { // ROR absolute,X
                let address = self.bus.read_u16(self.pc).wrapping_add(self.x as u16);
                let value = self.bus.read(address);
                let result = self.ror(value);
                self.bus.write(address, result);
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 7;
            }
        
            // compares
            0xC9 => self.cmp(AddressingMode::Immediate, 2),
            0xC5 => self.cmp(AddressingMode::ZeroPage, 3),
            0xD5 => self.cmp(AddressingMode::ZeroPageX, 4),
            0xCD => self.cmp(AddressingMode::Absolute, 4),
            0xDD => self.cmp(AddressingMode::AbsoluteX, 4),
            0xD9 => self.cmp(AddressingMode::AbsoluteY, 4),
            0xC1 => self.cmp(AddressingMode::IndirectX, 6),
            0xD1 => self.cmp(AddressingMode::IndirectY, 5),

            0xE0 => self.cpx(AddressingMode::Immediate, 2),
            0xE4 => self.cpx(AddressingMode::ZeroPage, 3),
            0xEC => self.cpx(AddressingMode::Absolute, 4),

            0xC0 => self.cpy(AddressingMode::Immediate, 2),
            0xC4 => self.cpy(AddressingMode::ZeroPage, 3),
            0xCC => self.cpy(AddressingMode::Absolute, 4),

            // --
            // miscs (in nestest's order)

            0xEA => { // NOP
                // literally just do nothing
                self.cycles += 2;
            }

            0x69 => self.adc2(AddressingMode::Immediate, 2),
            0x65 => self.adc2(AddressingMode::ZeroPage, 3),
            0x75 => self.adc2(AddressingMode::ZeroPageX, 4),
            0x6D => self.adc2(AddressingMode::Absolute, 4),
            0x7D => self.adc2(AddressingMode::AbsoluteX, 4),
            0x79 => self.adc2(AddressingMode::AbsoluteY, 4),
            0x61 => self.adc2(AddressingMode::IndirectX, 6),
            0x71 => self.adc2(AddressingMode::IndirectY, 5),

            0xE9 => self.sbc2(AddressingMode::Immediate, 2),
            0xE5 => self.sbc2(AddressingMode::ZeroPage, 3),
            0xF5 => self.sbc2(AddressingMode::ZeroPageX, 4),
            0xED => self.sbc2(AddressingMode::Absolute, 4),
            0xFD => self.sbc2(AddressingMode::AbsoluteX, 4),
            0xF9 => self.sbc2(AddressingMode::AbsoluteY, 4),
            0xE1 => self.sbc2(AddressingMode::IndirectX, 6),
            0xF1 => self.sbc2(AddressingMode::IndirectY, 5),
            
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
            0x2C => { // BIT absolute
                let address = self.bus.read_u16(self.pc);
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
                self.pc = self.pc.wrapping_add(2);
                self.cycles += 4;
            }

            0x00 => { // BRK
                self.pc = self.pc.wrapping_add(1);
                self.interrupt(0xFFFE, true);
            }

            _ => {
                panic!("!!! Unknown opcode: {:02X} at {:04X} (dec: {})", opcode, self.pc.wrapping_sub(1), self.pc.wrapping_sub(1)); // real nes hardware wraps around on overflow, so we wanna use wrapping_sub here and everywhere to emulate that behavior. if we used -, rust would panic (well, everywhere except the cycle counter as i dont think that really matters)
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

            // nestest marks unofficial opcodes with a * before the mnemonic, stop at the first one
            if line.as_bytes()[15] == b'*' {
                assert_eq!(number, 5004, "!!! expected the first unofficial opcode at line 5004, got {number}");
                break;
            }

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
