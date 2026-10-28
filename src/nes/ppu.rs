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
    pub framecomplete: bool,
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

    pub fn ppu_read(&self, addr: u16, cart: &Cartridge) -> u8 {
        let addr = addr & 0x3fff;
        match addr {
            0x0000..=0x1FFF => {1 as u8}, //placeholders just so that it stops throwing errors
            0x2000..=0x3EFF => {1 as u8},
            0x3F00..=0x3FFF => {1 as u8},
            _ => unreachable!()
        }
    }
}