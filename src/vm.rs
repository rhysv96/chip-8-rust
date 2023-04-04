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

pub struct System {
    memory: [u8; MEMORY_BYTES],
    pub screen: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    pub drew_in_last_tick: bool,
    pc: u16,
    index: u16,
    stack: Vec<u16>,
    registers: [u8; 16],
}

impl System {
    pub fn new() -> System {
        let mut sys = System {
            memory: [0 as u8; MEMORY_BYTES],
            screen: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            drew_in_last_tick: false,
            index: 0,
            pc: 0x200,
            stack: vec![],
            registers: [0 as u8; 16],
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

    pub fn tick(&mut self) {
        self.drew_in_last_tick = false;
        let opcode = self.fetch();
        let opcode = System::decode(opcode).unwrap();
        self.execute(opcode);
    }

    pub fn load_rom(&mut self, data: Vec<u8>) {
        let mut index = 0x0200 as usize;
        for byte in data {
            self.memory[index] = byte;
            index += 1;
        }
    }

    fn fetch(&mut self) -> u16 {
        let pc = self.pc as usize;
        let byte1 = self.memory[pc] as u16;
        let byte2 = self.memory[pc + 1] as u16;
        let instruction = (byte1 << 8) + byte2;
        self.pc += 2;
        instruction
    }

    fn decode(opcode: u16) -> Result<OpCode, String> {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = ((opcode & 0x000F)) as u8;
        let nn = ((opcode & 0x00FF)) as u8;
        let nnn = ((opcode & 0x0FFF)) as u16;

        // 0x00E0
        if opcode == 0x00E0 {
            return Ok(OpCode::ClearScreen);
        }

        // 0x00EE
        if opcode == 0x00EE {
            return Ok(OpCode::ExitSubroutine);
        }

        // 0x1###
        if opcode & 0xF000 == 0x1000 {
            return Ok(OpCode::Jump(nnn));
        }

        // 0x2###
        if opcode & 0xF000 == 0x2000 {
            return Ok(OpCode::EnterSubroutine(nnn));
        }

        // 0x3###
        if opcode & 0xF000 == 0x3000 {
            return Ok(OpCode::SkipIfMemoryEqual(x, nn));
        }

        // 0x4###
        if opcode & 0xF000 == 0x4000 {
            return Ok(OpCode::SkipIfMemoryNotEqual(x, nn));
        }

        // 0x5###
        if opcode & 0xF000 == 0x5000 {
            return Ok(OpCode::SkipIfRegisterEqual(x, y));
        }

        // 0x8###
        if opcode & 0xF000 == 0x8000 {
            if opcode & 0x000F == 0x0000 {
                return Ok(OpCode::SetXtoY(x, y));
            }
            if opcode & 0x000F == 0x0001 {
                return Ok(OpCode::BitwiseOr(x, y));
            }
            if opcode & 0x000F == 0x0002 {
                return Ok(OpCode::BitwiseAnd(x, y));
            }
            if opcode & 0x000F == 0x0003 {
                return Ok(OpCode::BitwiseXor(x, y));
            }
            if opcode & 0x000F == 0x0004 {
                return Ok(OpCode::AddYtoX(x, y));
            }
            if opcode & 0x000F == 0x0005 {
                return Ok(OpCode::SubtractYfromX(x, y));
            }
            if opcode & 0x000F == 0x0006 {
                return Ok(OpCode::ShiftLeft(x, y));
            }
            if opcode & 0x000F == 0x0005 {
                return Ok(OpCode::SubtractXfromY(x, y));
            }
            if opcode & 0x000F == 0x000E {
                return Ok(OpCode::ShiftRight(x, y));
            }
        }

        // 0x9###
        if opcode & 0xF000 == 0x9000 {
            return Ok(OpCode::SkipIfRegisterNotEqual(x, y));
        }

        // 0x6###
        if opcode & 0xF000 == 0x6000 {
            return Ok(OpCode::SetRegister(x, nn));
        }

        // 0x7###
        if opcode & 0xF000 == 0x7000 {
            return Ok(OpCode::AddRegister(x, nn));
        }

        // 0xA###
        if opcode & 0xF000 == 0xA000 {
            return Ok(OpCode::SetIndexRegister(nnn));
        }

        // 0xD###
        if opcode & 0xF000 == 0xD000 {
            return Ok(OpCode::Draw(x, y, n));
        }

        Err(format!("failed to parse opcode {:#06x}", opcode))
    }

    fn execute(&mut self, opcode: OpCode) {
        match opcode {
            OpCode::AddRegister(address, value) => {
                self.registers[address as usize] += value;
            }
            OpCode::SetRegister(address, value) => {
                self.registers[address as usize] = value;
            }
            OpCode::ClearScreen => {
                self.screen = [[false; 64]; 32];
            },
            OpCode::Draw(x, y, height) => {
                // trigger redraw after tick
                self.drew_in_last_tick = true;

                // get x and y pos from the register specified by args
                let x = (self.registers[x as usize] % SCREEN_WIDTH as u8) as usize;
                let y = (self.registers[y as usize] % SCREEN_HEIGHT as u8) as usize;

                // set flag reg to 0
                self.registers[0xF] = 0;

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
                            self.registers[0xF] = 1;
                            self.screen[y + n][x + bit] = false;

                        // if screen isn't on but is on on sprite, then turn it on
                        } else if new && !current {
                            self.screen[y + n][x + bit] = true;
                        }
                    }
                }
            }
            OpCode::Jump(address) => {
                self.pc = address;
            },
            OpCode::EnterSubroutine(address) => {
                self.stack.push(self.pc);
                self.pc = address;
            },
            OpCode::ExitSubroutine => {
                self.pc = self.stack.pop().unwrap();
            }
            OpCode::SetIndexRegister(value) => {
                self.index = value;
            },
            OpCode::SkipIfMemoryEqual(x, addr) => {
                if self.registers[x as usize] == self.memory[addr as usize] {
                    self.pc += 2
                }
            },
            OpCode::SkipIfMemoryNotEqual(x, addr) => {
                if self.registers[x as usize] != self.memory[addr as usize] {
                    self.pc += 2
                }
            },
            OpCode::SkipIfRegisterEqual(x, y) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2
                }
            },
            OpCode::SkipIfRegisterNotEqual(x, y) => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.pc += 2
                }
            },
            OpCode::SetXtoY(x, y) => {
                self.registers[x as usize] = self.registers[y as usize];
            },
            OpCode::BitwiseOr(x, y) => {
                self.registers[x as usize] |= self.registers[y as usize];
            }
            OpCode::BitwiseAnd(x, y) => {
                self.registers[x as usize] &= self.registers[y as usize];
            }
            OpCode::BitwiseXor(x, y) => {
                self.registers[x as usize] ^= self.registers[y as usize];
            }
            OpCode::ShiftRight(x, y) => {
                self.registers[x as usize] >>= self.registers[y as usize];
            }
            OpCode::ShiftLeft(x, y) => {
                self.registers[x as usize] <<= self.registers[y as usize];
            }
            OpCode::AddYtoX(x, y) => {
                self.registers[x as usize] += self.registers[y as usize];
            }
            OpCode::SubtractYfromX(x, y) => {
                self.registers[x as usize] -= self.registers[y as usize];
            }
            OpCode::SubtractXfromY(x, y) => {
                self.registers[x as usize] = self.registers[x as usize] - self.registers[y as usize];
            }
        };
    }
}

enum OpCode {
    //// COMPLETE
    // Assign
    SetXtoY(u8, u8), // 8XY0

    // BCD
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

    // KeyOp
    // Math
    AddYtoX(u8, u8), // 8XY4
    SubtractYfromX(u8, u8), // 8XY5
    SubtractXfromY(u8, u8), // 8XY7

    // Memory
    SetIndexRegister(u16), // ANNN

    // Rand
    // Sound
    // Timer

    //// TODO
    /*
    JumpWithOffset(u16), // BNNN
    Random(u8, u8), // CXNN
    SkipIfKeyPressed(u8), // EX9E
    SkipIfKeyNotPressed(u8), // EXA1
    GetDelayTimerValue(u8), // FX07
    SetDelayTimerValue(u8), // FX15
    SetSoundTimerValue(u8), // FX18
    AddXToIndexRegister(u8), // FX1E
    GetKeyBlocking(u8), // FX0A
    SetIndexToFontCharacter(u8), // FX29
    SaveBCDConversionToMemory(u8), // FX33
    StoreMemory(u8), // FX55
    LoadMemory(u8), // FX65
    */
}
