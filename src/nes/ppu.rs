use crate::nes::cartridge::{Cartridge, Mirroring};

pub struct Ppu {
    pub vram: [u8; 0x800],
    pub oam: [u8; 0x100],
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub oamaddr: u8,
    pub palette: [u8; 0x20],
    pub v: u16,  // VRAM addr
    pub t: u16,  // Transfer addr
    pub x: u8,   // x scroll
    pub w: bool, // write toggle
    pub read_buf: u8,
    pub scanline: u16,
    pub dot: u16,
    pub frame_complete: bool,
    pub nmi: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; 0x800],
            oam: [0; 0x100],
            ctrl: 0,
            mask: 0,
            status: 0,
            oamaddr: 0,
            palette: [0; 0x20],
            v: 0,
            t: 0,
            x: 0,
            w: false,
            read_buf: 0,
            scanline: 0,
            dot: 0,
            frame_complete: false,
            nmi: false,
        }
    }

    fn vram_index(&self, addr: u16, mirroring: &Mirroring) -> usize {
        let addr = (addr - 0x2000) & 0x0fff;
        let table = addr / 0x400;
        let offset = addr % 0x400;
        let bank = match (mirroring, table) {
            (Mirroring::Horizontal, 0 | 1) => 0,
            (Mirroring::Horizontal, 2 | 3) => 1,
            (Mirroring::Vertical, 0 | 2) => 0,
            (Mirroring::Vertical, 1 | 3) => 1,
            _ => unreachable!(),
        };
        bank * 0x400 + offset as usize
    }

    fn palette_index(&self, addr: u16) -> usize {
        let mut addr = addr & 0x1F;
        match addr {
            0x10 | 0x14 | 0x18 | 0x1C => addr -= 0x10,
            _ => {}
        }
        addr as usize
    }

    pub fn ppu_read(&self, addr: u16, cart: &Cartridge) -> u8 {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x1FFF => cart.chr_rom.get(addr as usize).copied().unwrap_or(0),
            0x2000..=0x3EFF => self.vram[self.vram_index(addr, &cart.header.mirroring)],
            0x3F00..=0x3FFF => self.palette[self.palette_index(addr)],
            _ => unreachable!(),
        }
    }

    pub fn ppu_write(&mut self, addr: u16, value: u8, cart: &Cartridge) {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x1FFF => {
                // dont write to chr rom because its rom (icba to do chr ram right now)
            }
            0x2000..=0x3EFF => {
                let vramindex = self.vram_index(addr, &cart.header.mirroring);
                self.vram[vramindex] = value;
            }
            0x3F00..=0x3FFF => {
                let paletteindex = self.palette_index(addr);
                self.palette[paletteindex] = value;
            }
            _ => unreachable!(),
        }
    }

    fn inc_v(&mut self) {
        if (self.ctrl & 0x04) != 0 {
            self.v = self.v.wrapping_add(32);
        } else {
            self.v = self.v.wrapping_add(1);
        }
    }

    pub fn cpu_read(&mut self, reg: u16, cart: &Cartridge) -> u8 {
        let reg = reg & 7;
        match reg {
            2 => {
                let status = self.status;
                self.status &= !0x80;
                self.w = false;
                status
            }
            4 => self.oam[self.oamaddr as usize],
            7 => {
                let result;
                if (self.v & 0x3fff) < 0x3f00 {
                    result = self.read_buf;
                    self.read_buf = self.ppu_read(self.v, cart);
                } else {
                    result = self.ppu_read(self.v, cart);
                    self.read_buf = self.ppu_read(self.v - 0x1000, cart);
                }
                self.inc_v();
                result
            }
            _ => 0,
        }
    }

    pub fn cpu_write(&mut self, reg: u16, value: u8, cart: &Cartridge) {
        let reg = reg & 7;
        match reg {
            0 => {
                self.ctrl = value;
                let mask = !0x0C00;
                self.t = (self.t & mask) | (((value & 0b11) as u16) << 10);
            }
            1 => self.mask = value,
            2 => {}
            3 => self.oamaddr = value,
            4 => {
                self.oam[self.oamaddr as usize] = value;
                self.oamaddr = self.oamaddr.wrapping_add(1);
            }
            5 => {
                if !self.w {
                    self.t = (self.t & 0xFFE0) | ((value >> 3) as u16);
                    self.x = value & 7;
                } else {
                    self.t = (self.t & !0x73e0) | (((value & 7) as u16) << 12);
                    self.t = (self.t) | ((value >> 3) as u16) << 5;
                }
                self.w = !self.w;
            }
            6 => {
                if !self.w {
                    self.t = (self.t & 0x00ff) | ((value & 0x3f) as u16) << 8;
                } else {
                    self.t = (self.t & 0xff00) | (value as u16);
                    self.v = self.t;
                }
                self.w = !self.w;
            }
            7 => {
                self.ppu_write(self.v, value, cart);
                self.inc_v();
            }
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.dot += 1;
        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;
        }

        if self.scanline > 261 {
            self.scanline = 0;
            self.frame_complete = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nes::cartridge::{Cartridge, Header, Mirroring};

    const PPUCTRL: u16 = 0x2000;
    const PPUSTATUS: u16 = 0x2002;
    const OAMADDR: u16 = 0x2003;
    const OAMDATA: u16 = 0x2004;
    const PPUSCROLL: u16 = 0x2005;
    const PPUADDR: u16 = 0x2006;
    const PPUDATA: u16 = 0x2007;

    // a cart with blank chr, horizontal mirroring
    fn blank_cart() -> Cartridge {
        Cartridge {
            header: Header {
                prg_rom_size: 1,
                chr_rom_size: 1,
                mapper: 0,
                mirroring: Mirroring::Horizontal,
                has_trainer: false,
            },
            prg_rom: vec![0; 16384],
            chr_rom: vec![0; 8192],
        }
    }

    // two writes to $2006, high byte first
    fn set_addr(ppu: &mut Ppu, addr: u16, cart: &Cartridge) {
        ppu.cpu_write(PPUADDR, (addr >> 8) as u8, cart);
        ppu.cpu_write(PPUADDR, (addr & 0xFF) as u8, cart);
    }

    #[test]
    fn test_2006_sets_v() {
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        ppu.cpu_write(PPUADDR, 0x21, &cart);
        assert_eq!(
            ppu.v, 0,
            "v shouldnt change after only the first $2006 write"
        );
        assert!(ppu.w, "w should be set after the first $2006 write");
        ppu.cpu_write(PPUADDR, 0x08, &cart);
        assert_eq!(ppu.v, 0x2108, "v should be 0x2108 after writing 21 then 08");
        assert!(!ppu.w, "w should be cleared after the second $2006 write");
    }

    #[test]
    fn test_2006_high_byte_is_6_bits() {
        // bit 14 and 15 get dropped
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        set_addr(&mut ppu, 0xFF00, &cart);
        assert_eq!(ppu.v, 0x3F00);
    }

    #[test]
    fn test_2007_read_is_buffered() {
        // first read gives the stale buffer, values come one read late
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        set_addr(&mut ppu, 0x2000, &cart);
        ppu.cpu_write(PPUDATA, 0xAB, &cart);
        ppu.cpu_write(PPUDATA, 0xCD, &cart);

        set_addr(&mut ppu, 0x2000, &cart);
        assert_eq!(
            ppu.cpu_read(PPUDATA, &cart),
            0x00,
            "first read should return the old buffer"
        );
        assert_eq!(
            ppu.cpu_read(PPUDATA, &cart),
            0xAB,
            "second read should return the byte at 0x2000"
        );
        assert_eq!(
            ppu.cpu_read(PPUDATA, &cart),
            0xCD,
            "third read should return the byte at 0x2001"
        );
    }

    #[test]
    fn test_2007_palette_read_is_immediate() {
        // palette skips the buffer, buffer gets the nametable byte under it
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        set_addr(&mut ppu, 0x2F00, &cart);
        ppu.cpu_write(PPUDATA, 0x77, &cart);
        set_addr(&mut ppu, 0x3F00, &cart);
        ppu.cpu_write(PPUDATA, 0x0F, &cart);

        set_addr(&mut ppu, 0x3F00, &cart);
        assert_eq!(
            ppu.cpu_read(PPUDATA, &cart),
            0x0F,
            "palette read should not be delayed"
        );
        assert_eq!(
            ppu.read_buf, 0x77,
            "buffer should hold the nametable byte at 0x2F00"
        );
    }

    #[test]
    fn test_2007_increment() {
        let cart = blank_cart();
        let mut ppu = Ppu::new();

        set_addr(&mut ppu, 0x2000, &cart);
        ppu.cpu_write(PPUDATA, 0, &cart);
        assert_eq!(ppu.v, 0x2001, "ctrl bit 2 clear should add 1");

        ppu.cpu_write(PPUCTRL, 0x04, &cart);
        set_addr(&mut ppu, 0x2000, &cart);
        ppu.cpu_write(PPUDATA, 0, &cart);
        assert_eq!(ppu.v, 0x2020, "ctrl bit 2 set should add 32");

        ppu.cpu_read(PPUDATA, &cart);
        assert_eq!(ppu.v, 0x2040, "reads should increment too");
    }

    #[test]
    fn test_2002_read_side_effects() {
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        ppu.status = 0xE0;
        ppu.cpu_write(PPUADDR, 0x21, &cart); // leaves w set

        assert_eq!(
            ppu.cpu_read(PPUSTATUS, &cart),
            0xE0,
            "read should return status before clearing vblank"
        );
        assert_eq!(
            ppu.status, 0x60,
            "vblank bit should be cleared after the read"
        );
        assert!(!ppu.w, "w should be reset by reading $2002");
    }

    #[test]
    fn test_2000_nametable_bits_go_into_t() {
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        ppu.cpu_write(PPUCTRL, 0x03, &cart);
        assert_eq!(ppu.t, 0x0C00, "nametable 3 should set bits 10-11 of t");

        ppu.cpu_write(PPUCTRL, 0xFC, &cart);
        assert_eq!(
            ppu.t, 0x0000,
            "other ctrl bits shouldnt leak into t, and nametable 0 should clear bits 10-11"
        );
        assert_eq!(ppu.ctrl, 0xFC);
    }

    #[test]
    fn test_2005_scroll() {
        let cart = blank_cart();
        let mut ppu = Ppu::new();

        ppu.cpu_write(PPUSCROLL, 0x7D, &cart); // 01111 101 -> coarse x 15, fine x 5
        assert_eq!(ppu.t, 0x000F, "first write should set coarse x");
        assert_eq!(ppu.x, 5, "first write should set fine x");
        assert!(ppu.w);

        ppu.cpu_write(PPUSCROLL, 0x5E, &cart); // 01011 110 -> coarse y 11, fine y 6
        assert_eq!(
            ppu.t, 0x616F,
            "second write should set fine y (bits 12-14) and coarse y (bits 5-9)"
        );
        assert!(!ppu.w);
    }

    #[test]
    fn test_2005_second_write_clears_old_y() {
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        ppu.cpu_write(PPUSCROLL, 0x00, &cart);
        ppu.cpu_write(PPUSCROLL, 0xFF, &cart);
        ppu.cpu_write(PPUSCROLL, 0x00, &cart);
        ppu.cpu_write(PPUSCROLL, 0x00, &cart);
        assert_eq!(ppu.t, 0x0000, "writing 0 for y should clear the old y bits");
    }

    #[test]
    fn test_oam_data() {
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        ppu.cpu_write(OAMADDR, 0x10, &cart);
        ppu.cpu_write(OAMDATA, 0x42, &cart);
        assert_eq!(ppu.oam[0x10], 0x42);
        assert_eq!(ppu.oamaddr, 0x11, "writing $2004 should increment oamaddr");

        ppu.cpu_write(OAMADDR, 0x10, &cart);
        assert_eq!(ppu.cpu_read(OAMDATA, &cart), 0x42);
        assert_eq!(
            ppu.oamaddr, 0x10,
            "reading $2004 shouldnt increment oamaddr"
        );
    }

    #[test]
    fn test_registers_mirror_every_8() {
        let cart = blank_cart();
        let mut ppu = Ppu::new();
        ppu.cpu_write(0x3FFE, 0x21, &cart); // $3FFE mirrors $2006
        ppu.cpu_write(0x200E, 0x08, &cart); // so does $200E
        assert_eq!(ppu.v, 0x2108);
    }
}
