pub trait MemoryMap {
    fn read(&self, _addr: u16) -> u8;
    fn write(&mut self, _addr: u16, value: u8);
}
