use super::Cpu;

#[derive(Debug, Clone, Copy)]
pub(super) enum Instruction {
    Stack(StackInstruction),
    AccumImpl(AccumImplInstruction),
    Imm(ImmInstruction),
    Abs(AbsInstruction),
    ZeroPage(ZeroPageInstruction),
    ZeroPageIdxX(ZeroPageIdxInstruction),
    ZeroPageIdxY(ZeroPageIdxInstruction),
    AbsIdxX(AbsIdxInstruction),
    AbsIdxY(AbsIdxInstruction),
    Rel(RelInstruction),
    IdxInd(IdxIndInstruction),
    IndIdx(IndIdxInstruction),
    AbsInd(AbsIndInstruction),
    Invalid(u8),
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Nop))
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum StackInstruction {
    Brk(Interrupt),
    Rti,
    Rts,
    Pha,
    Php,
    Pla,
    Plp,
    Jsr,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Interrupt {
    Rst,
    Irq,
    Nmi,
    Brk,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum JumpInstruction {
    Jmp,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum BranchInstruction {
    Bcc,
    Bcs,
    Bne,
    Beq,
    Bpl,
    Bmi,
    Bvc,
    Bvs,
}

impl BranchInstruction {
    pub(super) fn execute(&self, cpu: &Cpu) -> bool {
        match self {
            BranchInstruction::Bcc => !cpu.p.c,
            BranchInstruction::Bcs => cpu.p.c,
            BranchInstruction::Bne => !cpu.p.z,
            BranchInstruction::Beq => cpu.p.z,
            BranchInstruction::Bpl => !cpu.p.n,
            BranchInstruction::Bmi => cpu.p.n,
            BranchInstruction::Bvc => !cpu.p.v,
            BranchInstruction::Bvs => cpu.p.v,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum InternalInstruction {
    Txa,
    Txs,
    Tax,
    Tsx,
    Tay,
    Tya,
    Dex,
    Dey,
    Clc,
    Sec,
    Cli,
    Sei,
    Cld,
    Sed,
    Clv,
    Inx,
    Iny,
    Nop,
}

impl InternalInstruction {
    pub(super) fn execute(&self, cpu: &mut Cpu) {
        match self {
            InternalInstruction::Txa => {
                cpu.a = cpu.x;
                cpu.p.set_n(cpu.a);
                cpu.p.set_z(cpu.a);
            }
            InternalInstruction::Txs => {
                cpu.s = cpu.x;
            }
            InternalInstruction::Tax => {
                cpu.x = cpu.a;
                cpu.p.set_n(cpu.x);
                cpu.p.set_z(cpu.x);
            }
            InternalInstruction::Tsx => {
                cpu.x = cpu.s;
                cpu.p.set_n(cpu.x);
                cpu.p.set_z(cpu.x);
            }
            InternalInstruction::Tay => {
                cpu.y = cpu.a;
                cpu.p.set_n(cpu.y);
                cpu.p.set_z(cpu.y);
            }
            InternalInstruction::Tya => {
                cpu.a = cpu.y;
                cpu.p.set_n(cpu.a);
                cpu.p.set_z(cpu.a);
            }
            InternalInstruction::Dex => {
                cpu.x = cpu.x.wrapping_sub(1);
                cpu.p.set_n(cpu.x);
                cpu.p.set_z(cpu.x);
            }
            InternalInstruction::Dey => {
                cpu.y = cpu.y.wrapping_sub(1);
                cpu.p.set_n(cpu.y);
                cpu.p.set_z(cpu.y);
            }
            InternalInstruction::Clc => {
                cpu.p.c = false;
            }
            InternalInstruction::Sec => {
                cpu.p.c = true;
            }
            InternalInstruction::Cli => {
                cpu.p.i = false;
            }
            InternalInstruction::Sei => {
                cpu.p.i = true;
            }
            InternalInstruction::Cld => {
                cpu.p.d = false;
            }
            InternalInstruction::Sed => {
                cpu.p.d = true;
            }
            InternalInstruction::Clv => {
                cpu.p.v = false;
            }
            InternalInstruction::Inx => {
                cpu.x = cpu.x.wrapping_add(1);
                cpu.p.set_n(cpu.x);
                cpu.p.set_z(cpu.x);
            }
            InternalInstruction::Iny => {
                cpu.y = cpu.y.wrapping_add(1);
                cpu.p.set_n(cpu.y);
                cpu.p.set_z(cpu.y);
            }
            InternalInstruction::Nop => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ReadInstruction {
    Lda,
    Ldx,
    Ldy,
    Eor,
    And,
    Ora,
    Adc,
    Sbc,
    Cmp,
    Cpy,
    Cpx,
    Bit,
}

impl ReadInstruction {
    pub(super) fn execute(&self, cpu: &mut Cpu, m: u8) {
        match self {
            ReadInstruction::Lda => {
                cpu.a = m;
                cpu.p.set_n(cpu.a);
                cpu.p.set_z(cpu.a);
            }
            ReadInstruction::Ldx => {
                cpu.x = m;
                cpu.p.set_n(cpu.x);
                cpu.p.set_z(cpu.x);
            }
            ReadInstruction::Ldy => {
                cpu.y = m;
                cpu.p.set_n(cpu.y);
                cpu.p.set_z(cpu.y);
            }
            ReadInstruction::Eor => {
                cpu.a ^= m;
                cpu.p.set_n(cpu.a);
                cpu.p.set_z(cpu.a);
            }
            ReadInstruction::And => {
                cpu.a &= m;
                cpu.p.set_n(cpu.a);
                cpu.p.set_z(cpu.a);
            }
            ReadInstruction::Ora => {
                cpu.a |= m;
                cpu.p.set_n(cpu.a);
                cpu.p.set_z(cpu.a);
            }
            ReadInstruction::Adc => {
                let c = if cpu.p.c { 1u8 } else { 0u8 };
                let a = cpu.a;
                if cpu.p.d {
                    let al = (a & 0x0F) as u16 + (m & 0x0F) as u16 + c as u16;
                    let al = al + if al > 9 { 6 } else { 0 };
                    let ah = (a >> 4) as u16 + (m >> 4) as u16 + (if al > 0x0F { 1 } else { 0 });
                    cpu.p.set_z(a.wrapping_add(m).wrapping_add(c));
                    cpu.p.n = ah & 0x08 != 0;
                    cpu.p.v = ((ah << 4) ^ a as u16) & 0x80 != 0 && ((a ^ m) & 0x80 == 0);
                    let ah = ah + if ah > 9 { 6 } else { 0 };
                    cpu.p.c = ah > 0x0F;
                    cpu.a = (ah << 4) as u8 | (al & 0x0F) as u8;
                } else {
                    cpu.a = a.wrapping_add(m).wrapping_add(c);
                    cpu.p.set_c(a as u16 + m as u16 + c as u16);
                    cpu.p.set_v(a, m, c);
                    cpu.p.set_n(cpu.a);
                    cpu.p.set_z(cpu.a);
                }
            }
            ReadInstruction::Sbc => {
                let c = if cpu.p.c { 1u8 } else { 0u8 };
                let a = cpu.a;
                if cpu.p.d {
                    let al = ((a & 0x0F) as u16)
                        .wrapping_sub((m & 0x0F) as u16)
                        .wrapping_sub((1 - c) as u16);
                    let al = al - if al & 0x10 != 0 { 6 } else { 0 };
                    let ah = ((a >> 4) as u16)
                        .wrapping_sub((m >> 4) as u16)
                        .wrapping_sub(if al & 0x10 != 0 { 1 } else { 0 });
                    let ah = ah - if ah & 0x10 != 0 { 6 } else { 0 };
                    cpu.p.set_c(a as u16 + !m as u16 + c as u16);
                    cpu.p.set_v(a, !m, c);
                    cpu.a = (ah << 4) as u8 | (al & 0x0F) as u8;
                    cpu.p.set_n(cpu.a);
                    cpu.p.set_z(cpu.a);
                } else {
                    let m = !m;
                    cpu.a = a.wrapping_add(m).wrapping_add(c);
                    cpu.p.set_c(a as u16 + m as u16 + c as u16);
                    cpu.p.set_v(a, m, c);
                    cpu.p.set_n(cpu.a);
                    cpu.p.set_z(cpu.a);
                }
            }
            ReadInstruction::Cmp => {
                let c = 1u8;
                let m = !m;
                let a = cpu.a.wrapping_add(m).wrapping_add(c);

                cpu.p.set_c(cpu.a as u16 + m as u16 + c as u16);
                cpu.p.set_n(a);
                cpu.p.set_z(a);
            }
            ReadInstruction::Cpy => {
                let c = 1u8;
                let m = !m;
                let y = cpu.y.wrapping_add(m).wrapping_add(c);

                cpu.p.set_c(cpu.y as u16 + m as u16 + c as u16);
                cpu.p.set_n(y);
                cpu.p.set_z(y);
            }
            ReadInstruction::Cpx => {
                let c = 1u8;
                let m = !m;
                let x = cpu.x.wrapping_add(m).wrapping_add(c);

                cpu.p.set_c(cpu.x as u16 + m as u16 + c as u16);
                cpu.p.set_n(x);
                cpu.p.set_z(x);
            }
            ReadInstruction::Bit => {
                cpu.p.set_z(cpu.a & m);
                cpu.p.n = m & (1 << 7) != 0;
                cpu.p.v = m & (1 << 6) != 0;
            }
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum WriteInstruction {
    Sta,
    Stx,
    Sty,
}

impl WriteInstruction {
    pub(super) fn execute(&self, cpu: &mut Cpu) -> u8 {
        match self {
            WriteInstruction::Sta => cpu.a,
            WriteInstruction::Stx => cpu.x,
            WriteInstruction::Sty => cpu.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ReadModifyWriteInstruction {
    Asl,
    Lsr,
    Rol,
    Ror,
    Inc,
    Dec,
}

impl ReadModifyWriteInstruction {
    pub(super) fn execute(&self, cpu: &mut Cpu, m: u8) -> u8 {
        match self {
            ReadModifyWriteInstruction::Asl => {
                let a = m.wrapping_shl(1);
                cpu.p.set_n(a);
                cpu.p.set_z(a);
                cpu.p.c = m & 0x80 != 0;

                a
            }
            ReadModifyWriteInstruction::Lsr => {
                let a = m.wrapping_shr(1);
                cpu.p.set_n(a);
                cpu.p.set_z(a);
                cpu.p.c = m & 0x01 != 0;

                a
            }
            ReadModifyWriteInstruction::Rol => {
                let c = if cpu.p.c { 1u8 } else { 0u8 };
                let a = m.wrapping_shl(1) | c;

                cpu.p.set_n(a);
                cpu.p.set_z(a);
                cpu.p.c = m & (1 << 7) != 0;

                a
            }
            ReadModifyWriteInstruction::Ror => {
                let c = if cpu.p.c { 0x80u8 } else { 0u8 };
                let a = m.wrapping_shr(1) | c;

                cpu.p.set_n(a);
                cpu.p.set_z(a);
                cpu.p.c = m & (1 << 0) != 0;

                a
            }
            ReadModifyWriteInstruction::Inc => {
                let m = m.wrapping_add(1);
                cpu.p.set_n(m);
                cpu.p.set_z(m);
                m
            }
            ReadModifyWriteInstruction::Dec => {
                let m = m.wrapping_sub(1);
                cpu.p.set_n(m);
                cpu.p.set_z(m);
                m
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ImmInstruction {
    Read(ReadInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AccumImplInstruction {
    ReadModifyWrite(ReadModifyWriteInstruction),
    Internal(InternalInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AbsInstruction {
    Jump(JumpInstruction),
    Read(ReadInstruction),
    ReadModifyWrite(ReadModifyWriteInstruction),
    Write(WriteInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ZeroPageInstruction {
    Read(ReadInstruction),
    ReadModifyWrite(ReadModifyWriteInstruction),
    Write(WriteInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ZeroPageIdxInstruction {
    Read(ReadInstruction),
    ReadModifyWrite(ReadModifyWriteInstruction),
    Write(WriteInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AbsIdxInstruction {
    Read(ReadInstruction),
    ReadModifyWrite(ReadModifyWriteInstruction),
    Write(WriteInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum RelInstruction {
    Branch(BranchInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum IdxIndInstruction {
    Read(ReadInstruction),
    Write(WriteInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum IndIdxInstruction {
    Read(ReadInstruction),
    Write(WriteInstruction),
}

#[derive(Debug, Clone, Copy)]
pub(super) enum AbsIndInstruction {
    Jump(JumpInstruction),
}

impl From<u8> for Instruction {
    fn from(opcode: u8) -> Instruction {
        match opcode {
            0x00 => Instruction::Stack(StackInstruction::Brk(Interrupt::Brk)),
            0x01 => Instruction::IdxInd(IdxIndInstruction::Read(ReadInstruction::Ora)),
            0x05 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Ora)),
            0x06 => Instruction::ZeroPage(ZeroPageInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Asl,
            )),
            0x08 => Instruction::Stack(StackInstruction::Php),
            0x09 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Ora)),
            0x0A => Instruction::AccumImpl(AccumImplInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Asl,
            )),
            0x0D => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Ora)),
            0x0E => Instruction::Abs(AbsInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Asl,
            )),
            0x10 => Instruction::Rel(RelInstruction::Branch(BranchInstruction::Bpl)),
            0x11 => Instruction::IndIdx(IndIdxInstruction::Read(ReadInstruction::Ora)),
            0x15 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(ReadInstruction::Ora)),
            0x16 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Asl,
            )),
            0x18 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Clc))
            }
            0x19 => Instruction::AbsIdxY(AbsIdxInstruction::Read(ReadInstruction::Ora)),
            0x1D => Instruction::AbsIdxX(AbsIdxInstruction::Read(ReadInstruction::Ora)),
            0x1E => Instruction::AbsIdxX(AbsIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Asl,
            )),
            0x20 => Instruction::Stack(StackInstruction::Jsr),
            0x21 => Instruction::IdxInd(IdxIndInstruction::Read(ReadInstruction::And)),
            0x24 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Bit)),
            0x25 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::And)),
            0x26 => Instruction::ZeroPage(ZeroPageInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Rol,
            )),
            0x28 => Instruction::Stack(StackInstruction::Plp),
            0x29 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::And)),
            0x2A => Instruction::AccumImpl(AccumImplInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Rol,
            )),
            0x2C => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Bit)),
            0x2D => Instruction::Abs(AbsInstruction::Read(ReadInstruction::And)),
            0x2E => Instruction::Abs(AbsInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Rol,
            )),
            0x30 => Instruction::Rel(RelInstruction::Branch(BranchInstruction::Bmi)),
            0x31 => Instruction::IndIdx(IndIdxInstruction::Read(ReadInstruction::And)),
            0x35 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(ReadInstruction::And)),
            0x36 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Rol,
            )),
            0x38 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Sec))
            }
            0x39 => Instruction::AbsIdxY(AbsIdxInstruction::Read(ReadInstruction::And)),
            0x3D => Instruction::AbsIdxX(AbsIdxInstruction::Read(ReadInstruction::And)),
            0x3E => Instruction::AbsIdxX(AbsIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Rol,
            )),
            0x40 => Instruction::Stack(StackInstruction::Rti),
            0x41 => Instruction::IdxInd(IdxIndInstruction::Read(ReadInstruction::Eor)),
            0x45 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Eor)),
            0x46 => Instruction::ZeroPage(ZeroPageInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Lsr,
            )),
            0x48 => Instruction::Stack(StackInstruction::Pha),
            0x49 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Eor)),
            0x4A => Instruction::AccumImpl(AccumImplInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Lsr,
            )),
            0x4C => Instruction::Abs(AbsInstruction::Jump(JumpInstruction::Jmp)),
            0x4D => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Eor)),
            0x4E => Instruction::Abs(AbsInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Lsr,
            )),
            0x50 => Instruction::Rel(RelInstruction::Branch(BranchInstruction::Bvc)),
            0x51 => Instruction::IndIdx(IndIdxInstruction::Read(ReadInstruction::Eor)),
            0x55 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(ReadInstruction::Eor)),
            0x56 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Lsr,
            )),
            0x58 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Cli))
            }
            0x59 => Instruction::AbsIdxY(AbsIdxInstruction::Read(ReadInstruction::Eor)),
            0x5D => Instruction::AbsIdxX(AbsIdxInstruction::Read(ReadInstruction::Eor)),
            0x5E => Instruction::AbsIdxX(AbsIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Lsr,
            )),
            0x60 => Instruction::Stack(StackInstruction::Rts),
            0x61 => Instruction::IdxInd(IdxIndInstruction::Read(ReadInstruction::Adc)),
            0x65 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Adc)),
            0x66 => Instruction::ZeroPage(ZeroPageInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Ror,
            )),
            0x68 => Instruction::Stack(StackInstruction::Pla),
            0x69 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Adc)),
            0x6A => Instruction::AccumImpl(AccumImplInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Ror,
            )),
            0x6C => Instruction::AbsInd(AbsIndInstruction::Jump(JumpInstruction::Jmp)),
            0x6D => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Adc)),
            0x6E => Instruction::Abs(AbsInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Ror,
            )),
            0x70 => Instruction::Rel(RelInstruction::Branch(BranchInstruction::Bvs)),
            0x71 => Instruction::IndIdx(IndIdxInstruction::Read(ReadInstruction::Adc)),
            0x75 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(ReadInstruction::Adc)),
            0x76 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Ror,
            )),
            0x78 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Sei))
            }
            0x79 => Instruction::AbsIdxY(AbsIdxInstruction::Read(ReadInstruction::Adc)),
            0x7D => Instruction::AbsIdxX(AbsIdxInstruction::Read(ReadInstruction::Adc)),
            0x7E => Instruction::AbsIdxX(AbsIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Ror,
            )),
            0x81 => Instruction::IdxInd(IdxIndInstruction::Write(WriteInstruction::Sta)),
            0x84 => Instruction::ZeroPage(ZeroPageInstruction::Write(WriteInstruction::Sty)),
            0x85 => Instruction::ZeroPage(ZeroPageInstruction::Write(WriteInstruction::Sta)),
            0x86 => Instruction::ZeroPage(ZeroPageInstruction::Write(WriteInstruction::Stx)),
            0x88 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Dey))
            }
            0x8A => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Txa))
            }
            0x8C => Instruction::Abs(AbsInstruction::Write(WriteInstruction::Sty)),
            0x8D => Instruction::Abs(AbsInstruction::Write(WriteInstruction::Sta)),
            0x8E => Instruction::Abs(AbsInstruction::Write(WriteInstruction::Stx)),
            0x90 => Instruction::Rel(RelInstruction::Branch(BranchInstruction::Bcc)),
            0x91 => Instruction::IndIdx(IndIdxInstruction::Write(WriteInstruction::Sta)),
            0x94 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Write(WriteInstruction::Sty)),
            0x95 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Write(WriteInstruction::Sta)),
            0x96 => Instruction::ZeroPageIdxY(ZeroPageIdxInstruction::Write(WriteInstruction::Stx)),
            0x98 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Tya))
            }
            0x99 => Instruction::AbsIdxY(AbsIdxInstruction::Write(WriteInstruction::Sta)),
            0x9A => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Txs))
            }
            0x9D => Instruction::AbsIdxX(AbsIdxInstruction::Write(WriteInstruction::Sta)),
            0xA0 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Ldy)),
            0xA1 => Instruction::IdxInd(IdxIndInstruction::Read(ReadInstruction::Lda)),
            0xA2 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Ldx)),
            0xA4 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Ldy)),
            0xA5 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Lda)),
            0xA6 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Ldx)),
            0xA8 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Tay))
            }
            0xA9 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Lda)),
            0xAA => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Tax))
            }
            0xAC => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Ldy)),
            0xAD => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Lda)),
            0xAE => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Ldx)),
            0xB0 => Instruction::Rel(RelInstruction::Branch(BranchInstruction::Bcs)),
            0xB1 => Instruction::IndIdx(IndIdxInstruction::Read(ReadInstruction::Lda)),
            0xB4 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(ReadInstruction::Ldy)),
            0xB5 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(ReadInstruction::Lda)),
            0xB6 => Instruction::ZeroPageIdxY(ZeroPageIdxInstruction::Read(ReadInstruction::Ldx)),
            0xB8 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Clv))
            }
            0xB9 => Instruction::AbsIdxY(AbsIdxInstruction::Read(ReadInstruction::Lda)),
            0xBA => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Tsx))
            }
            0xBC => Instruction::AbsIdxX(AbsIdxInstruction::Read(ReadInstruction::Ldy)),
            0xBD => Instruction::AbsIdxX(AbsIdxInstruction::Read(ReadInstruction::Lda)),
            0xBE => Instruction::AbsIdxY(AbsIdxInstruction::Read(ReadInstruction::Ldx)),
            0xC0 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Cpy)),
            0xC1 => Instruction::IdxInd(IdxIndInstruction::Read(ReadInstruction::Cmp)),
            0xC4 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Cpy)),
            0xC5 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Cmp)),
            0xC6 => Instruction::ZeroPage(ZeroPageInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Dec,
            )),
            0xC8 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Iny))
            }
            0xC9 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Cmp)),
            0xCA => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Dex))
            }
            0xCC => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Cpy)),
            0xCD => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Cmp)),
            0xCE => Instruction::Abs(AbsInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Dec,
            )),
            0xD0 => Instruction::Rel(RelInstruction::Branch(BranchInstruction::Bne)),
            0xD1 => Instruction::IndIdx(IndIdxInstruction::Read(ReadInstruction::Cmp)),
            0xD5 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(ReadInstruction::Cmp)),
            0xD6 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Dec,
            )),
            0xD8 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Cld))
            }
            0xD9 => Instruction::AbsIdxY(AbsIdxInstruction::Read(ReadInstruction::Cmp)),
            0xDD => Instruction::AbsIdxX(AbsIdxInstruction::Read(ReadInstruction::Cmp)),
            0xDE => Instruction::AbsIdxX(AbsIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Dec,
            )),
            0xE0 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Cpx)),
            0xE1 => Instruction::IdxInd(IdxIndInstruction::Read(ReadInstruction::Sbc)),
            0xE4 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Cpx)),
            0xE5 => Instruction::ZeroPage(ZeroPageInstruction::Read(ReadInstruction::Sbc)),
            0xE6 => Instruction::ZeroPage(ZeroPageInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Inc,
            )),
            0xE8 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Inx))
            }
            0xE9 => Instruction::Imm(ImmInstruction::Read(ReadInstruction::Sbc)),
            0xEA => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Nop))
            }
            0xEC => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Cpx)),
            0xED => Instruction::Abs(AbsInstruction::Read(ReadInstruction::Sbc)),
            0xEE => Instruction::Abs(AbsInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Inc,
            )),
            0xF0 => Instruction::Rel(RelInstruction::Branch(BranchInstruction::Beq)),
            0xF1 => Instruction::IndIdx(IndIdxInstruction::Read(ReadInstruction::Sbc)),
            0xF5 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::Read(ReadInstruction::Sbc)),
            0xF6 => Instruction::ZeroPageIdxX(ZeroPageIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Inc,
            )),
            0xF8 => {
                Instruction::AccumImpl(AccumImplInstruction::Internal(InternalInstruction::Sed))
            }
            0xF9 => Instruction::AbsIdxY(AbsIdxInstruction::Read(ReadInstruction::Sbc)),
            0xFD => Instruction::AbsIdxX(AbsIdxInstruction::Read(ReadInstruction::Sbc)),
            0xFE => Instruction::AbsIdxX(AbsIdxInstruction::ReadModifyWrite(
                ReadModifyWriteInstruction::Inc,
            )),
            op => Instruction::Invalid(op),
        }
    }
}
