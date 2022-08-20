use anyhow::anyhow;
use anyhow::Result;

#[derive(Default)]
pub struct Cartridge {
    prg: Vec<u8>,
    chr: Vec<u8>,
}

impl Cartridge {
    pub const START_ADDR: u16 = 0x8000;
    pub const END_ADDR: u16 = 0xFFFF;

    pub fn load_program(&mut self, data: &[u8]) {
        self.prg = data.into();
    }

    pub fn load_ines(&mut self, raw: &[u8]) -> Result<()> {
        println!("{:?}", &raw[0..16]);

        if raw[0] != b'N' || raw[1] != b'E' || raw[2] != b'S' {
            return Err(anyhow!("Expected NES header."));
        }
        let prg_len = raw[4] as usize * 16 * 1024;
        let chr_len = raw[5] as usize * 8 * 1024;

        let prg_start = 16;
        let prg_end = prg_start + prg_len;
        let chr_end = prg_end + chr_len;

        if chr_end != raw.len() {
            return Err(anyhow!(
                "Expected rom size to be {}, but it is {}",
                chr_end,
                raw.len()
            ));
        }

        self.prg = raw[prg_start..prg_end].to_vec();
        self.chr = raw[prg_end..chr_end].to_vec();

        Ok(())
    }

    pub fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize % self.prg.len();
        self.prg[addr]
    }

    pub fn write(&mut self, addr: u16, _: u8) {
        panic!("Illegal write to rom device at {addr:04X}.");
    }
}
