use anyhow::anyhow;
use anyhow::Ok;
use anyhow::Result;
use rand::thread_rng;
use rand::Rng;
use std::fmt::Display;
use wasm_timer::SystemTime;

type Reg = usize; // u4 actually.
type Addr = usize; // u12 actually.

pub struct RawOp {
    raw: u16,
}

static FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl RawOp {
    fn from_bytes(bytes: &[u8; 2]) -> RawOp {
        RawOp {
            raw: (bytes[0] as u16) << 8 | (bytes[1] as u16),
        }
    }
    fn nib1(&self) -> u8 {
        ((self.raw & 0xF000) >> 12) as u8
    }
    fn nib2(&self) -> u8 {
        ((self.raw & 0x0F00) >> 8) as u8
    }
    fn nib3(&self) -> u8 {
        ((self.raw & 0x00F0) >> 4) as u8
    }
    fn nib4(&self) -> u8 {
        (self.raw & 0x000F) as u8
    }
    fn byte2(&self) -> u8 {
        (self.raw & 0x00FF) as u8
    }
    fn addr(&self) -> Addr {
        (self.raw & 0x0FFF).into()
    }
}

impl Display for RawOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04x}", self.raw)
    }
}

pub struct Chip8Display {
    pub pixels: [[bool; 64]; 32],
}

impl Default for Chip8Display {
    fn default() -> Self {
        Self {
            pixels: [[false; 64]; 32],
        }
    }
}

impl Chip8Display {
    fn clear(&mut self) -> Result<State> {
        self.pixels = [[false; 64]; 32];
        Ok(State::DisplayUpdated)
    }

    fn draw(&mut self, x: usize, y: usize, bytes: &[u8]) -> Result<bool> {
        let display_height = self.pixels.len();
        let display_width = self.pixels[0].len();
        let mut collision = false;
        for (line_num, line) in bytes.iter().enumerate() {
            for pixel_num in 0..8 {
                let new_pixel = (line & (0x80 >> pixel_num)) != 0;
                if new_pixel {
                    let pixel_y = (y + line_num) % display_height;
                    let pixel_x = (x + pixel_num) % display_width;
                    let pixel = &mut self.pixels[pixel_y][pixel_x];
                    if *pixel {
                        collision = true;
                    }
                    *pixel = !*pixel;
                }
            }
        }
        Ok(collision)
    }
}

pub struct Chip8Timer {
    set_value: u8,
    set_time: SystemTime,
}

impl Default for Chip8Timer {
    fn default() -> Self {
        Self {
            set_value: 0,
            set_time: SystemTime::now(),
        }
    }
}

impl Chip8Timer {
    fn set(&mut self, value: u8) {
        self.set_value = value;
        self.set_time = SystemTime::now();
    }

    pub fn read(&self) -> u8 {
        let duration_since_set = SystemTime::now().duration_since(self.set_time).unwrap();
        let ticks_since_set = (duration_since_set.as_secs_f32() * 60.0)
            .clamp(0.0, 255.0)
            .floor();
        self.set_value.saturating_sub(ticks_since_set as u8)
    }
}

pub enum State {
    Ordinary,
    DisplayUpdated,
    InfiniteLoop,
    Halt,
}

pub struct Chip8 {
    pub register: [u8; 16],
    pub memory: [u8; 4096],
    pub index: Addr,
    pub pc: Addr,
    pub stack: Vec<Addr>,
    pub keypad: [u8; 16],
    pub delay_timer: Chip8Timer,
    pub sound_timer: Chip8Timer,
    pub display: Chip8Display,
    pub keys: [bool; 16],
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8 {
            register: [0; 16],
            memory: [0; 4096],
            index: 0,
            pc: 0x200,
            stack: vec![],
            keypad: [0; 16],
            delay_timer: Chip8Timer::default(),
            sound_timer: Chip8Timer::default(),
            display: Chip8Display::default(),
            keys: [false; 16],
        }
    }
}

impl Chip8 {
    pub fn with_program(program: &[u8]) -> Chip8 {
        let mut cpu = Chip8::default();
        cpu.load_into_memory(0x050, &FONT);
        cpu.load_into_memory(0x200, program);
        cpu
    }

    pub fn load_into_memory(&mut self, location: Addr, block: &[u8]) {
        let end = location + block.len();
        self.memory[location..end].copy_from_slice(block);
    }

    pub fn emulate_until_halt(&mut self) -> Result<()> {
        loop {
            match self.emulate_tick()? {
                State::InfiniteLoop | State::Halt => break,
                _ => (),
            };
        }
        Ok(())
    }

    pub fn emulate_tick(&mut self) -> Result<State> {
        let op = self.fetch_op();
        self.execute_op(op)
    }

    pub fn instruction_at(&self, i: usize) -> RawOp {
        RawOp::from_bytes(&[self.memory[i], self.memory[i + 1]])
    }

