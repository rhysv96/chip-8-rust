const MEMORY_BYTES: usize = 4096;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const FONT: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

// options
const LOAD_Y_BEFORE_SHIFT: bool = true; // used in bitshift, loads reg y into x before shifting x

pub enum Status {
    Active,
    Terminated,
}

pub struct System {
    memory: [u8; MEMORY_BYTES],
    pub screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    pub keys: u16,
    pub sound_timer: u8,
    pub delay_timer: u8,
    pub status: Status,
    pc: u16,
    index: u16,
    stack: Vec<u16>,
    registers: [u8; 16],
}

impl Clone for Status {
    fn clone(&self) -> Self {
        match self {
            Self::Active => Self::Active,
            Self::Terminated => Self::Terminated,
        }
    }
}

impl Clone for System {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
            screen: self.screen.clone(),
            keys: self.keys.clone(),
            sound_timer: self.sound_timer.clone(),
            delay_timer: self.delay_timer.clone(),
            pc: self.pc.clone(),
            index: self.index.clone(),
            stack: self.stack.clone(),
            registers: self.registers.clone(),
            status: self.status.clone(),
        }
    }
}

impl System {
    pub fn new() -> System {
        let mut sys = System {
            memory: [0 as u8; MEMORY_BYTES],
            screen: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            keys: 0,
            index: 0,
            pc: 0x200,
            stack: vec![],
            registers: [0 as u8; 16],
            sound_timer: 0,
            delay_timer: 0,
            status: Status::Active,
        };

        // initialize font
        let mut index: usize = 0;
        for letter in FONT {
            for byte in letter {
                sys.memory[index] = byte;
                index += 1;
            }
        }

        sys
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        let mut index = 0x0200 as usize;
        for byte in data {
            self.memory[index] = byte;
            index += 1;
        }
    }

    pub fn tick(&self, next: &mut Self) {
        let opcode = self.fetch(next);
        let opcode = Self::decode(opcode).unwrap();
        self.execute(opcode, next);
    }

    fn fetch(&self, next: &mut Self) -> u16 {
        let pc = self.pc as usize;
        let byte1 = self.memory[pc] as u16;
        let byte2 = self.memory[pc + 1] as u16;
        let instruction = (byte1 << 8) + byte2;
        next.pc += 2;
        instruction
    }

