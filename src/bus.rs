pub trait BusDevice {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub enum BusEvent {
    Read(u16),
    Write(u16, u8)
}
