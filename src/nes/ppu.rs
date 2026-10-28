use crate::nes::cartridge::{Cartridge, Mirroring};

pub struct Ppu {
    pub vram: [u8; 0x800],
    pub oam: [u8; 0x100],
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub oamaddr: u8,
    pub palette: [u8; 0x20],
    pub v: u16, // VRAM addr
    pub t: u16, // Transfer addr
    pub x: u8, // x scroll
    pub w: bool, // write toggle
    pub read_buf: u8,
    pub scanline: u16,
    pub dot: u16,
    pub frame_complete: bool,
    pub nmi: bool,
}

impl Ppu {
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
            0x10 | 0x14 | 0x18 | 0x1C => { addr -= 0x10 },
            _ => {}
        }
        addr as usize
    }
        

    pub fn ppu_read(&self, addr: u16, cart: &Cartridge) -> u8 {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x1FFF => {
                cart.chr_rom.get(addr as usize).copied().unwrap_or(0)
            },
            0x2000..=0x3EFF => {
                self.vram[self.vram_index(addr, &cart.header.mirroring)]
            },
            0x3F00..=0x3FFF => {
                self.palette[self.palette_index(addr)]
            },
            _ => unreachable!()
        }
    }

    pub fn ppu_write(&mut self, addr: u16, value: u8, cart: &Cartridge) {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x1FFF => {
                // dont write to chr rom because its rom (icba to do chr ram right now)
            },
            0x2000..=0x3EFF => {
                let vramindex = self.vram_index(addr, &cart.header.mirroring);
                self.vram[vramindex] = value;
            },
            0x3F00..=0x3FFF => {
                let paletteindex = self.palette_index(addr);
                self.palette[paletteindex] = value;
            },
            _ => unreachable!()
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