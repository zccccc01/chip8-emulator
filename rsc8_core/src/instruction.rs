use crate::error::InstructionError;

pub enum Instruction {
    Ins00E0,             // 清屏
    Ins00EE,             // 返回
    Ins1NNN(u16),        // 跳转到addr NNN
    Ins2NNN(u16),        // 调用子程序
    Ins3XNN(u8, u8),     // Skip if(VX == 0xNN)
    Ins4XNN(u8, u8),     // Skip if(VX != 0xNN)
    Ins5XY0(u8, u8),     // Skip if(VX == VY)
    Ins6XNN(u8, u8),     // VX = 0xNN
    Ins7XNN(u8, u8),     // VX += 0xNN Doesn't affect carry flag(VF)
    Ins8XY0(u8, u8),     // VX = VY
    Ins8XY1(u8, u8),     // VX |= VY
    Ins8XY2(u8, u8),     // VX &= VY
    Ins8XY3(u8, u8),     // VX ^= VY
    Ins8XY4(u8, u8),     // VX += VY
    Ins8XY5(u8, u8),     // VX -= VY
    Ins8XY6(u8, u8),     // VX >>= 1
    Ins8XY7(u8, u8),     // VX = VY - VX
    Ins8XYE(u8, u8),     // VX <<= 1
    Ins9XY0(u8, u8),     // Skip if(VX != VY)
    InsANNN(u16),        // I = 0xNNN
    InsBNNN(u16),        // 跳转到V0 + 0xNNN
    InsCXNN(u8, u8),     // VX = rand() & 0xNN
    InsDXYN(u8, u8, u8), // 在(VX, VY)绘制N行像素
    InsEX9E(u8),         // Skip if(key index in VX is pressed)
    InsEXA1(u8),         //	Skip if(key index in VX isn't pressed)
    InsFX07(u8),         // VX = Delay Timer
    InsFX0A(u8),         //	等待键盘输入, 存储进VX
    InsFX15(u8),         // Delay Timer = VX
    InsFX18(u8),         // Sound Timer = VX
    InsFX1E(u8),         //	I += VX
    InsFX29(u8),         // Set I to address of font character in VX
    InsFX33(u8),         // Stores BCD encoding of VX into I
    InsFX55(u8),         // Stores V0 thru VX into RAM address starting at I
    InsFX65(u8),         // Fills V0 thru VX with RAM values starting at address in I
}

impl Instruction {
    // 分解为四个 4 位(半字节)的部分
    fn nibbles(opcode: u16) -> (u8, u8, u8, u8) {
        (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        )
    }

    // addr
    fn nnn(opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    // immediate value
    fn nn(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }
}

impl TryFrom<u16> for Instruction {
    type Error = InstructionError;

