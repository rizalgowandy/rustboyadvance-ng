use crate::bit::BitIndex;
use crate::num_traits::FromPrimitive;
use std::convert::TryFrom;

pub use super::display;

#[derive(Debug, PartialEq)]
pub enum ArmError {
    UnknownInstructionFormat(u32),
    UndefinedConditionCode(u32),
    InvalidShiftType(u32),
}

#[derive(Debug, Copy, Clone, PartialEq, Primitive)]
pub enum ArmCond {
    Equal = 0b0000,
    NotEqual = 0b0001,
    UnsignedHigherOrSame = 0b0010,
    UnsignedLower = 0b0011,
    Negative = 0b0100,
    PositiveOrZero = 0b0101,
    Overflow = 0b0110,
    NoOverflow = 0b0111,
    UnsignedHigher = 0b1000,
    UnsignedLowerOrSame = 0b1001,
    GreaterOrEqual = 0b1010,
    LessThan = 0b1011,
    GreaterThan = 0b1100,
    LessThanOrEqual = 0b1101,
    Always = 0b1110,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ArmInstructionFormat {
    // Branch and Exchange
    BX,
    // Branch /w Link
    B_BL,
    // Multiply and Multiply-Accumulate
    MUL_MLA,
    // Multiply Long and Multiply-Accumulate Long
    MULL_MLAL,
    // Single Data Transfer
    LDR_STR,
    // Halfword and Signed Data Transfer
    LDR_STR_HS_REG,
    // Halfword and Signed Data Transfer
    LDR_STR_HS_IMM,
    // Data Processing
    DP,
    // Block Data Transfer
    LDM_STM,
    // Single Data Swap
    SWP,
    // Transfer PSR contents to a register
    MRS,
    // Transfer register contents to PSR
    MSR_REG,
    // Tanssfer immediate/register to PSR flags only
    MSR_FLAGS,
}

#[derive(Debug, Primitive)]
pub enum ArmOpCode {
    AND = 0b0000,
    EOR = 0b0001,
    SUB = 0b0010,
    RSB = 0b0011,
    ADD = 0b0100,
    ADC = 0b0101,
    SBC = 0b0110,
    RSC = 0b0111,
    TST = 0b1000,
    TEQ = 0b1001,
    CMP = 0b1010,
    CMN = 0b1011,
    ORR = 0b1100,
    MOV = 0b1101,
    BIC = 0b1110,
    MVN = 0b1111,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ArmInstruction {
    pub cond: ArmCond,
    pub fmt: ArmInstructionFormat,
    pub raw: u32,
    pub pc: u32,
}

impl TryFrom<(u32, u32)> for ArmInstruction {
    type Error = ArmError;

