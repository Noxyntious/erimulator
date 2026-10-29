use crate::nes::cartridge::Cartridge;
use crate::nes::ppu::Ppu;

pub struct Bus {
    pub ram: [u8; 2048], // he has ram! get him! 2kb of ram
    pub cartridge: Cartridge, // the cartridge that is currently loaded
    pub ppu: Ppu,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self { ram: [0; 2048], cartridge, ppu: Ppu::new() }
    }
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize],
            0x8000..=0xFFFF => {
                let index = (addr - 0x8000) as usize % self.cartridge.prg_rom.len();
                self.cartridge.prg_rom[index]
            }
            _ => 0,
        }
    }
    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => self.ram[(addr & 0x07FF) as usize] = value,    
            _ => {}
        }
    }
    pub fn read_u16(&mut self, addr: u16) -> u16{
        let lo = self.read(addr);
        let hi = self.read(addr.wrapping_add(1));
        u16::from_le_bytes([lo, hi])
    }
}
