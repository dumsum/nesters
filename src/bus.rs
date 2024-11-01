#[derive(Default)]
pub struct Bus {
    pub addr: u16,
    pub data: u8,
}

impl Bus {
    pub fn new() -> Self {
        Bus::default()
    }
}
