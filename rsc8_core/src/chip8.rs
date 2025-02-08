use crate::{error::InstructionError, instruction::Instruction};

pub const MEMORY_SIZE: usize = 4096;
pub const NUM_REGISTERS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const PROGRAM_START: u16 = 0x200;
pub const ROM_START: usize = 512;
// 字符集
pub const FONTSET_START: usize = 0;
pub const FONTSET_SIZE: usize = 80;

pub const KEYPAD_SIZE: usize = 16;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

// 每行是 8 位(一个 u8), CHIP-8 只用前 4 位
// 0010 0000
// 0110 0000
// 0010 0000
// 0010 0000
// 0111 0000 数字1 组成的地方, 肉眼看着像1
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Chip8<R>
where
    R: Iterator<Item = u16>,
{
    pub memory: [u8; MEMORY_SIZE],
    pub pc: u16,
    pub v_reg: [u8; NUM_REGISTERS],
    pub i_reg: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack: [u16; STACK_SIZE],
    pub stack_pointer: u16,
    pub keypad: [bool; KEYPAD_SIZE],
    pub screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub draw_flag: bool,
    pub rng: R, // 随机数生成器
    pub wait_for_key_release: Option<usize>,
}

impl<R> Chip8<R>
where
    R: Iterator<Item = u16>,
{
    pub fn new(rng: R) -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            pc: PROGRAM_START,
            v_reg: [0; NUM_REGISTERS],
            i_reg: 0,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            keypad: [false; KEYPAD_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            draw_flag: false,
            rng,
            wait_for_key_release: None,
        }
    }

    pub fn load_fontset(&mut self) {
        self.memory[..FONTSET.len()].copy_from_slice(&FONTSET);
    }

    pub fn load_rom(&mut self, buf: &[u8]) {
        let rom_end = ROM_START + buf.len();
        self.memory[ROM_START..rom_end].copy_from_slice(buf);
    }

    pub fn tick(&mut self) -> Result<(), InstructionError> {
        let opcode = self.fetch_opcode();
        let instruction = Instruction::try_from(opcode)?;
        self.execute_instruction(&instruction);
        Ok(())
    }

    pub fn tick_timer(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keypad[idx] = pressed;
    }

    // |   |
    // | h | 0xA2 -> 左移8位 0xA200
    // |_l_| 0xF0 -> 按位或  0xA2F0
    // Chip8 大端格式
    fn fetch_opcode(&mut self) -> u16 {
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte = self.memory[self.pc as usize + 1] as u16;

        // Chip8 操作码都是 2 字节
        self.pc += 2;

        (high_byte << 8) | low_byte
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn reset(&mut self) {
        self.pc = PROGRAM_START;
        self.memory = [0; MEMORY_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGISTERS];
        self.i_reg = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keypad = [false; KEYPAD_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.draw_flag = false;
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn execute_instruction(&mut self, instruction: &Instruction) {
        match *instruction {
            Instruction::Ins00E0 => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
                self.draw_flag = true;
            }
            Instruction::Ins00EE => {
                self.stack_pointer -= 1;
                self.pc = self.stack[self.stack_pointer as usize];
            }
            Instruction::Ins1NNN(nnn) => {
                self.pc = nnn;
            }
            Instruction::Ins2NNN(nnn) => {
                if self.stack_pointer as usize >= STACK_SIZE {
                    panic!("2NNN failure, stack overflow");
                }
                self.stack[self.stack_pointer as usize] = self.pc;
                self.stack_pointer += 1;
                self.pc = nnn;
            }
            Instruction::Ins3XNN(x, nn) => {
                if self.v_reg[x as usize] == nn {
                    self.pc += 2;
                }
            }
            Instruction::Ins4XNN(x, nn) => {
                if self.v_reg[x as usize] != nn {
                    self.pc += 2;
                }
            }
            Instruction::Ins5XY0(x, y) => {
                if self.v_reg[x as usize] == self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }
            Instruction::Ins6XNN(x, nn) => {
                self.v_reg[x as usize] = nn;
            }
            Instruction::Ins7XNN(x, nn) => {
                // 只保留低 8 位
                self.v_reg[x as usize] = self.v_reg[x as usize].wrapping_add(nn);
            }
            Instruction::Ins8XY0(x, y) => {
                self.v_reg[x as usize] = self.v_reg[y as usize];
            }
            Instruction::Ins8XY1(x, y) => {
                self.v_reg[x as usize] |= self.v_reg[y as usize];
                self.v_reg[0xF] = 0;
            }
            Instruction::Ins8XY2(x, y) => {
                self.v_reg[x as usize] &= self.v_reg[y as usize];
                self.v_reg[0xF] = 0;
            }
            Instruction::Ins8XY3(x, y) => {
                self.v_reg[x as usize] ^= self.v_reg[y as usize];
                self.v_reg[0xF] = 0;
            }
            Instruction::Ins8XY4(x, y) => {
                let (res, carry) = self.v_reg[x as usize].overflowing_add(self.v_reg[y as usize]);
                self.v_reg[x as usize] = res;
                self.v_reg[0xF] = carry as u8;
            }
            Instruction::Ins8XY5(x, y) => {
                let (res, borrow) = self.v_reg[x as usize].overflowing_sub(self.v_reg[y as usize]);
                self.v_reg[x as usize] = res;
                self.v_reg[0xF] = !borrow as u8;
            }
            Instruction::Ins8XY6(x, y) => {
                self.v_reg[x as usize] = self.v_reg[y as usize];
                let dropped = self.v_reg[x as usize] & 1;
                self.v_reg[x as usize] >>= 1;
                self.v_reg[0xF] = dropped;
            }
            Instruction::Ins8XY7(x, y) => {
                let (res, borrow) = self.v_reg[y as usize].overflowing_sub(self.v_reg[x as usize]);
                self.v_reg[x as usize] = res;
                self.v_reg[0xF] = !borrow as u8;
            }
            Instruction::Ins8XYE(x, y) => {
                self.v_reg[x as usize] = self.v_reg[y as usize];
                let dropped = self.v_reg[x as usize] >> 7;
                self.v_reg[x as usize] <<= 1;
                self.v_reg[0xF] = dropped;
            }
            Instruction::Ins9XY0(x, y) => {
                if self.v_reg[x as usize] != self.v_reg[y as usize] {
                    self.pc += 2;
                }
            }
            Instruction::InsANNN(nnn) => {
                self.i_reg = nnn;
            }
            Instruction::InsBNNN(nnn) => {
                self.pc = self.v_reg[0] as u16 + nnn;
            }
            Instruction::InsCXNN(x, nn) => {
                let random = self.rng.next().unwrap_or_default();
                self.v_reg[x as usize] = random as u8 & nn;
            }
            Instruction::InsDXYN(x, y, n) => {
                let vx = self.v_reg[x as usize] % SCREEN_WIDTH as u8;
                let vy = self.v_reg[y as usize] % SCREEN_HEIGHT as u8;
                self.v_reg[0xF] = 0;
                for row in 0..n {
                    let screen_y = vy + row;
                    if screen_y >= SCREEN_HEIGHT as u8 {
                        break;
                    }
                    let sprite_row = self.memory[(self.i_reg + row as u16) as usize];
                    for col in 0..8 {
                        let screen_x = vx + col;
                        if screen_x >= SCREEN_WIDTH as u8 {
                            break;
                        }
                        // 逐位(bit)检查 判断当前像素是否是 1
                        let sprite_pixel = (sprite_row & (0b1000_0000 >> col)) != 0;
                        // 将二维坐标转换为一维数组索引
                        let screen_pixel_index =
                            screen_x as usize + screen_y as usize * SCREEN_WIDTH;
                        let screen_pixel = self.screen[screen_pixel_index];
                        // 碰撞检测 VF碰撞检测标志位
                        if sprite_pixel && screen_pixel {
                            self.v_reg[0xF] = 1;
                        }
                        self.screen[screen_pixel_index] ^= sprite_pixel;
                    }
                    self.draw_flag = true;
                }
            }
            Instruction::InsEX9E(x) => {
                if self.keypad[self.v_reg[x as usize] as usize] {
                    self.pc += 2;
                }
            }
            Instruction::InsEXA1(x) => {
                if !self.keypad[self.v_reg[x as usize] as usize] {
                    self.pc += 2;
                }
            }
            Instruction::InsFX07(x) => {
                self.v_reg[x as usize] = self.delay_timer;
            }
            Instruction::InsFX0A(x) => {
                let mut pressed = false;
                for (key_code, &key_pressed) in self.keypad.iter().enumerate() {
                    if key_pressed {
                        pressed = true;
                        self.wait_for_key_release = Some(key_code);
                        self.v_reg[x as usize] = key_code as u8;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            }
            Instruction::InsFX15(x) => {
                self.delay_timer = self.v_reg[x as usize];
            }
            Instruction::InsFX18(x) => {
                self.sound_timer = self.v_reg[x as usize];
            }
            Instruction::InsFX1E(x) => {
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x as usize] as u16);
            }
            Instruction::InsFX29(x) => {
                self.i_reg = self.v_reg[x as usize] as u16 * 5;
            }
            Instruction::InsFX33(x) => {
                let hundreds = self.v_reg[x as usize] / 100;
                let tens = (self.v_reg[x as usize] / 10) % 10;
                let units = self.v_reg[x as usize] % 10;
                self.memory[self.i_reg as usize] = hundreds;
                self.memory[self.i_reg as usize + 1] = tens;
                self.memory[self.i_reg as usize + 2] = units;
            }
            Instruction::InsFX55(x) => {
                for i in 0..=x {
                    self.memory[(self.i_reg + i as u16) as usize] = self.v_reg[i as usize]
                }
                self.i_reg += x as u16 + 1;
            }
            Instruction::InsFX65(x) => {
                for i in 0..=x {
                    self.v_reg[i as usize] = self.memory[(self.i_reg + i as u16) as usize];
                }
                self.i_reg += x as u16 + 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Instruction;
    use crate::rng::LinearCongruentialGenerator;

    fn create_chip8() -> Chip8<LinearCongruentialGenerator> {
        let mut c8 = Chip8::new(LinearCongruentialGenerator::default());
        c8.load_fontset(); // 确保加载字体
        c8
    }

    #[test]
    fn test_memory_load_fontset() {
        let c8 = create_chip8();
        // 检查前5个字符（数字0）
        assert_eq!(&c8.memory[0..5], &[0xF0, 0x90, 0x90, 0x90, 0xF0]);
    }

    #[test]
    fn test_rom_loading() {
        let mut c8 = create_chip8();
        let rom: [u8; 3] = [0x12, 0x34, 0x56];
        c8.load_rom(&rom);
        assert_eq!(&c8.memory[ROM_START..ROM_START + 3], &[0x12, 0x34, 0x56]);
    }

    #[test]
    fn test_opcode_execution() {
        let mut c8 = create_chip8();
        // 测试 6XNN (LD Vx, NN)
        c8.memory[0x200] = 0x6A; // Vx = A
        c8.memory[0x201] = 0xFF; // NN = FF
        c8.pc = PROGRAM_START;

        let opcode = c8.fetch_opcode();
        let instruction = Instruction::try_from(opcode).unwrap();
        c8.execute_instruction(&instruction);

        assert_eq!(c8.v_reg[0xA], 0xFF);
    }

    #[test]
    fn test_jump_instruction() {
        let mut c8 = create_chip8();
        // 测试 1NNN (JP)
        c8.memory[0x200] = 0x12;
        c8.memory[0x201] = 0x30; // JP 0x230
        c8.pc = PROGRAM_START;

        let opcode = c8.fetch_opcode();
        let instruction = Instruction::try_from(opcode).unwrap();
        c8.execute_instruction(&instruction);

        assert_eq!(c8.pc, 0x230);
    }

    #[test]
    fn test_stack_operations() {
        let mut c8 = create_chip8();
        // 测试 2NNN (CALL)
        c8.pc = 0x200;
        c8.memory[0x200] = 0x23;
        c8.memory[0x201] = 0x00; // CALL 0x300

        // 执行CALL
        let opcode = c8.fetch_opcode();
        let instruction = Instruction::try_from(opcode).unwrap();
        c8.execute_instruction(&instruction);

        assert_eq!(c8.stack[0], 0x202); // 返回地址
        assert_eq!(c8.stack_pointer, 1);
        assert_eq!(c8.pc, 0x300);

        // 测试 00EE (RET)
        c8.memory[0x300] = 0x00;
        c8.memory[0x301] = 0xEE;
        c8.pc = 0x300;

        let opcode = c8.fetch_opcode();
        let instruction = Instruction::try_from(opcode).unwrap();
        c8.execute_instruction(&instruction);

        assert_eq!(c8.stack_pointer, 0);
        assert_eq!(c8.pc, 0x202);
    }

    #[test]
    fn test_timer_decrement() {
        let mut c8 = create_chip8();
        c8.delay_timer = 5;
        c8.sound_timer = 3;

        c8.tick_timer();
        assert_eq!(c8.delay_timer, 4);
        assert_eq!(c8.sound_timer, 2);

        c8.tick_timer();
        assert_eq!(c8.delay_timer, 3);
        assert_eq!(c8.sound_timer, 1);
    }

    #[test]
    fn test_draw_instruction() {
        let mut c8 = create_chip8();
        // 初始化精灵数据
        c8.i_reg = 0x00;
        c8.memory[0] = 0b11110000; // 4像素宽

        // 设置坐标
        c8.v_reg[0] = 0; // V0 = X
        c8.v_reg[1] = 0; // V1 = Y

        // 执行DXYN（D015）
        c8.execute_instruction(&Instruction::InsDXYN(0, 1, 1));

        // 验证第一行像素
        assert!(c8.screen[0]); // 第1列
        assert!(c8.screen[1]); // 第2列
        assert!(c8.screen[2]); // 第3列
        assert!(c8.screen[3]); // 第4列
        assert_eq!(c8.v_reg[0xF], 0); // 无碰撞
    }

    #[test]
    fn test_arithmetic_instructions() {
        let mut c8 = create_chip8();

        // 测试8XY4（ADD）
        c8.v_reg[0] = 0xFE;
        c8.v_reg[1] = 0x03;
        c8.execute_instruction(&Instruction::Ins8XY4(0, 1));
        assert_eq!(c8.v_reg[0], 0x01); // 溢出
        assert_eq!(c8.v_reg[0xF], 1); // 进位标志

        // 测试8XY5（SUB）
        c8.v_reg[0] = 0x05;
        c8.v_reg[1] = 0x03;
        c8.execute_instruction(&Instruction::Ins8XY5(0, 1));
        assert_eq!(c8.v_reg[0], 0x02);
        assert_eq!(c8.v_reg[0xF], 1); // 无借位
    }

    #[test]
    fn test_keyboard_instructions() {
        let mut c8 = create_chip8();

        // 测试EXA1（SKNP）
        c8.v_reg[0] = 0xA; // 检查按键A（hex key）
        c8.keypad[0xA] = false;
        c8.execute_instruction(&Instruction::InsEXA1(0));
        assert_eq!(c8.pc, 0x200 + 2); // 应该跳过

        // 测试FX0A（等待按键）
        c8.keypad[0x5] = true;
        c8.execute_instruction(&Instruction::InsFX0A(0));
        assert_eq!(c8.v_reg[0], 0x5);
    }
}
