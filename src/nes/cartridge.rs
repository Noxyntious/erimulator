#[derive(Debug, PartialEq)]
pub struct Cartridge {
    pub header: Header,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}
#[derive(Debug, PartialEq)]
pub struct Header {
    pub prg_rom_size: u8,
    pub chr_rom_size: u8,
    pub mapper: u8,
    pub mirroring: Mirroring,
    pub has_trainer: bool,
}

#[derive(Debug, PartialEq)]
pub enum CartParseError {
    InvalidHeader,
    InvalidPrgRomSize,
    InvalidChrRomSize,
    BadMagic,
}

#[derive(Debug, PartialEq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
}

impl Cartridge {
    pub fn from_bytes(bytes: &[u8]) -> Result<Cartridge, CartParseError> {
        let header_bytes = bytes.get(0..16).ok_or(CartParseError::InvalidHeader)?;
        let header = parse_header(header_bytes)?;
        
        let prg_rom_start = 16 + if header.has_trainer { 512 } else { 0 }; // skip trainer if present
        let prg_rom_size = (header.prg_rom_size as usize) * 16384; // program rom size in bytes
        let prg_rom_end = prg_rom_start + prg_rom_size;
        let prg_rom = bytes.get(prg_rom_start..prg_rom_end).ok_or(CartParseError::InvalidPrgRomSize)?.to_vec();

        let chr_rom_size = (header.chr_rom_size as usize) * 8192; // character rom size in bytes
        let chr_rom_start = prg_rom_end;
        let chr_rom_end = chr_rom_start + chr_rom_size;
        let chr_rom = bytes.get(chr_rom_start..chr_rom_end).ok_or(CartParseError::InvalidChrRomSize)?.to_vec();

        Ok(Cartridge {
            header,
            prg_rom,
            chr_rom,
        })
    }
}

pub fn parse_header(header: &[u8]) -> Result<Header, CartParseError> {
    if header.len() != 16 {
        return Err(CartParseError::InvalidHeader);
    }
    if &header[0..4] != b"NES\x1A" { // header doesnt start with "NES\x1A"
        return Err(CartParseError::BadMagic);
    }

    let prg_rom_size = header[4];
    let chr_rom_size = header[5];
    let mapper = (header[6] >> 4) | if header[7] & 0x0C != 0x08 && header[12..16].iter().any(|&b| b != 0) { 0 } else { header[7] & 0xF0 }; // ignore byte 7 if not nes 2.0 and bytes 12-15 have junk (apparently this causes other emulators to crash or fail to load roms. btw: who the hell is diskdude?)
    let has_trainer = header[6] & 0b0000_0100 != 0;
    let mirroring = if header[6] & 0b0001 != 0 {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    };

    Ok(Header {
        prg_rom_size,
        chr_rom_size,
        mapper,
        mirroring,
        has_trainer,
    })
}