    fn try_from(opcode: u16) -> Result<Self, Self::Error> {
        let (n1, n2, n3, n4) = Instruction::nibbles(opcode);
        match (n1, n2, n3, n4) {
            // cls
            (0x0, 0x0, 0xE, 0x0) => Ok(Instruction::Ins00E0),
            // ret
            (0x0, 0x0, 0xE, 0xE) => Ok(Instruction::Ins00EE),
            // jmp NNN
            (0x1, _, _, _) => Ok(Instruction::Ins1NNN(Instruction::nnn(opcode))),
            // CALL NNN
            (0x2, _, _, _) => Ok(Instruction::Ins2NNN(Instruction::nnn(opcode))),
            // SKIP VX == NN
            (0x3, _, _, _) => Ok(Instruction::Ins3XNN(n2, Instruction::nn(opcode))),
            // SKIP VX != NN
            (0x4, _, _, _) => Ok(Instruction::Ins4XNN(n2, Instruction::nn(opcode))),
            // SKIP VX == VY
            (0x5, _, _, 0x0) => Ok(Instruction::Ins5XY0(n2, n3)),
            // VX = NN
            (0x6, _, _, _) => Ok(Instruction::Ins6XNN(n2, Instruction::nn(opcode))),
            // VX += NN
            (0x7, _, _, _) => Ok(Instruction::Ins7XNN(n2, Instruction::nn(opcode))),
            // VX = VY
            (0x8, _, _, 0x0) => Ok(Instruction::Ins8XY0(n2, n3)),
            // VX |= VY
            (0x8, _, _, 0x1) => Ok(Instruction::Ins8XY1(n2, n3)),
            // VX &= VY
            (0x8, _, _, 0x2) => Ok(Instruction::Ins8XY2(n2, n3)),
            // VX ^= VY
            (0x8, _, _, 0x3) => Ok(Instruction::Ins8XY3(n2, n3)),
            // VX += VY
            (0x8, _, _, 0x4) => Ok(Instruction::Ins8XY4(n2, n3)),
            // VX -= VY
            (0x8, _, _, 0x5) => Ok(Instruction::Ins8XY5(n2, n3)),
            // VX >>= 1
            (0x8, _, _, 0x6) => Ok(Instruction::Ins8XY6(n2, n3)),
            // VX = VY - VX
            (0x8, _, _, 0x7) => Ok(Instruction::Ins8XY7(n2, n3)),
            // VX <<= 1
            (0x8, _, _, 0xE) => Ok(Instruction::Ins8XYE(n2, n3)),
            // SKIP VX != VY
            (0x9, _, _, 0x0) => Ok(Instruction::Ins9XY0(n2, n3)),
            // I = NNN
            (0xA, _, _, _) => Ok(Instruction::InsANNN(Instruction::nnn(opcode))),
            // JMP V0 + NNN
            (0xB, _, _, _) => Ok(Instruction::InsBNNN(Instruction::nnn(opcode))),
            // VX = rand() & NN
            (0xC, _, _, _) => Ok(Instruction::InsCXNN(n2, Instruction::nn(opcode))),
            // DRAW
            (0xD, _, _, _) => Ok(Instruction::InsDXYN(n2, n3, n4)),
            // SKIP KEY PRESS
            (0xE, _, 0x9, 0xE) => Ok(Instruction::InsEX9E(n2)),
            // SKIP KEY RELEASE
            (0xE, _, 0xA, 0x1) => Ok(Instruction::InsEXA1(n2)),
            // VX = DT
            (0xF, _, 0x0, 0x7) => Ok(Instruction::InsFX07(n2)),
            // WAIT KEY
            (0xF, _, 0x0, 0xA) => Ok(Instruction::InsFX0A(n2)),
            // DT = VX
            (0xF, _, 0x1, 0x5) => Ok(Instruction::InsFX15(n2)),
            // ST = VX
            (0xF, _, 0x1, 0x8) => Ok(Instruction::InsFX18(n2)),
            // I += VX
            (0xF, _, 0x1, 0xE) => Ok(Instruction::InsFX1E(n2)),
            // I = FONT
            (0xF, _, 0x2, 0x9) => Ok(Instruction::InsFX29(n2)),
            // BCD
            (0xF, _, 0x3, 0x3) => Ok(Instruction::InsFX33(n2)),
            // STORE V0 - VX
            (0xF, _, 0x5, 0x5) => Ok(Instruction::InsFX55(n2)),
            // LOAD V0 - VX
            (0xF, _, 0x6, 0x5) => Ok(Instruction::InsFX65(n2)),
            // err
            _ => Err(InstructionError::UnknownOpcode(opcode)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_instructions() {
        // 00E0 - CLS
        assert!(matches!(
            Instruction::try_from(0x00E0),
            Ok(Instruction::Ins00E0)
        ));

        // 00EE - RET
        assert!(matches!(
            Instruction::try_from(0x00EE),
            Ok(Instruction::Ins00EE)
        ));
    }

    #[test]
    fn test_jump_and_call() {
        // 1NNN - Jump
        let opcode = 0x1123;
        if let Ok(Instruction::Ins1NNN(addr)) = Instruction::try_from(opcode) {
            assert_eq!(addr, 0x123);
        } else {
            panic!("1NNN decode failed");
        }

        // 2NNN - Call
        let opcode = 0x2FFF;
        if let Ok(Instruction::Ins2NNN(addr)) = Instruction::try_from(opcode) {
            assert_eq!(addr, 0xFFF);
        } else {
            panic!("2NNN decode failed");
        }
    }

    #[test]
    fn test_skip_instructions() {
        // 3XNN - Skip if VX == NN
        let opcode = 0x3AFF;
        if let Ok(Instruction::Ins3XNN(x, nn)) = Instruction::try_from(opcode) {
            assert_eq!(x, 0xA);
            assert_eq!(nn, 0xFF);
        }

        // 5XY0 - Skip if VX == VY
        let valid_opcode = 0x5AB0;
        assert!(matches!(
            Instruction::try_from(valid_opcode),
            Ok(Instruction::Ins5XY0(0xA, 0xB))
        ));

        // 应该失败的非标准 5XY1
        let invalid_opcode = 0x5CD1;
        assert!(matches!(
            Instruction::try_from(invalid_opcode),
            Err(InstructionError::UnknownOpcode(0x5CD1))
        ));
    }

    #[test]
    fn test_arithmetic_instructions() {
        // 8XY4 - ADD
        let opcode = 0x8CD4;
        if let Ok(Instruction::Ins8XY4(x, y)) = Instruction::try_from(opcode) {
            assert_eq!(x, 0xC);
            assert_eq!(y, 0xD);
        }

        // 8XY6 - SHR
        let opcode = 0x8EF6;
        assert!(matches!(
            Instruction::try_from(opcode),
            Ok(Instruction::Ins8XY6(0xE, 0xF))
        ));
    }

    #[test]
    fn test_fx_instructions() {
        // FX18 - Sound timer
        let opcode = 0xF718;
        assert!(matches!(
            Instruction::try_from(opcode),
            Ok(Instruction::InsFX18(0x7))
        ));

        // FX55 - Store memory
        let opcode = 0xFA55;
        assert!(matches!(
            Instruction::try_from(opcode),
            Ok(Instruction::InsFX55(0xA))
        ));
    }

    #[test]
    fn test_special_instructions() {
        // DXYN - Draw
        let opcode = 0xD123;
        if let Ok(Instruction::InsDXYN(x, y, n)) = Instruction::try_from(opcode) {
            assert_eq!(x, 0x1);
            assert_eq!(y, 0x2);
            assert_eq!(n, 0x3);
        }

        // EXA1 - Skip if key not pressed
        let opcode = 0xEBA1;
        assert!(matches!(
            Instruction::try_from(opcode),
            Ok(Instruction::InsEXA1(0xB))
        ));
    }

    #[test]
    fn test_invalid_opcodes() {
        // 未知指令
        assert!(matches!(
            Instruction::try_from(0x0001),
            Err(InstructionError::UnknownOpcode(0x0001))
        ));

        // 非法 0x0 前缀指令
        assert!(matches!(
            Instruction::try_from(0x00F1),
            Err(InstructionError::UnknownOpcode(0x00F1))
        ));

        // 非法 8XY 格式
        assert!(matches!(
            Instruction::try_from(0x8AB9),
            Err(InstructionError::UnknownOpcode(0x8AB9))
        ));
    }

    #[test]
    fn test_edge_cases() {
        // 最大地址测试
        let _max_addr = 0xFFFF;
        assert!(matches!(
            Instruction::try_from(0xAFFF),
            Ok(Instruction::InsANNN(0xFFF))
        ));
        assert!(matches!(
            Instruction::try_from(0xBFFF),
            Ok(Instruction::InsBNNN(0xFFF))
        ));

        // 寄存器边界值
        let opcode = 0x6F0F; // VF = 0x0F
        if let Ok(Instruction::Ins6XNN(x, nn)) = Instruction::try_from(opcode) {
            assert_eq!(x, 0xF);
            assert_eq!(nn, 0x0F);
        }
    }
}
