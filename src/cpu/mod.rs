mod flags;
mod instruction;
use super::bus::Bus;
use flags::*;
use instruction::*;

#[derive(Default)]
pub struct Cpu {
    step: u8,
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: Flags,
    inst: Instruction,
    temp: u8,
    irq: bool,
    nmi: bool,
    rst: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu::default()
    }

    pub fn rst(&mut self) {
        self.rst = true;
    }

    pub fn irq(&mut self) {
        if !self.p.i {
            self.irq = true;
        }
    }

    pub fn nmi(&mut self) {
        self.nmi = true;
    }

    /// Advances the CPU by one clock cycle. Returns true when bus action is read.
    pub fn clock(&mut self, bus: &mut Bus) -> bool {
        if self.step == 0 {
            self.inst = if self.rst {
                Instruction::Stack(StackInstruction::Brk(Interrupt::Rst))
            } else if self.nmi {
                Instruction::Stack(StackInstruction::Brk(Interrupt::Nmi))
            } else if self.irq {
                Instruction::Stack(StackInstruction::Brk(Interrupt::Irq))
            } else {
                bus.data.into()
            };
        }

        self.step += 1;

        if self.step == 1 {
            self.pc += 1;
            bus.addr = self.pc;
        } else {
            match self.inst {
                Instruction::Stack(stack_instruction) => match stack_instruction {
                    StackInstruction::Brk(int) => match self.step {
                        2 => {
                            self.pc += match int {
                                Interrupt::Brk | Interrupt::Rst => 1,
                                Interrupt::Nmi | Interrupt::Irq => 0,
                            };

                            bus.addr = 0x100 | self.s as u16;
                            self.s = self.s.wrapping_sub(1);

                            match int {
                                Interrupt::Rst => {}
                                Interrupt::Irq | Interrupt::Nmi | Interrupt::Brk => {
                                    bus.data = ((self.pc & 0xFF00) >> 8) as u8;
                                    return false;
                                }
                            }
                        }
                        3 => {
                            bus.addr = 0x100 | self.s as u16;
                            self.s = self.s.wrapping_sub(1);

                            match int {
                                Interrupt::Rst => {}
                                Interrupt::Irq | Interrupt::Nmi | Interrupt::Brk => {
                                    bus.data = (self.pc & 0x00FF) as u8;
                                    return false;
                                }
                            }
                        }
                        4 => {
                            bus.addr = 0x100 | self.s as u16;
                            self.s = self.s.wrapping_sub(1);
                            match int {
                                Interrupt::Rst => {}
                                Interrupt::Irq | Interrupt::Nmi => {
                                    bus.data = self.p.into();
                                    self.p.i = true;
                                    return false;
                                }
                                Interrupt::Brk => {
                                    bus.data = u8::from(self.p) | 1u8 << 4; //assert B with BRK
                                    self.p.i = true;
                                    return false;
                                }
                            }
                        }
                        5 => {
                            bus.addr = match int {
                                Interrupt::Brk | Interrupt::Irq => 0xFFFE,
                                Interrupt::Nmi => 0xFFFA,
                                Interrupt::Rst => 0xFFFC,
                            }
                        }
                        6 => {
                            self.temp = bus.data;
                            bus.addr = match int {
                                Interrupt::Brk | Interrupt::Irq => 0xFFFF,
                                Interrupt::Nmi => 0xFFFB,
                                Interrupt::Rst => 0xFFFD,
                            };
                        }
                        7 => {
                            self.pc = (bus.data as u16) << 8 | self.temp as u16;
                            bus.addr = self.pc;

                            match int {
                                Interrupt::Rst => self.rst = false,
                                Interrupt::Nmi => self.nmi = false,
                                _ => {}
                            }
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    },
                    StackInstruction::Rti => match self.step {
                        2 => {
                            bus.addr = self.s as u16 + 0x100;
                        }
                        3 => {
                            self.s = self.s.wrapping_add(1);
                            bus.addr = self.s as u16 + 0x100;
                        }
                        4 => {
                            self.s = self.s.wrapping_add(1);
                            self.p = bus.data.into();
                            bus.addr = self.s as u16 + 0x100;
                        }
                        5 => {
                            self.s = self.s.wrapping_add(1);
                            self.temp = bus.data;
                            bus.addr = self.s as u16 + 0x100;
                        }
                        6 => {
                            self.pc = (bus.data as u16) << 8 | self.temp as u16;
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    },
                    StackInstruction::Rts => match self.step {
                        2 => {
                            bus.addr = self.s as u16 + 0x100;
                        }
                        3 => {
                            self.s = self.s.wrapping_add(1);
                            bus.addr = self.s as u16 + 0x100;
                        }
                        4 => {
                            self.s = self.s.wrapping_add(1);
                            self.temp = bus.data;
                            bus.addr = self.s as u16 + 0x100;
                        }
                        5 => {
                            self.pc = (bus.data as u16) << 8 | self.temp as u16;
                            bus.addr = self.pc;
                        }
                        6 => {
                            self.pc += 1;
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    },
                    StackInstruction::Pha => match self.step {
                        2 => {
                            bus.addr = self.s as u16 + 0x100;
                            bus.data = self.a;
                            return false;
                        }
                        3 => {
                            self.s = self.s.wrapping_sub(1);
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    },
                    StackInstruction::Php => match self.step {
                        2 => {
                            bus.addr = self.s as u16 + 0x100;
                            bus.data = u8::from(self.p) | (1u8 << 4); // assert B for PHP
                            return false;
                        }
                        3 => {
                            self.s = self.s.wrapping_sub(1);
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    },
                    StackInstruction::Pla => match self.step {
                        2 => {
                            bus.addr = self.s as u16 + 0x100;
                        }
                        3 => {
                            self.s = self.s.wrapping_add(1);
                            bus.addr = self.s as u16 + 0x100;
                        }
                        4 => {
                            self.a = bus.data;
                            self.p.set_n(self.a);
                            self.p.set_z(self.a);
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    },
                    StackInstruction::Plp => match self.step {
                        2 => {
                            bus.addr = self.s as u16 + 0x100;
                        }
                        3 => {
                            self.s = self.s.wrapping_add(1);
                            bus.addr = self.s as u16 + 0x100;
                        }
                        4 => {
                            self.p = bus.data.into();
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    },
                    StackInstruction::Jsr => match self.step {
                        2 => {
                            self.pc += 1;
                            self.temp = bus.data;
                            bus.addr = self.s as u16 + 0x100;
                        }
                        3 => {
                            bus.addr = self.s as u16 + 0x100;
                            bus.data = ((self.pc & 0xFF00) >> 8) as u8;
                            return false;
                        }
                        4 => {
                            self.s = self.s.wrapping_sub(1);
                            bus.addr = self.s as u16 + 0x100;
                            bus.data = (self.pc & 0x00FF) as u8;
                            return false;
                        }
                        5 => {
                            self.s = self.s.wrapping_sub(1);
                            bus.addr = self.pc;
                        }
                        6 => {
                            self.pc = (bus.data as u16) << 8 | self.temp as u16;
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    },
                },
                Instruction::AccumImpl(accum_impl_instruction) => match self.step {
                    2 => {
                        match accum_impl_instruction {
                            AccumImplInstruction::ReadModifyWrite(
                                read_modify_write_instruction,
                            ) => {
                                self.a = read_modify_write_instruction.execute(self, self.a);
                            }
                            AccumImplInstruction::Internal(internal_instruction) => {
                                internal_instruction.execute(self)
                            }
                        }

                        bus.addr = self.pc;
                        self.step = 0;
                    }

                    _ => unreachable!(),
                },
                Instruction::Imm(ImmInstruction::Read(read_instruction)) => match self.step {
                    2 => {
                        self.pc += 1;
                        let m = bus.data;
                        read_instruction.execute(self, m);

                        bus.addr = self.pc;
                        self.step = 0;
                    }

                    _ => unreachable!(),
                },
                Instruction::Abs(AbsInstruction::Jump(_)) => match self.step {
                    2 => {
                        self.pc += 1;
                        bus.addr = self.pc;
                        self.temp = bus.data;
                    }
                    3 => {
                        self.pc = (bus.data as u16) << 8 | self.temp as u16;
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::Abs(AbsInstruction::Read(read_instruction)) => match self.step {
                    2 => {
                        self.pc += 1;
                        bus.addr = self.pc;
                        self.temp = bus.data;
                    }
                    3 => {
                        self.pc += 1;
                        bus.addr = (bus.data as u16) << 8 | self.temp as u16;
                    }
                    4 => {
                        read_instruction.execute(self, bus.data);
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::Abs(AbsInstruction::Write(write_instruction)) => match self.step {
                    2 => {
                        self.pc += 1;
                        bus.addr = self.pc;
                        self.temp = bus.data;
                    }
                    3 => {
                        self.pc += 1;
                        bus.addr = (bus.data as u16) << 8 | self.temp as u16;
                        bus.data = write_instruction.execute(self);
                        return false;
                    }
                    4 => {
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::Abs(AbsInstruction::ReadModifyWrite(
                    read_modify_write_instruction,
                )) => match self.step {
                    2 => {
                        self.pc += 1;
                        bus.addr = self.pc;
                        self.temp = bus.data;
                    }
                    3 => {
                        self.pc += 1;
                        bus.addr = (bus.data as u16) << 8 | self.temp as u16;
                    }
                    4 => {
                        self.temp = read_modify_write_instruction.execute(self, bus.data);
                        return false;
                    }
                    5 => {
                        bus.data = self.temp;
                        return false;
                    }
                    6 => {
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },

                Instruction::ZeroPage(ZeroPageInstruction::Read(read_instruction)) => {
                    match self.step {
                        2 => {
                            self.pc += 1;
                            bus.addr = bus.data as u16;
                        }
                        3 => {
                            read_instruction.execute(self, bus.data);
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::ZeroPage(ZeroPageInstruction::Write(write_instruction)) => {
                    match self.step {
                        2 => {
                            self.pc += 1;
                            bus.addr = bus.data as u16;
                            bus.data = write_instruction.execute(self);
                            return false;
                        }
                        3 => {
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::ZeroPage(ZeroPageInstruction::ReadModifyWrite(
                    read_modify_write_instruction,
                )) => match self.step {
                    2 => {
                        self.pc += 1;
                        bus.addr = bus.data as u16;
                    }
                    3 => {
                        self.temp = read_modify_write_instruction.execute(self, bus.data);
                        return false;
                    }
                    4 => {
                        bus.data = self.temp;
                        return false;
                    }
                    5 => {
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(read_instruction))
                | Instruction::ZeroPageIdxY(ZeroPageIdxInstruction::Read(read_instruction)) => {
                    match self.step {
                        2 => {
                            self.pc += 1;
                            bus.addr = bus.data as u16;
                        }
                        3 => {
                            bus.addr += match self.inst {
                                Instruction::ZeroPageIdxX(_) => self.x,
                                Instruction::ZeroPageIdxY(_) => self.y,
                                _ => unreachable!(),
                            } as u16;
                            bus.addr &= 0x00FF;
                        }
                        4 => {
                            read_instruction.execute(self, bus.data);
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Write(write_instruction))
                | Instruction::ZeroPageIdxY(ZeroPageIdxInstruction::Write(write_instruction)) => {
                    match self.step {
                        2 => {
                            self.pc += 1;
                            bus.addr = bus.data as u16;
                        }
                        3 => {
                            bus.addr += match self.inst {
                                Instruction::ZeroPageIdxX(_) => self.x,
                                Instruction::ZeroPageIdxY(_) => self.y,
                                _ => unreachable!(),
                            } as u16;
                            bus.addr &= 0x00FF;
                            bus.data = write_instruction.execute(self);
                            return false;
                        }
                        4 => {
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::ReadModifyWrite(
                    read_modify_write_instruction,
                )) => match self.step {
                    2 => {
                        self.pc += 1;
                        bus.addr = bus.data as u16;
                    }
                    3 => {
                        bus.addr += self.x as u16;
                        bus.addr &= 0x00FF;
                    }
                    4 => {
                        self.temp = bus.data;
                        return false;
                    }
                    5 => {
                        bus.data = read_modify_write_instruction.execute(self, self.temp);
                        return false;
                    }
                    6 => {
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::ZeroPageIdxY(ZeroPageIdxInstruction::ReadModifyWrite(_)) => {
                    unreachable!()
                }
                Instruction::AbsIdxX(AbsIdxInstruction::Read(read_instruction))
                | Instruction::AbsIdxY(AbsIdxInstruction::Read(read_instruction)) => {
                    match self.step {
                        2 => {
                            self.pc += 1;
                            self.temp = bus.data;
                            bus.addr = self.pc;
                        }
                        3 => {
                            self.pc += 1;
                            let addr = (bus.data as u16) << 8 | (self.temp as u16);
                            bus.addr = addr
                                + match self.inst {
                                    Instruction::AbsIdxX(_) => self.x,
                                    Instruction::AbsIdxY(_) => self.y,
                                    _ => unreachable!(),
                                } as u16;
                            if (addr & 0xFF00) != (bus.addr & 0xff00) {
                                self.temp = 1;
                            } else {
                                self.temp = 0;
                            }
                        }
                        4 => {
                            if self.temp == 0 {
                                read_instruction.execute(self, bus.data);
                                bus.addr = self.pc;
                                self.step = 0;
                            }
                        }
                        5 => {
                            read_instruction.execute(self, bus.data);
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::AbsIdxX(AbsIdxInstruction::Write(write_instruction))
                | Instruction::AbsIdxY(AbsIdxInstruction::Write(write_instruction)) => {
                    match self.step {
                        2 => {
                            self.pc += 1;
                            self.temp = bus.data;
                            bus.addr = self.pc;
                        }
                        3 => {
                            self.pc += 1;
                            let addr = (bus.data as u16) << 8 | (self.temp as u16);
                            bus.addr = addr
                                + match self.inst {
                                    Instruction::AbsIdxX(_) => self.x,
                                    Instruction::AbsIdxY(_) => self.y,
                                    _ => unreachable!(),
                                } as u16;
                        }
                        4 => {
                            bus.data = write_instruction.execute(self);
                            return false;
                        }
                        5 => {
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                        _ => unreachable!(),
                    }
                }
                Instruction::AbsIdxX(AbsIdxInstruction::ReadModifyWrite(
                    read_modify_write_instruction,
                )) => match self.step {
                    2 => {
                        self.pc += 1;
                        self.temp = bus.data;
                        bus.addr = self.pc;
                    }
                    3 => {
                        self.pc += 1;
                        let addr = (bus.data as u16) << 8 | (self.temp as u16);
                        bus.addr = addr
                            + match self.inst {
                                Instruction::AbsIdxX(_) => self.x,
                                Instruction::AbsIdxY(_) => self.y,
                                _ => unreachable!(),
                            } as u16;
                        if (addr & 0xFF00) != (bus.addr & 0xff00) {
                            self.temp = 1;
                        } else {
                            self.temp = 0;
                        }
                    }
                    4 => {
                        if self.temp != 0 {
                            bus.addr += 0x100;
                        }
                    }
                    5 => {
                        self.temp = bus.data;
                        return false;
                    }
                    6 => {
                        bus.data = read_modify_write_instruction.execute(self, self.temp);
                        return false;
                    }
                    7 => {
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::AbsIdxY(AbsIdxInstruction::ReadModifyWrite(_)) => {
                    unreachable!()
                }
                Instruction::Rel(RelInstruction::Branch(branch_instruction)) => match self.step {
                    2 => {
                        self.pc += 1;
                        self.temp = bus.data;
                        if branch_instruction.execute(self) {
                            let j = (self.temp as i8) as i16;
                            let pc = self.pc.wrapping_add_signed(j);
                            self.temp = if (pc & 0xFF00) != (self.pc & 0xFF00) {
                                1
                            } else {
                                0
                            };
                            self.pc = pc;
                        } else {
                            self.step = 0;
                        }
                        bus.addr = self.pc;
                    }
                    3 => {
                        if self.temp == 0 {
                            self.step = 0;
                        }
                        bus.addr = self.pc;
                    }
                    4 => {
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::IdxInd(IdxIndInstruction::Read(read_instruction)) => match self.step {
                    2 => {
                        self.pc += 1;
                        bus.addr = bus.data as u16;
                    }
                    3 => {
                        bus.addr = bus.addr.wrapping_add(self.x as u16);
                        bus.addr &= 0x00FF;
                    }
                    4 => {
                        self.temp = bus.data;
                        bus.addr = bus.addr.wrapping_add(1);
                        bus.addr &= 0x00FF;
                    }
                    5 => {
                        bus.addr = (bus.data as u16) << 8 | self.temp as u16;
                    }
                    6 => {
                        read_instruction.execute(self, bus.data);
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::IdxInd(IdxIndInstruction::Write(write_instruction)) => match self.step
                {
                    2 => {
                        self.pc += 1;
                        bus.addr = bus.data as u16;
                    }
                    3 => {
                        bus.addr = bus.addr.wrapping_add(self.x as u16);
                        bus.addr &= 0x00FF;
                    }
                    4 => {
                        self.temp = bus.data;
                        bus.addr = bus.addr.wrapping_add(1);
                        bus.addr &= 0x00FF;
                    }
                    5 => {
                        bus.addr = (bus.data as u16) << 8 | self.temp as u16;
                        bus.data = write_instruction.execute(self);
                        return false;
                    }
                    6 => {
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::IndIdx(IndIdxInstruction::Read(read_instruction)) => match self.step {
                    2 => {
                        self.pc += 1;
                        bus.addr = bus.data as u16;
                    }
                    3 => {
                        self.temp = bus.data; // ADL
                        bus.addr = bus.addr.wrapping_add(1) & 0x00FF;
                    }
                    4 => {
                        let adl_idx = self.temp.wrapping_add(self.y);
                        bus.addr = (bus.data as u16) << 8 | adl_idx as u16;
                    }
                    5 => {
                        let adl_idx = (bus.addr & 0x00FF) as u8;
                        if adl_idx < self.temp {
                            bus.addr += 0x100;
                        } else {
                            read_instruction.execute(self, bus.data);
                            bus.addr = self.pc;
                            self.step = 0;
                        }
                    }
                    6 => {
                        read_instruction.execute(self, bus.data);
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::IndIdx(IndIdxInstruction::Write(write_instruction)) => match self.step
                {
                    2 => {
                        self.pc += 1;
                        bus.addr = bus.data as u16;
                    }
                    3 => {
                        self.temp = bus.data; // ADL
                        bus.addr = bus.addr.wrapping_add(1) & 0x00FF;
                    }
                    4 => {
                        let adl_idx = self.temp.wrapping_add(self.y);
                        bus.addr = (bus.data as u16) << 8 | adl_idx as u16;
                    }
                    5 => {
                        let adl_idx = (bus.addr & 0x00FF) as u8;
                        if adl_idx < self.temp {
                            bus.addr += 0x100;
                        }
                        bus.data = write_instruction.execute(self);
                        return false;
                    }
                    6 => {
                        bus.addr = self.pc;
                        self.step = 0;
                    }
                    _ => unreachable!(),
                },
                Instruction::AbsInd(_) => match self.step {
                    2 => {
                        self.pc += 1;
                        self.temp = bus.data;
                        bus.addr = self.pc;
                    }
                    3 => {
                        self.pc += 1;
                        bus.addr = (bus.data as u16) << 8 | self.temp as u16;
                    }
                    4 => {
                        self.temp = bus.data;
                        let adh = bus.addr & 0xFF00;
                        let adl = (bus.addr & 0x00FF) as u8;
                        let adl = adl.wrapping_add(1);
                        bus.addr = adh | adl as u16;
                    }
                    5 => {
                        self.pc = (bus.data as u16) << 8 | self.temp as u16;
                        self.step = 0;
                        bus.addr = self.pc;
                    }
                    _ => unreachable!(),
                },
                Instruction::Invalid(op) => panic!("Invalid Instruction {op:#04x}"),
            };
        }

        // handle reset immediately
        if self.rst {
            self.step = 0;
        }

        // IRQ is level triggered - needs to be set each clock.
        self.irq = false;

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;

    #[test]
    fn _6502_functional_test() {
        let mut ram = [0u8; 0x10000];

        let path = Path::new("data/6502_functional_test.bin");
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(why) => panic!("Couldn't open {}: {}", path.display(), why),
        };

        match file.read(&mut ram) {
            Ok(n) => assert_eq!(n, 0x10000),
            Err(why) => panic!("Couldn't read {}: {}", path.display(), why),
        }

        let mut cpu = Cpu::new();
        let mut bus = Bus::new();

        cpu.pc = 0x400;
        cpu.step = 0;
        bus.addr = 0x400;
        bus.data = ram[0x400];

        for _ in 0u64..96241364 {
            if cpu.clock(&mut bus) {
                bus.data = ram[bus.addr as usize];
            } else {
                ram[bus.addr as usize] = bus.data;
            }
        }

        assert_eq!(cpu.pc, 0x3469);
    }
}
