mod instruction;
mod flags;
use instruction::*;
use flags::*;

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
}

impl Cpu {
    pub fn new() -> Self {
        Cpu::default()
    }

    pub fn clock(&mut self, pins: &mut Pins) {
        if self.step == 0 || pins.rst || pins.nmi || pins.irq {
            self.inst = if pins.rst {
                Instruction::Stack(StackInstruction::Brk(Interrupt::Rst))
            } else if pins.nmi {
                Instruction::Stack(StackInstruction::Brk(Interrupt::Nmi))
            } else if pins.irq {
                Instruction::Stack(StackInstruction::Brk(Interrupt::Irq))
            } else {
                pins.data.into()
            };
        }

        self.step += 1;
        pins.write = false;

        match self.inst {
            Instruction::Stack(stack_instruction) => match stack_instruction {
                StackInstruction::Brk(int) => match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        self.pc += match int {
                            Interrupt::Brk | Interrupt::Rst => 1,
                            Interrupt::Nmi | Interrupt::Irq => 0,
                        };

                        pins.addr = 0x100 | self.s as u16;
                        self.s = self.s.wrapping_sub(1);

                        match int {
                            Interrupt::Rst => {}
                            Interrupt::Irq | Interrupt::Nmi | Interrupt::Brk => {
                                pins.data = ((self.pc & 0xFF00) >> 8) as u8;
                                pins.write = true;
                            }
                        }
                    }
                    3 => {
                        pins.addr = 0x100 | self.s as u16;
                        self.s = self.s.wrapping_sub(1);

                        match int {
                            Interrupt::Rst => {}
                            Interrupt::Irq | Interrupt::Nmi | Interrupt::Brk => {
                                pins.data = (self.pc & 0x00FF) as u8;
                                pins.write = true;
                            }
                        }
                    }
                    4 => {
                        pins.addr = 0x100 | self.s as u16;
                        self.s = self.s.wrapping_sub(1);
                        match int {
                            Interrupt::Rst => {}
                            Interrupt::Irq | Interrupt::Nmi => {
                                pins.data = self.p.into();
                                self.p.i = true;
                                pins.write = true;
                            }
                            Interrupt::Brk => {
                                pins.data = u8::from(self.p) | 1u8 << 4; //assert B with BRK
                                self.p.i = true;
                                pins.write = true;
                            }
                        }
                    }
                    5 => {
                        pins.addr = match int {
                            Interrupt::Brk | Interrupt::Irq => 0xFFFE,
                            Interrupt::Nmi => 0xFFFA,
                            Interrupt::Rst => 0xFFFC,
                        }
                    }
                    6 => {
                        self.temp = pins.data;
                        pins.addr = match int {
                            Interrupt::Brk | Interrupt::Irq => 0xFFFF,
                            Interrupt::Nmi => 0xFFFB,
                            Interrupt::Rst => 0xFFFD,
                        };
                    }
                    7 => {
                        self.pc = (pins.data as u16) << 8 | self.temp as u16;
                        pins.addr = self.pc;
                        self.step = 0;
                    }

                    _ => panic!(),
                },
                StackInstruction::Rti => match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        pins.addr = self.s as u16 + 0x100;
                    }
                    3 => {
                        self.s = self.s.wrapping_add(1);
                        pins.addr = self.s as u16 + 0x100;
                    }
                    4 => {
                        self.s = self.s.wrapping_add(1);
                        self.p = pins.data.into();
                        pins.addr = self.s as u16 + 0x100;
                    }
                    5 => {
                        self.s = self.s.wrapping_add(1);
                        self.temp = pins.data;
                        pins.addr = self.s as u16 + 0x100;
                    }
                    6 => {
                        self.pc = (pins.data as u16) << 8 | self.temp as u16;
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                },
                StackInstruction::Rts => match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        pins.addr = self.s as u16 + 0x100;
                    }
                    3 => {
                        self.s = self.s.wrapping_add(1);
                        pins.addr = self.s as u16 + 0x100;
                    }
                    4 => {
                        self.s = self.s.wrapping_add(1);
                        self.temp = pins.data;
                        pins.addr = self.s as u16 + 0x100;
                    }
                    5 => {
                        self.pc = (pins.data as u16) << 8 | self.temp as u16;
                        pins.addr = self.pc;
                    }
                    6 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                },
                StackInstruction::Pha => match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        pins.addr = self.s as u16 + 0x100;
                        pins.data = self.a;
                        pins.write = true;
                    }
                    3 => {
                        self.s = self.s.wrapping_sub(1);
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                },
                StackInstruction::Php => match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        pins.addr = self.s as u16 + 0x100;
                        pins.data = u8::from(self.p) | (1u8 << 4); // assert B for PHP
                        pins.write = true;
                    }
                    3 => {
                        self.s = self.s.wrapping_sub(1);
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                },
                StackInstruction::Pla => match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        pins.addr = self.s as u16 + 0x100;
                    }
                    3 => {
                        self.s = self.s.wrapping_add(1);
                        pins.addr = self.s as u16 + 0x100;
                    }
                    4 => {
                        self.a = pins.data;
                        self.p.set_n(self.a);
                        self.p.set_z(self.a);
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                },
                StackInstruction::Plp => match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        pins.addr = self.s as u16 + 0x100;
                    }
                    3 => {
                        self.s = self.s.wrapping_add(1);
                        pins.addr = self.s as u16 + 0x100;
                    }
                    4 => {
                        self.p = pins.data.into();
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                },
                StackInstruction::Jsr => match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        self.pc += 1;
                        self.temp = pins.data;
                        pins.addr = self.s as u16 + 0x100;
                    }
                    3 => {
                        pins.addr = self.s as u16 + 0x100;
                        pins.data = ((self.pc & 0xFF00) >> 8) as u8;
                        pins.write = true;
                    }
                    4 => {
                        self.s = self.s.wrapping_sub(1);
                        pins.addr = self.s as u16 + 0x100;
                        pins.data = (self.pc & 0x00FF) as u8;
                        pins.write = true;
                    }
                    5 => {
                        self.s = self.s.wrapping_sub(1);
                        pins.addr = self.pc;
                    }
                    6 => {
                        self.pc = (pins.data as u16) << 8 | self.temp as u16;
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                },
            },
            Instruction::AccumImpl(accum_impl_instruction) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    match accum_impl_instruction {
                        AccumImplInstruction::ReadModifyWrite(read_modify_write_instruction) => {
                            self.a = read_modify_write_instruction.execute(self, self.a);
                        }
                        AccumImplInstruction::Internal(internal_instruction) => {
                            internal_instruction.execute(self)
                        }
                    }

                    pins.addr = self.pc;
                    self.step = 0;
                }

                _ => panic!(),
            },
            Instruction::Imm(ImmInstruction::Read(read_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    let m = pins.data;
                    read_instruction.execute(self, m);

                    pins.addr = self.pc;
                    self.step = 0;
                }

                _ => panic!(),
            },
            Instruction::Abs(AbsInstruction::Jump(_)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                    self.temp = pins.data;
                }
                3 => {
                    self.pc = (pins.data as u16) << 8 | self.temp as u16;
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::Abs(AbsInstruction::Read(read_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                    self.temp = pins.data;
                }
                3 => {
                    self.pc += 1;
                    pins.addr = (pins.data as u16) << 8 | self.temp as u16;
                }
                4 => {
                    read_instruction.execute(self, pins.data);
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::Abs(AbsInstruction::Write(write_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                    self.temp = pins.data;
                }
                3 => {
                    self.pc += 1;
                    pins.addr = (pins.data as u16) << 8 | self.temp as u16;
                    pins.data = write_instruction.execute(self);
                    pins.write = true;
                }
                4 => {
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::Abs(AbsInstruction::ReadModifyWrite(read_modify_write_instruction)) => {
                match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                        self.temp = pins.data;
                    }
                    3 => {
                        self.pc += 1;
                        pins.addr = (pins.data as u16) << 8 | self.temp as u16;
                    }
                    4 => {
                        self.temp = read_modify_write_instruction.execute(self, pins.data);
                        pins.write = true;
                    }
                    5 => {
                        pins.data = self.temp;
                        pins.write = true;
                    }
                    6 => {
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                }
            }

            Instruction::ZeroPage(ZeroPageInstruction::Read(read_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = pins.data as u16;
                }
                3 => {
                    read_instruction.execute(self, pins.data);
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::ZeroPage(ZeroPageInstruction::Write(write_instruction)) => match self.step
            {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = pins.data as u16;
                    pins.data = write_instruction.execute(self);
                    pins.write = true;
                }
                3 => {
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::ZeroPage(ZeroPageInstruction::ReadModifyWrite(
                read_modify_write_instruction,
            )) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = pins.data as u16;
                }
                3 => {
                    self.temp = read_modify_write_instruction.execute(self, pins.data);
                    pins.write = true;
                }
                4 => {
                    pins.data = self.temp;
                    pins.write = true;
                }
                5 => {
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(read_instruction))
            | Instruction::ZeroPageIdxY(ZeroPageIdxInstruction::Read(read_instruction)) => {
                match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        self.pc += 1;
                        pins.addr = pins.data as u16;
                    }
                    3 => {
                        pins.addr += match self.inst {
                            Instruction::ZeroPageIdxX(_) => self.x,
                            Instruction::ZeroPageIdxY(_) => self.y,
                            _ => unreachable!(),
                        } as u16;
                        pins.addr &= 0x00FF;
                    }
                    4 => {
                        read_instruction.execute(self, pins.data);
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                }
            }
            Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Write(write_instruction))
            | Instruction::ZeroPageIdxY(ZeroPageIdxInstruction::Write(write_instruction)) => {
                match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        self.pc += 1;
                        pins.addr = pins.data as u16;
                    }
                    3 => {
                        pins.addr += match self.inst {
                            Instruction::ZeroPageIdxX(_) => self.x,
                            Instruction::ZeroPageIdxY(_) => self.y,
                            _ => unreachable!(),
                        } as u16;
                        pins.addr &= 0x00FF;
                        pins.data = write_instruction.execute(self);
                        pins.write = true;
                    }
                    4 => {
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                }
            }
            Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::ReadModifyWrite(
                read_modify_write_instruction,
            )) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = pins.data as u16;
                }
                3 => {
                    pins.addr += self.x as u16;
                    pins.addr &= 0x00FF;
                }
                4 => {
                    self.temp = pins.data;
                    pins.write = true;
                }
                5 => {
                    pins.data = read_modify_write_instruction.execute(self, self.temp);
                    pins.write = true;
                }
                6 => {
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::ZeroPageIdxY(ZeroPageIdxInstruction::ReadModifyWrite(_)) => unreachable!(),
            Instruction::AbsIdxX(AbsIdxInstruction::Read(read_instruction))
            | Instruction::AbsIdxY(AbsIdxInstruction::Read(read_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    self.temp = pins.data;
                    pins.addr = self.pc;
                }
                3 => {
                    self.pc += 1;
                    let addr = (pins.data as u16) << 8 | (self.temp as u16);
                    pins.addr = addr
                        + match self.inst {
                            Instruction::AbsIdxX(_) => self.x,
                            Instruction::AbsIdxY(_) => self.y,
                            _ => unreachable!(),
                        } as u16;
                    if (addr & 0xFF00) != (pins.addr & 0xff00) {
                        self.temp = 1;
                    } else {
                        self.temp = 0;
                    }
                }
                4 => {
                    if self.temp == 0 {
                        read_instruction.execute(self, pins.data);
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                }
                5 => {
                    read_instruction.execute(self, pins.data);
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::AbsIdxX(AbsIdxInstruction::Write(write_instruction))
            | Instruction::AbsIdxY(AbsIdxInstruction::Write(write_instruction)) => {
                match self.step {
                    1 => {
                        self.pc += 1;
                        pins.addr = self.pc;
                    }
                    2 => {
                        self.pc += 1;
                        self.temp = pins.data;
                        pins.addr = self.pc;
                    }
                    3 => {
                        self.pc += 1;
                        let addr = (pins.data as u16) << 8 | (self.temp as u16);
                        pins.addr = addr
                            + match self.inst {
                                Instruction::AbsIdxX(_) => self.x,
                                Instruction::AbsIdxY(_) => self.y,
                                _ => unreachable!(),
                            } as u16;
                    }
                    4 => {
                        pins.data = write_instruction.execute(self);
                        pins.write = true;
                    }
                    5 => {
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                    _ => panic!(),
                }
            }
            Instruction::AbsIdxX(AbsIdxInstruction::ReadModifyWrite(
                read_modify_write_instruction,
            )) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    self.temp = pins.data;
                    pins.addr = self.pc;
                }
                3 => {
                    self.pc += 1;
                    let addr = (pins.data as u16) << 8 | (self.temp as u16);
                    pins.addr = addr
                        + match self.inst {
                            Instruction::AbsIdxX(_) => self.x,
                            Instruction::AbsIdxY(_) => self.y,
                            _ => unreachable!(),
                        } as u16;
                    if (addr & 0xFF00) != (pins.addr & 0xff00) {
                        self.temp = 1;
                    } else {
                        self.temp = 0;
                    }
                }
                4 => {
                    if self.temp != 0 {
                        pins.addr += 0x100;
                    }
                }
                5 => {
                    self.temp = pins.data;
                    pins.write = true;
                }
                6 => {
                    pins.data = read_modify_write_instruction.execute(self, self.temp);
                    pins.write = true;
                }
                7 => {
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::AbsIdxY(AbsIdxInstruction::ReadModifyWrite(_)) => {
                unreachable!()
            }
            Instruction::Rel(RelInstruction::Branch(branch_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    self.temp = pins.data;
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
                    pins.addr = self.pc;
                }
                3 => {
                    if self.temp == 0 {
                        self.step = 0;
                    }
                    pins.addr = self.pc;
                }
                4 => {
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::IdxInd(IdxIndInstruction::Read(read_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = pins.data as u16;
                }
                3 => {
                    pins.addr = pins.addr.wrapping_add(self.x as u16);
                    pins.addr &= 0x00FF;
                }
                4 => {
                    self.temp = pins.data;
                    pins.addr = pins.addr.wrapping_add(1);
                    pins.addr &= 0x00FF;
                }
                5 => {
                    pins.addr = (pins.data as u16) << 8 | self.temp as u16;
                }
                6 => {
                    read_instruction.execute(self, pins.data);
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::IdxInd(IdxIndInstruction::Write(write_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = pins.data as u16;
                }
                3 => {
                    pins.addr = pins.addr.wrapping_add(self.x as u16);
                    pins.addr &= 0x00FF;
                }
                4 => {
                    self.temp = pins.data;
                    pins.addr = pins.addr.wrapping_add(1);
                    pins.addr &= 0x00FF;
                }
                5 => {
                    pins.addr = (pins.data as u16) << 8 | self.temp as u16;
                    pins.data = write_instruction.execute(self);
                    pins.write = true;
                }
                6 => {
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::IndIdx(IndIdxInstruction::Read(read_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = pins.data as u16;
                }
                3 => {
                    self.temp = pins.data; // ADL
                    pins.addr = pins.addr.wrapping_add(1) & 0x00FF;
                }
                4 => {
                    let adl_idx = self.temp.wrapping_add(self.y);
                    pins.addr = (pins.data as u16) << 8 | adl_idx as u16;
                }
                5 => {
                    let adl_idx = (pins.addr & 0x00FF) as u8;
                    if adl_idx < self.temp {
                        pins.addr += 0x100;
                    } else {
                        read_instruction.execute(self, pins.data);
                        pins.addr = self.pc;
                        self.step = 0;
                    }
                }
                6 => {
                    read_instruction.execute(self, pins.data);
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::IndIdx(IndIdxInstruction::Write(write_instruction)) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    pins.addr = pins.data as u16;
                }
                3 => {
                    self.temp = pins.data; // ADL
                    pins.addr = pins.addr.wrapping_add(1) & 0x00FF;
                }
                4 => {
                    let adl_idx = self.temp.wrapping_add(self.y);
                    pins.addr = (pins.data as u16) << 8 | adl_idx as u16;
                }
                5 => {
                    let adl_idx = (pins.addr & 0x00FF) as u8;
                    if adl_idx < self.temp {
                        pins.addr += 0x100;
                    }
                    pins.data = write_instruction.execute(self);
                    pins.write = true;
                }
                6 => {
                    pins.addr = self.pc;
                    self.step = 0;
                }
                _ => panic!(),
            },
            Instruction::AbsInd(_) => match self.step {
                1 => {
                    self.pc += 1;
                    pins.addr = self.pc;
                }
                2 => {
                    self.pc += 1;
                    self.temp = pins.data;
                    pins.addr = self.pc;
                }
                3 => {
                    self.pc += 1;
                    pins.addr = (pins.data as u16) << 8 | self.temp as u16;
                }
                4 => {
                    self.temp = pins.data;
                    let adh = pins.addr & 0xFF00;
                    let adl = (pins.addr & 0x00FF) as u8;
                    let adl = adl.wrapping_add(1);
                    pins.addr = adh | adl as u16;
                }
                5 => {
                    self.pc = (pins.data as u16) << 8 | self.temp as u16;
                    self.step = 0;
                    pins.addr = self.pc;
                }
                _ => panic!(),
            },
            Instruction::Invalid(op) => panic!("Invalid Instruction {op:#04x}"),
        };
    }
}

#[derive(Default)]
pub struct Pins {
    addr: u16,
    data: u8,
    write: bool,
    rst: bool,
    nmi: bool,
    irq: bool,
}

impl Pins {
    pub fn new() -> Self {
        Pins::default()
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
        let mut pins = Pins::new();

        cpu.pc = 0x400;
        cpu.step = 0;
        pins.rst = false;
        pins.nmi = false;
        pins.irq = false;
        pins.addr = 0x400;
        pins.data = ram[0x400];

        for _ in 0u64..96241364 {
            cpu.clock(&mut pins);
            if pins.write {
                ram[pins.addr as usize] = pins.data;
            } else {
                pins.data = ram[pins.addr as usize];
            }
        }

        assert_eq!(cpu.pc, 0x3469);
    }
}