    fn fetch_op(&mut self) -> RawOp {
        let raw = self.instruction_at(self.pc);
        self.pc += 2;
        raw
    }

    fn execute_op(&mut self, op: RawOp) -> Result<State> {
        // Operator encoding:
        // X, Y: Register number
        // KK, K: Constant Value
        // NNN: Address
        match op.nib1() {
            // 0x00KK
            0x0 => self.op_system(op.byte2()),
            // 0x1NNN
            0x1 => self.op_jump(op.addr()),
            // 0x2NNN
            0x2 => self.op_call(op.addr()),
            // 0x3XKK
            0x3 => self.op_skip_if_eq_value(op.nib2().into(), op.byte2(), false),
            // 0x4XKK
            0x4 => self.op_skip_if_eq_value(op.nib2().into(), op.byte2(), true),
            // 0x5XY0
            0x5 => self.op_skip_if_eq_req(op.nib2().into(), op.nib3().into(), false),
            // 0x6XKK
            0x6 => self.op_set(op.nib2().into(), op.byte2()),
            // 0x7XKK
            0x7 => self.op_add(op.nib2().into(), op.byte2()),
            // 0x8XYK
            0x8 => self.op_arithmetic(op.nib2().into(), op.nib3().into(), op.nib4()),
            // 0x9XY0
            0x9 => self.op_skip_if_eq_req(op.nib2().into(), op.nib3().into(), true),
            // 0xANNN
            0xA => self.op_set_index(op.addr()),
            // 0xBNNN
            0xB => self.op_jump_reg(op.addr()),
            // 0xCXKK
            0xC => self.op_rand(op.nib2().into(), op.byte2()),
            // 0xDXYK
            0xD => self.op_draw(op.nib2().into(), op.nib3().into(), op.nib4()),
            // 0xEXKK
            0xE => self.op_skip_if_pressed(op.nib2().into(), op.byte2()),
            // 0xFXKK
            0xF => self.op_functions(op.nib2().into(), op.byte2()),
            _ => Err(anyhow!("No such op {op}")),
        }
    }

    // Operations

    fn op_system(&mut self, op: u8) -> Result<State> {
        match op {
            // 0x0000 Empty memory, halt execution.
            0x00 => Ok(State::Halt),
            // 0x00E0 Clear
            0xE0 => self.display.clear(),
            // 0x00EE Return
            0xEE => {
                let addr = self.stack.pop().ok_or_else(|| anyhow!("Empty Stack"))?;
                self.op_jump(addr)
            }
            _ => Err(anyhow!("No such sytem opearation {op}")),
        }
    }

    fn op_jump(&mut self, addr: Addr) -> Result<State> {
        if addr + 2 == self.pc {
            self.pc = addr;
            Ok(State::InfiniteLoop)
        } else {
            self.pc = addr;
            Ok(State::Ordinary)
        }
    }

    fn op_call(&mut self, addr: Addr) -> Result<State> {
        self.stack.push(self.pc);
        self.op_jump(addr)
    }

    fn op_skip_if_eq_value(&mut self, reg: Reg, value: u8, invert: bool) -> Result<State> {
        let check = self.register[reg] == value;
        if check ^ invert {
            self.pc += 2;
        }
        Ok(State::Ordinary)
    }

    fn op_skip_if_eq_req(&mut self, x_reg: Reg, y_reg: Reg, invert: bool) -> Result<State> {
        let check = self.register[x_reg] == self.register[y_reg];
        if check ^ invert {
            self.pc += 2;
        }
        Ok(State::Ordinary)
    }

    fn op_set(&mut self, reg: Reg, value: u8) -> Result<State> {
        self.register[reg] = value;
        Ok(State::Ordinary)
    }

    fn op_add(&mut self, reg: Reg, value: u8) -> Result<State> {
        (self.register[reg], _) = self.register[reg].overflowing_add(value);
        Ok(State::Ordinary)
    }

    fn op_arithmetic(&mut self, reg: Reg, reg2: Reg, op: u8) -> Result<State> {
        let y = self.register[reg2];
        let x = self.register[reg];
        let new_x = match op {
            0x0 => y,
            0x1 => x | y,
            0x2 => x & y,
            0x3 => x ^ y,
            0x4 => {
                let (res, overflow) = x.overflowing_add(y);
                self.register[0xF] = if overflow { 1 } else { 0 };
                res
            }
            0x5 => {
                let (res, overflow) = x.overflowing_sub(y);
                // Note: VX = NOT borrow
                self.register[0xF] = if overflow { 0 } else { 1 };
                res
            }
            0x6 => {
                self.register[0xF] = if x & 0x01 == 1 { 1 } else { 0 };
                x >> 1
            }
            0x7 => {
                let (res, overflow) = y.overflowing_sub(x);
                // Note: VX = NOT borrow
                self.register[0xF] = if overflow { 0 } else { 1 };
                res
            }
            0xE => {
                self.register[0xF] = if x > 127 { 1 } else { 0 };
                x << 1
            }
            _ => return Err(anyhow!("No such arithmetic op '{op:2x}'")),
        };
        self.register[reg] = new_x;
        Ok(State::Ordinary)
    }

