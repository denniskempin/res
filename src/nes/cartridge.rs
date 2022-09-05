use anyhow::anyhow;
use anyhow::Result;

#[derive(Default)]
pub struct Cartridge {
    pub prg: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub chr: Vec<u8>,
}

impl Cartridge {
    pub fn load_program(&mut self, data: &[u8]) {
        self.prg = data.into();
    }

    pub fn load_ines(&mut self, raw: &[u8]) -> Result<()> {
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
        self.prg_ram.resize(8 * 1024, 0);

        Ok(())
    }

    pub fn cpu_bus_peek(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[addr as usize - 0x6000],
            0x8000..=0xFFFF => {
                let addr = addr as usize % self.prg.len();
                self.prg[addr]
            }
            _ => panic!("Warning. Illegal peek from: ${:04X}", addr),
        }
    }

    pub fn cpu_bus_read(&mut self, addr: u16) -> u8 {
        self.cpu_bus_peek(addr)
    }

    pub fn cpu_bus_write(&mut self, addr: u16, value: u8) {
        match addr {
            0x6000..=0x7FFF => self.prg_ram[addr as usize - 0x6000] = value,
            _ => panic!("Warning. Illegal write to: ${:04X}", addr),
        };
    }
}
