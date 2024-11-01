#[derive(Default, Clone, Copy)]
pub(super) struct Flags {
    pub(super) n: bool,
    pub(super) v: bool,
    pub(super) d: bool,
    pub(super) i: bool,
    pub(super) z: bool,
    pub(super) c: bool,
}

impl std::fmt::Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}-B{}{}{}{}",
            if self.n { 'N' } else { 'n' },
            if self.v { 'V' } else { 'v' },
            if self.d { 'D' } else { 'd' },
            if self.i { 'I' } else { 'i' },
            if self.z { 'Z' } else { 'z' },
            if self.c { 'C' } else { 'c' }
        )
    }
}

impl Flags {
    pub(super) fn set_z(&mut self, m: u8) {
        self.z = m == 0;
    }

    pub(super) fn set_n(&mut self, m: u8) {
        self.n = (m as i8) < 0;
    }

    pub(super) fn set_c(&mut self, m: u16) {
        self.c = m > 0xFF;
    }

    pub(super) fn set_v(&mut self, a: u8, m: u8, c: u8) {
        let isum = (a as i8 as i16) + (m as i8 as i16) + (c as i8 as i16);
        self.v = !(-128..=127).contains(&isum);
    }
}

impl From<Flags> for u8 {
    fn from(p: Flags) -> Self {
        let n = if p.n { 1u8 << 7 } else { 0u8 };
        let v = if p.v { 1u8 << 6 } else { 0u8 };
        let u = 1u8 << 5;
        let b = 0u8;
        let d = if p.d { 1u8 << 3 } else { 0u8 };
        let i = if p.i { 1u8 << 2 } else { 0u8 };
        let z = if p.z { 1u8 << 1 } else { 0u8 };
        let c = if p.c { 1u8 << 0 } else { 0u8 };

        n | v | u | b | d | i | z | c
    }
}

impl From<u8> for Flags {
    fn from(value: u8) -> Self {
        Self {
            n: value & (1 << 7) != 0,
            v: value & (1 << 6) != 0,
            d: value & (1 << 3) != 0,
            i: value & (1 << 2) != 0,
            z: value & (1 << 1) != 0,
            c: value & (1 << 0) != 0,
        }
    }
}