    fn try_from(value: (u32, u32)) -> Result<Self, Self::Error> {
        use ArmInstructionFormat::*;
        let (raw, addr) = value;

        let cond_code = raw.bit_range(28..32) as u8;
        let cond = match ArmCond::from_u8(cond_code) {
            Some(cond) => Ok(cond),
            None => Err(ArmError::UndefinedConditionCode(cond_code as u32)),
        }?;

        let fmt = if (0x0fff_fff0 & raw) == 0x012f_ff10 {
            Ok(BX)
        } else if (0x0e00_0000 & raw) == 0x0a00_0000 {
            Ok(B_BL)
        } else if (0x0fc0_00f0 & raw) == 0x0000_0090 {
            Ok(MUL_MLA)
        } else if (0x0f80_00f0 & raw) == 0x0080_0090 {
            Ok(MULL_MLAL)
        } else if (0x0c00_0000 & raw) == 0x0400_0000 {
            Ok(LDR_STR)
        } else if (0x0e40_0F90 & raw) == 0x0000_0090 {
            Ok(LDR_STR_HS_REG)
        } else if (0x0e40_0090 & raw) == 0x0040_0090 {
            Ok(LDR_STR_HS_IMM)
        } else if (0x0e00_0000 & raw) == 0x0800_0000 {
            Ok(LDM_STM)
        } else if (0x0fb0_0ff0 & raw) == 0x0100_0090 {
            Ok(SWP)
        } else if (0x0fbf_0fff & raw) == 0x010f_0000 {
            Ok(MRS)
        } else if (0x0fbf_fff0 & raw) == 0x0129_f000 {
            Ok(MSR_REG)
        } else if (0x0dbf_f000 & raw) == 0x0128_f000 {
            Ok(MSR_FLAGS)
        } else if (0x0fb0_0ff0 & raw) == 0x0100_0090 {
            Ok(SWP)
        } else if (0x0c00_0000 & raw) == 0x0000_0000 {
            Ok(DP)
        } else {
            Err(ArmError::UnknownInstructionFormat(raw))
        }?;

        Ok(ArmInstruction {
            cond: cond,
            fmt: fmt,
            raw: raw,
            pc: addr,
        })
    }
}

#[derive(Debug, PartialEq, Primitive)]
pub enum ArmShiftType {
    LSL = 0,
    LSR = 1,
    ASR = 2,
    ROR = 3,
}

#[derive(Debug, PartialEq)]
pub enum ArmShift {
    ImmediateShift(u32, ArmShiftType),
    RegisterShift(usize, ArmShiftType),
}

impl TryFrom<u32> for ArmShift {
    type Error = ArmError;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        let typ = match ArmShiftType::from_u8(v.bit_range(5..7) as u8) {
            Some(s) => Ok(s),
            _ => Err(ArmError::InvalidShiftType(v.bit_range(5..7))),
        }?;
        if v.bit(4) {
            let rs = v.bit_range(8..12) as usize;
            Ok(ArmShift::RegisterShift(rs, typ))
        } else {
            let amount = v.bit_range(7..12) as u32;
            Ok(ArmShift::ImmediateShift(amount, typ))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ArmInstructionShiftValue {
    ImmediateValue(u32),
    RotatedImmediate(u32, u32),
    ShiftedRegister(usize, ArmShift),
}

impl ArmInstructionShiftValue {
    /// Decode operand2 as an immediate value
    pub fn decode_rotated_immediate(&self) -> Option<i32> {
        if let ArmInstructionShiftValue::RotatedImmediate(immediate, rotate) = self {
            return Some(immediate.rotate_right(*rotate) as i32);
        }
        None
    }
}

impl ArmInstruction {
    pub fn rn(&self) -> usize {
        match self.fmt {
            ArmInstructionFormat::MUL_MLA => self.raw.bit_range(12..16) as usize,
            ArmInstructionFormat::MULL_MLAL => self.raw.bit_range(8..12) as usize,
            ArmInstructionFormat::BX => self.raw.bit_range(0..4) as usize,
            _ => self.raw.bit_range(16..20) as usize,
        }
    }

    pub fn rd(&self) -> usize {
        match self.fmt {
            ArmInstructionFormat::MUL_MLA => self.raw.bit_range(16..20) as usize,
            _ => self.raw.bit_range(12..16) as usize,
        }
    }

    pub fn rm(&self) -> usize {
        self.raw.bit_range(0..4) as usize
    }
    
    pub fn opcode(&self) -> Option<ArmOpCode> {
        ArmOpCode::from_u32(self.raw.bit_range(21..25))
    }

    pub fn branch_offset(&self) -> i32 {
        ((((self.raw << 8) as i32) >> 8) << 2) + 8
    }

    pub fn is_load(&self) -> bool {
        self.raw.bit(20)
    }

    pub fn is_set_cond(&self) -> bool {
        self.raw.bit(20)
    }

    pub fn is_write_back(&self) -> bool {
        self.raw.bit(21)
    }

    pub fn transfer_size(&self) -> usize {
        if self.raw.bit(22) {
            1
        } else {
            4
        }
    }

    pub fn is_loading_psr_and_forcing_user_mode(&self) -> bool {
        self.raw.bit(22)
    }

    pub fn is_spsr(&self) -> bool {
        self.raw.bit(22)
    }

    pub fn is_ofs_added(&self) -> bool {
        self.raw.bit(23)
    }

    pub fn is_pre_indexing(&self) -> bool {
        self.raw.bit(24)
    }

    pub fn is_linked_branch(&self) -> bool {
        self.raw.bit(24)
    }

    pub fn offset(&self) -> ArmInstructionShiftValue {
        let ofs = self.raw.bit_range(0..12);
        if self.raw.bit(25) {
            let rm = ofs & 0xf;
            let shift = ArmShift::try_from(ofs).unwrap();
            ArmInstructionShiftValue::ShiftedRegister(rm as usize, shift)
        } else {
            ArmInstructionShiftValue::ImmediateValue(ofs)
        }
    }

    pub fn operand2(&self) -> ArmInstructionShiftValue {
        let op2 = self.raw.bit_range(0..12);
        if self.raw.bit(25) {
            let immediate = op2 & 0xff;
            let rotate = 2 * op2.bit_range(8..12);
            ArmInstructionShiftValue::RotatedImmediate(immediate, rotate)
        } else {
            let reg = op2 & 0xf;
            let shift = ArmShift::try_from(op2).unwrap(); // TODO error handling
            ArmInstructionShiftValue::ShiftedRegister(reg as usize, shift)
        }
    }

    pub fn register_list(&self) -> Vec<usize> {
        let list_bits = self.raw & 0xffff;
        let mut list = Vec::with_capacity(16);
        for i in 0..16 {
            if (list_bits & (1 << i)) != 0 {
                list.push(i)
            }
        }
        list
    }
}