    fn op_set_index(&mut self, addr: Addr) -> Result<State> {
        self.index = addr;
        Ok(State::Ordinary)
    }

    fn op_jump_reg(&mut self, addr: Addr) -> Result<State> {
        self.op_jump(addr + self.register[0] as usize)
    }

    fn op_rand(&mut self, reg: Reg, value: u8) -> Result<State> {
        let rand: u8 = thread_rng().gen();
        self.register[reg] = rand & value;
        Ok(State::Ordinary)
    }

    fn op_draw(&mut self, reg_x: Reg, reg_y: Reg, height: u8) -> Result<State> {
        let x = self.register[reg_x];
        let y = self.register[reg_y];
        let start_idx = self.index;
        let end_idx = start_idx + (height as usize);
        let bytes = &self.memory[start_idx..end_idx];
        self.register[0xF] = self.display.draw(x.into(), y.into(), bytes)? as u8;
        Ok(State::DisplayUpdated)
    }

    fn op_skip_if_pressed(&mut self, reg: Reg, op: u8) -> Result<State> {
        let key = self.register[reg] as usize;
        if key >= 16 {
            return Err(anyhow!("No such key: {key}"));
        }
        let value = self.keys[key];
        match op {
            0x9E => {
                if value {
                    self.pc += 2
                }
            }
            0xA1 => {
                if !value {
                    self.pc += 2
                }
            }
            _ => return Err(anyhow!("Invalid key op {op}")),
        };
        Ok(State::Ordinary)
    }

    fn op_functions(&mut self, reg: Reg, op: u8) -> Result<State> {
        match op {
            0x07 => self.register[reg] = self.delay_timer.read(),
            0x0a => {
                match self.keys.into_iter().position(|k| k) {
                    Some(key) => self.register[reg] = key as u8,
                    None => self.pc -= 2, // Repeat the same instruction.
                }
            }
            0x15 => self.delay_timer.set(self.register[reg]),
            0x18 => self.sound_timer.set(self.register[reg]),
            0x1e => self.index = self.index.overflowing_add(self.register[reg] as usize).0,
            0x29 => self.index = 0x050 + (self.register[reg] as usize * 5),
            0x33 => {
                let value = self.register[reg];
                self.memory[self.index] = value / 100;
                self.memory[self.index + 1] = value / 10 % 10;
                self.memory[self.index + 2] = value % 10;
            }
            0x55 => {
                for i in 0..=(reg) {
                    self.memory[self.index + i] = self.register[i];
                }
            }
            0x65 => {
                for i in 0..=(reg) {
                    self.register[i] = self.memory[self.index + i];
                }
            }

            _ => return Err(anyhow!("No such function op {op:2x}")),
        }
        Ok(State::Ordinary)
    }
}

#[cfg(test)]
mod test {
    use super::Chip8;

    use super::RawOp;

    fn run_program(program: &[u16]) -> Chip8 {
        let byte_program: Vec<u8> = program.iter().flat_map(|op| op.to_be_bytes()).collect();
        let mut cpu = Chip8::with_program(&byte_program);
        cpu.emulate_until_halt().unwrap();
        cpu
    }

    #[test]
    fn test_raw_op() {
        let op = RawOp::from_bytes(&[0x12, 0x34]);
        assert_eq!(op.to_string(), "0x1234");
        assert_eq!(op.nib1(), 0x1);
        assert_eq!(op.nib2(), 0x2);
        assert_eq!(op.nib3(), 0x3);
        assert_eq!(op.nib4(), 0x4);
        assert_eq!(op.byte2(), 0x34);
        assert_eq!(op.addr(), 0x234);
    }

    #[test]
    fn test_op_set() {
        let cpu = run_program(&[0x6000]);
        assert_eq!(cpu.register[0x0], 0x00);
        let cpu = run_program(&[0x6312]);
        assert_eq!(cpu.register[0x3], 0x12);
        let cpu = run_program(&[0x6FFF]);
        assert_eq!(cpu.register[0xF], 0xFF);
    }

    #[test]
    fn test_op_add() {
        let cpu = run_program(&[0x7304, 0x7302]);
        assert_eq!(cpu.register[0x3], 0x06);

        // Test overflow
        let cpu = run_program(&[0x73FF, 0x7301]);
        assert_eq!(cpu.register[0x3], 0x00);
    }

    #[test]
    fn test_op_set_index() {
        let cpu = run_program(&[0xA123]);
        assert_eq!(cpu.index, 0x123);

        // Test overflow
        let cpu = run_program(&[0x73FF, 0x7301]);
        assert_eq!(cpu.register[0x3], 0x00);
    }
}