    fn decode(opcode: u16) -> Result<OpCode, String> {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = ((opcode & 0x000F)) as u8;
        let nn = ((opcode & 0x00FF)) as u8;
        let nnn = ((opcode & 0x0FFF)) as u16;

        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => return Ok(OpCode::ClearScreen),
                0x00EE => return Ok(OpCode::ExitSubroutine),
                _ => {},
            },
            0x1000 => return Ok(OpCode::Jump(nnn)),
            0x2000 => return Ok(OpCode::EnterSubroutine(nnn)),
            0x3000 => return Ok(OpCode::SkipIfMemoryEqual(x, nn)),
            0x4000 => return Ok(OpCode::SkipIfMemoryNotEqual(x, nn)),
            0x5000 => return Ok(OpCode::SkipIfRegisterEqual(x, y)),
            0x6000 => return Ok(OpCode::SetRegister(x, nn)),
            0x7000 => return Ok(OpCode::AddRegister(x, nn)),
            0x8000 => match opcode & 0x000F {
                0x0000 => return Ok(OpCode::SetXtoY(x, y)),
                0x0001 => return Ok(OpCode::BitwiseOr(x, y)),
                0x0002 => return Ok(OpCode::BitwiseAnd(x, y)),
                0x0003 => return Ok(OpCode::BitwiseXor(x, y)),
                0x0004 => return Ok(OpCode::AddYtoX(x, y)),
                0x0005 => return Ok(OpCode::SubtractYfromX(x, y)),
                0x0006 => return Ok(OpCode::ShiftRight(x, y)),
                0x0007 => return Ok(OpCode::SubtractXfromY(x, y)),
                0x000E => return Ok(OpCode::ShiftLeft(x, y)),
                _ => {},
            },
            0x9000 => return Ok(OpCode::SkipIfRegisterNotEqual(x, y)),
            0xA000 => return Ok(OpCode::SetIndexRegister(nnn)),
            0xB000 => return Ok(OpCode::JumpWithOffset(nnn)),
            0xC000 => return Ok(OpCode::Random(x, nn)),
            0xD000 => return Ok(OpCode::Draw(x, y, n)),
            0xE000 => match opcode & 0x00FF {
                0x009E => return Ok(OpCode::SkipIfKeyPressed(x)),
                0x00A1 => return Ok(OpCode::SkipIfKeyNotPressed(x)),
                _ => {},
            },
            0xF000 => match opcode & 0x00FF {
                0x0018 => return Ok(OpCode::SetSoundTimerValue(x)),
                0x0007 => return Ok(OpCode::GetDelayTimerValue(x)),
                0x0015 => return Ok(OpCode::SetDelayTimerValue(x)),
                0x0033 => return Ok(OpCode::SaveBCDConversionToMemory(x)),
                0x0055 => return Ok(OpCode::StoreMemory(x)),
                0x0065 => return Ok(OpCode::LoadMemory(x)),
                _ => {},
            },
            _ => {},
        };

        Err(format!("failed to parse opcode {:#06X}", opcode))
    }

    fn execute(&self, opcode: OpCode, next: &mut Self) {
        match opcode {
            OpCode::AddRegister(address, value) => {
                next.registers[address as usize] += value;
            }
            OpCode::SetRegister(address, value) => {
                next.registers[address as usize] = value;
            }
            OpCode::ClearScreen => {
                next.screen = [[false; 64]; 32];
            },
            OpCode::Draw(x, y, height) => {
                // get x and y pos from the register specified by args
                let x = (self.registers[x as usize] % SCREEN_WIDTH as u8) as usize;
                let y = (self.registers[y as usize] % SCREEN_HEIGHT as u8) as usize;

                // set flag reg to 0
                next.registers[0xF] = 0;

                for n in 0..height {
                    let n = n as usize;
                    // break out if off edge of screen
                    if y + n > SCREEN_HEIGHT {
                        break;
                    }

                    // grab sprite row from memory
                    let sprite_byte = self.memory[(self.index + n as u16) as usize];

                    for bit in 0..8 {
                        // break out if off edge of screen
                        if x + bit > SCREEN_WIDTH {
                            break;
                        }

                        // new bit is pixel in row
                        let new = (sprite_byte & (0x80 >> bit)) != 0;

                        // current is whatever is on screen
                        let current = self.screen[y as usize + n][x as usize + bit];

                        // if new and current are both set, invert and set flag register to 1
                        if new && current {
                            next.registers[0xF] = 1;
                            next.screen[y + n][x + bit] = false;

                        // if screen isn't on but is on on sprite, then turn it on
                        } else if new && !current {
                            next.screen[y + n][x + bit] = true;
                        }
                    }
                }
            }
            OpCode::Jump(address) => {
                next.pc = address;
            },
            OpCode::EnterSubroutine(address) => {
                next.stack.push(next.pc);
                next.pc = address;
            },
            OpCode::ExitSubroutine => {
                next.pc = next.stack.pop().unwrap();
            }
            OpCode::SetIndexRegister(value) => {
                next.index = value;
            },
            OpCode::SkipIfMemoryEqual(x, val) => {
                if self.registers[x as usize] == val {
                    next.pc += 2;
                }
            },
            OpCode::SkipIfMemoryNotEqual(x, val) => {
                if self.registers[x as usize] != val {
                    next.pc += 2;
                }
            },
            OpCode::SkipIfRegisterEqual(x, y) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    next.pc += 2;
                }
            },
            OpCode::SkipIfRegisterNotEqual(x, y) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    next.pc += 2;
                }
            },
            OpCode::SetXtoY(x, y) => {
                next.registers[x as usize] = self.registers[y as usize];
            },
            OpCode::BitwiseOr(x, y) => {
                next.registers[x as usize] |= self.registers[y as usize];
            },
            OpCode::BitwiseAnd(x, y) => {
                next.registers[x as usize] &= self.registers[y as usize];
            },
            OpCode::BitwiseXor(x, y) => {
                next.registers[x as usize] ^= self.registers[y as usize];
            },
            OpCode::ShiftRight(x, y) => {
                if LOAD_Y_BEFORE_SHIFT {
                    next.registers[x as usize] = self.registers[y as usize];
                }
                let value = self.registers[x as usize];
                next.registers[0xF as usize] = value & 1;
                next.registers[x as usize] >>= 1;
            },
            OpCode::ShiftLeft(x, y) => {
                if LOAD_Y_BEFORE_SHIFT {
                    next.registers[x as usize] = self.registers[y as usize];
                }
                let value = self.registers[x as usize];
                next.registers[0xF as usize] = (value & 0x80 == 0x80) as u8;
                next.registers[x as usize] <<= 1;
            },
            OpCode::AddYtoX(x, y) => {
                next.registers[x as usize] += self.registers[y as usize];
            },
            OpCode::SubtractYfromX(x, y) => {
                next.registers[x as usize] -= self.registers[y as usize];
            },
            OpCode::SubtractXfromY(x, y) => {
                next.registers[x as usize] = self.registers[x as usize] - self.registers[y as usize];
            },
            OpCode::JumpWithOffset(address) => {
                next.pc = address + self.registers[0] as u16;
            },
            OpCode::Random(x, mask) => {
                let val: u8 = rand::random();
                next.registers[x as usize] = mask & val;
            },
            OpCode::SkipIfKeyPressed(x) => {
                if self.keys >> x & 0x0001 == 1 {
                    next.pc += 2;
                }
            },
            OpCode::SkipIfKeyNotPressed(x) => {
                if self.keys >> x & 0x0001 == 0 {
                    next.pc += 2;
                }
            },
            OpCode::StoreMemory(x) => {
                for i in 0..x+1 {
                    next.memory[(self.index + i as u16) as usize] = self.registers[i as usize];
                }
            },
            OpCode::LoadMemory(x) => {
                for i in 0..x+1 {
                    next.registers[i as usize] = self.memory[(self.index + i as u16) as usize];
                }
            },
            OpCode::SaveBCDConversionToMemory(x) => {
                let value = next.registers[x as usize];
                let hundreds = (value / 100) as u8;
                let tens = ((value - (hundreds * 100)) / 10) as u8;
                let ones = (value - hundreds * 100 - tens * 10) as u8;
                next.memory[self.index as usize] = hundreds;
                next.memory[(self.index + 1) as usize] = tens;
                next.memory[(self.index + 2) as usize] = ones;
            },
            OpCode::SetSoundTimerValue(x) => {
                next.sound_timer = self.registers[x as usize];
            },
            OpCode::GetDelayTimerValue(x) => {
                next.registers[x as usize] = self.delay_timer;
            },
            OpCode::SetDelayTimerValue(x) => {
                next.delay_timer = self.registers[x as usize];
            },
        };
    }

    fn terminate(&mut self) {
        self.status = Status::Terminated;
    }
}