//tests commence

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let header_bytes = [
            0x4E, 0x45, 0x53, 0x1A, // NES\x1A / magic
            0x02, // program size (2 * 16KB = 32KB)
            0x01, // char rom size (1 * 8KB = 8KB) - 0 means we are using chr ram instead
            0x00, // flags 6 - mapper lower nibble, mirroring, battery, trainer
            0x00, // flags 7 - mapper upper nibble, VS/Playchoice
            0x00, // Flags 8 - mapper upper nibble, prg-ram size
            0x00, // Flags 9 - mapper upper nibble, TV system
            0x00, // Flags 10 - tv system, prg-ram presence (honestly not sure what this is for)
            0x00, 0x00, 0x00, 0x00, 0x00, // padding
        ];
        let header = parse_header(&header_bytes).unwrap();
        assert_eq!(header.prg_rom_size, 2);
        assert_eq!(header.chr_rom_size, 1);
        assert_eq!(header.mapper, 0);   
        assert_eq!(header.mirroring, Mirroring::Horizontal);
        assert_eq!(header.has_trainer, false);
    }
    #[test]
    fn test_empty_input() { // what if we just give nothing
        let empty_bytes: [u8; 0] = [];
        let result = Cartridge::from_bytes(&empty_bytes);
        assert_eq!(result, Err(CartParseError::InvalidHeader));
    }

    #[test]
    fn test_bad_magic() { // all zeroes instead of NES\x1A
        let bad_magic_bytes = [
            0x00, 0x00, 0x00, 0x00, // not NES\x1A
            0x02, // program size (2 * 16KB = 32KB)
            0x01, // char rom size (1 * 8KB = 8KB) - 0 means we are using chr ram instead
            0x00, // flags 6 - mapper lower nibble, mirroring, battery, trainer
            0x00, // flags 7 - mapper upper nibble, VS/Playchoice
            0x00, // Flags 8 - mapper upper nibble, prg-ram size
            0x00, // Flags 9 - mapper upper nibble, TV system
            0x00, // Flags 10 - tv system, prg-ram presence (honestly not sure what this is for)
            0x00, 0x00, 0x00, 0x00, 0x00, // padding
        ];
        let result = Cartridge::from_bytes(&bad_magic_bytes);
        assert_eq!(result, Err(CartParseError::BadMagic));
    }

    #[test]
    fn test_truncated_prg_rom() { // header promises 1 prg bank but the file ends early
        let mut bytes = vec![
            0x4E, 0x45, 0x53, 0x1A, // NES\x1A / magic
            0x01, // program size (1 * 16KB)
            0x01, // char rom size (1 * 8KB)
            0x00, 0x00, 0x00, 0x00, 0x00, // flags 6-10
            0x00, 0x00, 0x00, 0x00, 0x00, // padding
        ];
        bytes.extend(vec![0; 100]); // only 100 bytes of prg instead of 16384
        let result = Cartridge::from_bytes(&bytes);
        assert_eq!(result, Err(CartParseError::InvalidPrgRomSize));
    }

    #[test]
    fn test_truncated_chr_rom() { // prg is all there but chr is cut short
        let mut bytes = vec![
            0x4E, 0x45, 0x53, 0x1A, // NES\x1A / magic
            0x01, // program size (1 * 16KB)
            0x01, // char rom size (1 * 8KB)
            0x00, 0x00, 0x00, 0x00, 0x00, // flags 6-10
            0x00, 0x00, 0x00, 0x00, 0x00, // padding
        ];
        bytes.extend(vec![0; 16384]); // full prg bank
        bytes.extend(vec![0; 100]); // only 100 bytes of chr instead of 8192
        let result = Cartridge::from_bytes(&bytes);
        assert_eq!(result, Err(CartParseError::InvalidChrRomSize));
    }

    #[test]
    fn test_trainer_is_skipped() { // prg should start after the 512 byte trainer
        let mut bytes = vec![
            0x4E, 0x45, 0x53, 0x1A, // NES\x1A / magic
            0x01, // program size (1 * 16KB)
            0x01, // char rom size (1 * 8KB)
            0x04, // flags 6 - trainer bit set
            0x00, 0x00, 0x00, 0x00, // flags 7-10
            0x00, 0x00, 0x00, 0x00, 0x00, // padding
        ];
        bytes.extend(vec![0xFF; 512]); // trainer, filled with FF so it stands out
        bytes.push(0xAB); // marker as the first prg byte
        bytes.extend(vec![0; 16383]); // rest of the prg bank
        bytes.extend(vec![0xCD; 8192]); // chr bank, filled with CD

        let cartridge = Cartridge::from_bytes(&bytes).unwrap();
        assert_eq!(cartridge.header.has_trainer, true);
        assert_eq!(cartridge.header.mapper, 0); // trainer bit must not leak into the mapper
        assert_eq!(cartridge.prg_rom.len(), 16384);
        assert_eq!(cartridge.prg_rom[0], 0xAB);
        assert_eq!(cartridge.chr_rom.len(), 8192);
        assert_eq!(cartridge.chr_rom[0], 0xCD);
    }

    #[test]
    fn test_diskdude_junk_ignored() { // old dumps with "DiskDude!" in bytes 7-15 shouldnt get a bogus mapper
        let mut header_bytes = [0u8; 16];
        header_bytes[0..4].copy_from_slice(b"NES\x1A");
        header_bytes[4] = 0x01; // program size
        header_bytes[5] = 0x01; // char rom size
        header_bytes[6] = 0x10; // flags 6 - mapper lower nibble = 1
        header_bytes[7..16].copy_from_slice(b"DiskDude!");
        let header = parse_header(&header_bytes).unwrap();
        assert_eq!(header.mapper, 1); // 'D' (0x44) would make this 0x41 if byte 7 was used
    }

    #[test]
    fn test_mapper_upper_nibble() { // clean header should still use byte 7 for the upper nibble
        let mut header_bytes = [0u8; 16];
        header_bytes[0..4].copy_from_slice(b"NES\x1A");
        header_bytes[4] = 0x01; // program size
        header_bytes[5] = 0x01; // char rom size
        header_bytes[6] = 0x10; // flags 6 - mapper lower nibble = 1
        header_bytes[7] = 0x40; // flags 7 - mapper upper nibble = 4
        let header = parse_header(&header_bytes).unwrap();
        assert_eq!(header.mapper, 0x41);
    }
}   