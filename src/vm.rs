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
    pc: u16,
    index: u16,
    stack: Vec<u16>,
    registers: [u16; 16],
}

impl System {
    pub fn new() -> System {
        let mut sys = System {
            memory: [0 as u8; MEMORY_BYTES],
            screen: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            index: 0,
            pc: 0x200,
            stack: vec![],
            registers: [0 as u16; 16],
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

    fn decode(opcode: u16) -> Result<OpCode, u16> {
        // 0x0###
        if opcode == 0x00E0 {
            return Ok(OpCode::ClearScreen);
        }

        // 0x1###
        if opcode & 0xF000 == 0x1000 {
            return Ok(OpCode::Jump(opcode & 0x0FFF));
        }

        // 0x6###
        if opcode & 0xF000 == 0x6000 {
            return Ok(OpCode::SetRegister(((opcode & 0x0F00) >> 8) as u8, opcode & 0x00FF));
        }

        // 0x7###
        if opcode & 0xF000 == 0x7000 {
            return Ok(OpCode::AddRegister(((opcode & 0x0F00) >> 8) as u8, opcode & 0x00FF));
        }

        // 0xA###
        if opcode & 0xF000 == 0xA000 {
            return Ok(OpCode::SetIndexRegister(opcode & 0x0FFF));
        }

        // 0xD###
        if opcode & 0xF000 == 0xD000 {
            return Ok(OpCode::Draw(
                ((opcode & 0x0F00) >> 8) as u8,
                ((opcode & 0x00F0) >> 4) as u8,
                ((opcode & 0x000F)) as u8,
            ));
        }

        Err(opcode)
    }

    fn execute(&mut self, opcode: OpCode) {
        match opcode {
            OpCode::ClearScreen => {
                self.screen = [[false; 64]; 32];
            },
            OpCode::Jump(address) => {
                self.pc = address;
            },
            OpCode::SetRegister(address, value) => {
                self.registers[address as usize] = value;
            }
            OpCode::AddRegister(address, value) => {
                self.registers[address as usize] += value;
            }
            OpCode::SetIndexRegister(value) => {
                self.index = value;
            }
            OpCode::Draw(x, y, height) => {
                // get x and y pos from the register specified by args
                let x = (self.registers[x as usize] % SCREEN_WIDTH as u16) as usize;
                let y = (self.registers[y as usize] % SCREEN_HEIGHT as u16) as usize;

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
        };
    }
}

enum OpCode {
    ClearScreen,
    Jump(u16),
    SetRegister(u8, u16),
    AddRegister(u8, u16),
    SetIndexRegister(u16),
    Draw(u8, u8, u8),
}