#[derive(Debug)]
enum OpCode {
    //// COMPLETE
    // Assign
    SetXtoY(u8, u8), // 8XY0

    // BCD
    SaveBCDConversionToMemory(u8), // FX33

    // BitOp
    BitwiseOr(u8, u8), // 8XY1
    BitwiseAnd(u8, u8), // 8XY2
    BitwiseXor(u8, u8), // 8XY3
    ShiftRight(u8, u8), // 8XY6
    ShiftLeft(u8, u8), // 8XYE

    // Cond

    // Const
    SetRegister(u8, u8), // 6XNN
    AddRegister(u8, u8), // 7XNN

    // Display
    ClearScreen, /// 00e0
    Draw(u8, u8, u8), // DXYN

    // Flow
    Jump(u16), // 1NNN
    ExitSubroutine, // 00EE
    EnterSubroutine(u16), // 2NNN
    SkipIfMemoryEqual(u8, u8), // 3XNN
    SkipIfMemoryNotEqual(u8, u8), // 4XNN
    SkipIfRegisterEqual(u8, u8), // 5XY0
    SkipIfRegisterNotEqual(u8, u8), // 9XY0
    JumpWithOffset(u16), // BNNN
    SkipIfKeyPressed(u8), // EX9E
    SkipIfKeyNotPressed(u8), // EXA1

    // KeyOp
    // Math
    AddYtoX(u8, u8), // 8XY4
    SubtractYfromX(u8, u8), // 8XY5
    SubtractXfromY(u8, u8), // 8XY7

    // Memory
    SetIndexRegister(u16), // ANNN
    StoreMemory(u8), // FX55
    LoadMemory(u8), // FX65

    // Rand
    Random(u8, u8), // CXNN

    // Sound
    SetSoundTimerValue(u8), // FX18

    // Timer
    GetDelayTimerValue(u8), // FX07
    SetDelayTimerValue(u8), // FX15

    //// TODO
    /*
    AddXToIndexRegister(u8), // FX1E
    GetKeyBlocking(u8), // FX0A
    SetIndexToFontCharacter(u8), // FX29
    */
}
