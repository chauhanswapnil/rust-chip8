use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const START_ADDR: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;
const REGISTERS_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const KEYS_COUNT: usize = 16;
const FONT_SIZE: usize = 80;

const FONT: [u8; FONT_SIZE] = [
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

enum Instruction {
    NOP,              // 0000
    ClearScreen,      // 00E0
    Jump,             // 1NNN
    CallSubroutine,   // 2NNN
    ReturnSubroutine, // 00EE
    SkipVXEqualNN,    // 3XNN
    SkipVXNotEqualNN, // 4XNN
    SkipVXEqualVY,    // 5XY0
    SkipVXNotEqualVY, // 9XY0
    SetVXToNN,        // 6XNN
    AddNNToVX,        // 7XNN
    SetVXToVY,        // 8XY0
    OR,               // 8XY1 (VX |= VY)
    AND,              // 8XY2 (VX &= VY)
    XOR,              // 8XY3 (VX ^= VY)
    Add,              // 8XY4 (VX += VY)
    SubtractFrom,     // 8XY5 (VX -= VY)
    Subtract,         // 8XY7 (VX = VY - VX)
    // TODO: Make RShift and LShift user configurable later
    RightShift, // 8XY6 (VX >>= 1)
    LeftShift,  // 8XYE ( VX <<= 1)
    SetIndex,   // ANNN (I = NNN)
    // TODO: Make Jump with offset user configurable later
    JumpWithOffset,               // BNNN (JMP V0 + NNN)
    Random,                       // CXNN (VX = rand & NN)
    Display,                      // DXYN
    SkipKeyPress,                 // EX9E
    SkipKeyRelease,               // EXA1
    SetVXToTimer,                 // FX07 VX = Delay Timer
    SetTimerToVX,                 // FX15 Delay Timer = VX
    SetSoundTimerToVX,            // FX18 Sound Timer = VX
    AddToIndex,                   // FX1E (I += VX)
    WaitKey,                      // FX0A
    FontCharacter,                // FX29 (I = FONT)
    BinaryCodedDecimalConversion, // FX33
    StoreMemory,                  // FX55 (Store V0 to VX)
    LoadMemory,                   // FX65 (Load V0 to VX)
}

impl Instruction {
    fn from_opcode(opcode: (u16, u16, u16, u16)) -> Option<Self> {
        match opcode {
            (0, 0, 0, 0) => Some(Self::NOP),                            // 0000
            (0, 0, 0xE, 0) => Some(Self::ClearScreen),                  // 00E0
            (1, _, _, _) => Some(Self::Jump),                           // 1NNN
            (2, _, _, _) => Some(Self::CallSubroutine),                 // 2NNN
            (0, 0, 0xE, 0xE) => Some(Self::ReturnSubroutine),           // 00EE
            (3, _, _, _) => Some(Self::SkipVXEqualNN),                  // 3XNN
            (4, _, _, _) => Some(Self::SkipVXNotEqualNN),               // 4XNN
            (5, _, _, 0) => Some(Self::SkipVXEqualVY),                  // 5XY0
            (9, _, _, 0) => Some(Self::SkipVXNotEqualVY),               // 9XY0
            (6, _, _, _) => Some(Self::SetVXToNN),                      // 6XNN
            (7, _, _, _) => Some(Self::AddNNToVX),                      // 7XNN
            (8, _, _, 0) => Some(Self::SetVXToVY),                      // 8XY0
            (8, _, _, 1) => Some(Self::OR),                             // 8XY1 (VX |= VY)
            (8, _, _, 2) => Some(Self::AND),                            // 8XY2 (VX &= VY)
            (8, _, _, 3) => Some(Self::XOR),                            // 8XY3 (VX ^= VY)
            (8, _, _, 4) => Some(Self::Add),                            // 8XY4 (VX += VY)
            (8, _, _, 5) => Some(Self::SubtractFrom),                   // 8XY5 (VX -= VY)
            (8, _, _, 7) => Some(Self::Subtract),                       // 8XY7 (VX = VY - VX)
            (8, _, _, 6) => Some(Self::RightShift),                     // 8XY6 (VX >>= 1)
            (8, _, _, 0xE) => Some(Self::LeftShift),                    // 8XYE ( VX <<= 1)
            (0xA, _, _, _) => Some(Self::SetIndex),                     // ANNN (I = NNN)
            (0xB, _, _, _) => Some(Self::JumpWithOffset),               // BNNN (JMP V0 + NNN)
            (0xC, _, _, _) => Some(Self::Random),                       // CXNN (VX = rand & NN)
            (0xD, _, _, _) => Some(Self::Display),                      // DXYN
            (0xE, _, _, 0xE) => Some(Self::SkipKeyPress),               // EX9E
            (0xE, _, 0xA, 1) => Some(Self::SkipKeyRelease),             // EXA1
            (0xF, _, 0, 7) => Some(Self::SetVXToTimer),                 // FX07 VX = Delay Timer
            (0xF, _, 1, 5) => Some(Self::SetTimerToVX),                 // FX15 Delay Timer = VX
            (0xF, _, 1, 8) => Some(Self::SetSoundTimerToVX),            // FX18 Sound Timer = VX
            (0xF, _, 1, 0xE) => Some(Self::AddToIndex),                 // FX1E (I += VX)
            (0xF, _, 0, 0xA) => Some(Self::WaitKey),                    // FX0A
            (0xF, _, 2, 9) => Some(Self::FontCharacter),                // FX29 (I = FONT)
            (0xF, _, 3, 3) => Some(Self::BinaryCodedDecimalConversion), // FX33
            (0xF, _, 5, 5) => Some(Self::StoreMemory),                  // FX55 (Store V0 to VX)
            (0xF, _, 6, 5) => Some(Self::LoadMemory),                   // FX65 (Load V0 to VX)
            _ => return None,
        }
    }
}

#[derive(Debug)]
pub struct Chip8 {
    pc: u16,
    memory: [u8; MEMORY_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    registers: [u8; REGISTERS_COUNT],
    index_register: u16,
    stack: [u16; STACK_SIZE],
    sp: u8,
    keys: [bool; KEYS_COUNT],
    delay_timer: u8,
    sound_timer: u8,
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; MEMORY_SIZE];

        // The font is loaded and available
        // at the start of the memory
        memory[..FONT_SIZE].copy_from_slice(&FONT);

        let chip8 = Self {
            pc: START_ADDR,
            memory,
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            registers: [0; REGISTERS_COUNT],
            index_register: 0,
            stack: [0; STACK_SIZE],
            sp: 0,
            keys: [false; KEYS_COUNT],
            delay_timer: 0,
            sound_timer: 0,
        };

        chip8
    }

    pub fn tick(&mut self) {
        // fetch, decode and execute loop the heart of the emulator
        let op = self.fetch();
        self.decode_and_execute(op);
    }

    pub fn tick_timers(&mut self) {
        // Timers should be decremented by one 60 times per second (ie. at 60 Hz) as long
        // as their value is above 0
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // TODO: use some sound api to make a beep
            }
            self.sound_timer -= 1;
        }
    }

    pub fn get_screen(&self) -> &[bool] {
        return &self.screen;
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        // Load the whole ROM in memory starting from the
        // START_ADDR which will be 0x200
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.memory[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u16 {
        // Read the instruction that PC is currently pointing at from memory.
        // An instruction is two bytes, so we need to read two successive bytes from memory
        // and combine them into one 16-bit instruction.
        let first_byte = self.memory[self.pc as usize] as u16;
        let second_byte = self.memory[(self.pc + 1) as usize] as u16;
        // We want to combine the two instructions into one 16 bit instruction
        // To do that: left shift by 8 on first byte and then logical OR the second byte
        // Example: There are two bytes 00000100 (4) and 00000101 (5)
        // Left shift 4 by 8 gives us: 100000000 (1024)
        // Logical OR the result with 5 gives us: 10000101
        // This works because we know each byte in memory is represented as u8
        let opcode = (first_byte << 8) | second_byte;

        // Increment Program Counter by 2 as we fetched 2 bytes to form an opcode above
        self.pc += 2;
        opcode
    }

    fn push_on_stack(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop_from_stack(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn decode_and_execute(&mut self, op: u16) {
        /*
            1010 0010 0010 1010 -> 41514 -> OP Example
            First Nibble (half-byte) -> 1010 to Hexadecimal
            Second Nibble ...
        */
        let nibble_1 = (op >> 12) & 0b1111;
        let nibble_2 = (op >> 8) & 0b1111;
        let nibble_3 = (op >> 4) & 0b1111;
        let nibble_4 = op & 0b1111;

        println!("Executing Opcode: {}", op);

        let opcode = (nibble_1, nibble_2, nibble_3, nibble_4);

        if let Some(instruction) = Instruction::from_opcode(opcode) {
            match instruction {
                Instruction::NOP => return,
                Instruction::ClearScreen => {
                    println!("Executing Clear Screen: {}", op);
                    // Turn all pixels off; set all values in screen to false
                    self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
                }
                Instruction::Jump => {
                    println!("Executing JUMP: {}", op);
                    // This instruction should simply set PC to NNN
                    // causing the program to jump to that memory location.
                    self.pc = op & 0xFFF;
                }
                Instruction::SetVXToNN => {
                    // 6XNN
                    println!("Executing SetVXToNN: {}", op);
                    let nn = (op & 0xFF) as u8;
                    self.registers[nibble_2 as usize] = nn;
                }
                Instruction::AddNNToVX => {
                    // 7XNN
                    println!("Executing AddNNToVX: {}", op);
                    let nn = (op & 0xFF) as u8;
                    self.registers[nibble_2 as usize] =
                        self.registers[nibble_2 as usize].wrapping_add(nn);
                }
                Instruction::SetIndex => {
                    // ANNN (I = NNN)
                    println!("Executing SetIndex: {}", op);
                    self.index_register = op & 0xFFF;
                }
                Instruction::Display => {
                    println!("Executing Display: {}", op);
                    // https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#dxyn-display
                    // Get the x,y coordinates from the registers
                    let x_coord = self.registers[nibble_2 as usize] as u16;
                    let y_coord = self.registers[nibble_3 as usize] as u16;
                    // The last digit determines how many rows high the sprite is
                    let num_rows = nibble_4;

                    // Keep track if any pixels were flipped
                    let mut flipped = false;
                    // Iterate over each row of the sprite
                    for y_line in 0..num_rows {
                        // Determine which memory address the rows data is stored
                        let addr = self.index_register + y_line;
                        let pixels = self.memory[addr as usize];
                        // Iterate over each column in the row
                        for x_line in 0..8 {
                            // Use a mask to fetch current pixels bit. Only flip if a 1
                            if (pixels & (0b1000_0000 >> x_line)) != 0 {
                                // Sprites should wrap around screen, so apply modulo
                                let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                                let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                                // Get the pixel's index in the 1D screen array
                                let idx = x + SCREEN_WIDTH * y;
                                // Check if about to flip the pixel and set
                                flipped |= self.screen[idx];
                                self.screen[idx] ^= true;
                            }
                        }
                    }
                    // Populate VF register
                    if flipped {
                        self.registers[0xF] = 1;
                    } else {
                        self.registers[0xF] = 0;
                    }
                }
                Instruction::CallSubroutine => {
                    // Calls the subroutine at memory location NNN i.e should set PC to NNN.
                    let nnn = op & 0xFFF;
                    self.push_on_stack(self.pc);
                    self.pc = nnn;
                }
                Instruction::ReturnSubroutine => {
                    // Return from a subroutine by popping the last address
                    // from the stack and setting the PC to it
                    let addr = self.pop_from_stack();
                    self.pc = addr;
                }
                Instruction::SkipVXEqualNN => {
                    // 3XNN
                    let x = nibble_2 as usize;
                    let nn = (op & 0xFF) as u8;
                    if self.registers[x] == nn {
                        self.pc += 2;
                    }
                }
                Instruction::SkipVXNotEqualNN => {
                    // 4XNN
                    let x = nibble_2 as usize;
                    let nn = (op & 0xFF) as u8;
                    if self.registers[x] != nn {
                        self.pc += 2;
                    }
                }
                Instruction::SkipVXEqualVY => {
                    // 5XY0
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;
                    if self.registers[x] == self.registers[y] {
                        self.pc += 2;
                    }
                }
                Instruction::SkipVXNotEqualVY => {
                    // 9XY0
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;
                    if self.registers[x] != self.registers[y] {
                        self.pc += 2;
                    }
                }
                Instruction::SetVXToVY => {
                    // 8XY0
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;
                    self.registers[x] = self.registers[y];
                }
                Instruction::OR => {
                    // 8XY1 (VX |= VY)
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;
                    self.registers[x] |= self.registers[y];
                }
                Instruction::AND => {
                    // 8XY2 (VX &= VY)
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;
                    self.registers[x] &= self.registers[y];
                }
                Instruction::XOR => {
                    // 8XY3 (VX ^= VY)
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;
                    self.registers[x] ^= self.registers[y];
                }
                Instruction::Add => {
                    // 8XY4 (VX += VY)
                    // If the result is larger than 255 (and thus overflows the 8-bit register
                    // VX), the flag register VF is set to 1. If it doesn't overflow,
                    // VF is set to 0.
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;

                    let (new_vx, carry) = self.registers[x].overflowing_add(self.registers[y]);
                    let new_vf = if carry { 1 } else { 0 };

                    self.registers[x] = new_vx;
                    self.registers[0xF] = new_vf;
                }
                Instruction::SubtractFrom => {
                    // 8XY5 (VX -= VY)
                    // If the minuend (the first operand) is larger than the subtrahend
                    // (second operand), VF will be set to 1. If the subtrahend is larger, and
                    // “underflow” the result, VF is set to 0.
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;

                    let (new_vx, borrow) = self.registers[x].overflowing_sub(self.registers[y]);
                    let new_vf = if borrow { 0 } else { 1 };

                    self.registers[x] = new_vx;
                    self.registers[0xF] = new_vf;
                }
                Instruction::Subtract => {
                    // 8XY7 (VX = VY - VX)
                    // If the minuend (the first operand) is larger than the subtrahend
                    // (second operand), VF will be set to 1. If the subtrahend is larger, and
                    // “underflow” the result, VF is set to 0.
                    let x = nibble_2 as usize;
                    let y = nibble_3 as usize;

                    let (new_vx, borrow) = self.registers[y].overflowing_sub(self.registers[x]);
                    let new_vf = if borrow { 0 } else { 1 };

                    self.registers[x] = new_vx;
                    self.registers[0xF] = new_vf;
                }
                Instruction::RightShift => {
                    // 8XY6 (VX >>= 1)
                    let x = nibble_2 as usize;
                    let lsb = self.registers[x] & 1;
                    self.registers[x] >>= 1;
                    self.registers[0xF] = lsb;
                }
                Instruction::LeftShift => {
                    // 8XYE ( VX <<= 1)
                    let x = nibble_2 as usize;
                    let msb = (self.registers[x] >> 7) & 1;
                    self.registers[x] <<= 1;
                    self.registers[0xF] = msb;
                }
                Instruction::JumpWithOffset => {
                    // BNNN (JMP V0 + NNN)
                    let nnn = op & 0xFFF;
                    self.pc = (self.registers[0] as u16) + nnn;
                }
                Instruction::Random => {
                    // CXNN (VX = rand & NN)
                    // generates a random number, binary ANDs it with the value NN, and
                    // puts the result in VX
                    let x = nibble_2 as usize;
                    let nn = (op & 0xFF) as u8;
                    let rng: u8 = random();
                    self.registers[x] = rng & nn;
                }
                Instruction::SkipKeyPress => {
                    // EX9E
                    // skip one instruction (increment PC by 2) if the key corresponding to
                    // the value in VX is pressed.
                    let x = nibble_2 as usize;
                    let vx = self.registers[x];
                    let key = self.keys[vx as usize];
                    if key {
                        self.pc += 2;
                    }
                }
                Instruction::SkipKeyRelease => {
                    // EXA1
                    // skips if the key corresponding to the value in VX is not pressed.
                    let x = nibble_2 as usize;
                    let vx = self.registers[x];
                    let key = self.keys[vx as usize];
                    if !key {
                        self.pc += 2;
                    }
                }
                Instruction::SetVXToTimer => {
                    // FX07 VX = Delay Timer
                    let x = nibble_2 as usize;
                    self.registers[x] = self.delay_timer;
                }
                Instruction::SetTimerToVX => {
                    // FX15 Delay Timer = VX
                    let x = nibble_2 as usize;
                    self.delay_timer = self.registers[x];
                }
                Instruction::SetSoundTimerToVX => {
                    // FX18 Sound Timer = VX
                    let x = nibble_2 as usize;
                    self.sound_timer = self.registers[x];
                }
                Instruction::AddToIndex => {
                    // FX1E (I += VX)
                    let x = nibble_2 as usize;
                    let vx = self.registers[x] as u16;
                    self.index_register = self.index_register.wrapping_add(vx);
                }
                Instruction::WaitKey => {
                    // FX0A
                    // stops executing instructions and waits for key input
                    // (or loops forever, unless a key is pressed).
                    // Also, If a key is pressed while this instruction is waiting for input,
                    // its hexadecimal value will be put in VX and execution continues.
                    let x = nibble_2 as usize;
                    let mut pressed = false;
                    for i in 0..self.keys.len() {
                        if self.keys[i] {
                            self.registers[x] = i as u8;
                            pressed = true;
                            break;
                        }
                    }
                    if !pressed {
                        self.pc -= 2;
                    }
                }
                Instruction::FontCharacter => {
                    // FX29 (I = FONT)
                    // Set index register to the address of the hexadecimal character
                    // in VX
                    let x = nibble_2 as usize;
                    let c = self.registers[x] as u16;
                    self.index_register = c * 5;
                }
                Instruction::BinaryCodedDecimalConversion => {
                    // FX33
                    // Take the number in VX (which is one byte, so it can be any number
                    // from 0 to 255) and convert it to three decimal digits, storing these
                    // digits in memory at the address in the index register I.
                    let x = nibble_2 as usize;
                    let vx = self.registers[x] as f32;

                    let hundreds_digit = (vx / 100.0).floor() as u8;
                    let tens_digit = ((vx / 10.0) % 10.0).floor() as u8;
                    // Fetch the ones digit by tossing the hundreds and the tens
                    let ones_digit = (vx % 10.0) as u8;

                    self.memory[self.index_register as usize] = hundreds_digit;
                    self.memory[(self.index_register + 1) as usize] = tens_digit;
                    self.memory[(self.index_register + 2) as usize] = ones_digit;
                }
                Instruction::StoreMemory => {
                    // FX55 (Store V0 to VX)
                    // The value of each variable register from V0 to VX inclusive
                    // (if X is 0, then only V0) will be stored in successive memory addresses,
                    // starting with the one that’s stored in I.
                    // V0 will be stored at the address in I, V1 will be stored in I + 1,
                    // and so on, until VX is stored in I + X.
                    let x = nibble_2 as usize;
                    let i = self.index_register as usize;
                    for idx in 0..=x {
                        self.memory[i + idx] = self.registers[idx];
                    }
                }
                Instruction::LoadMemory => {
                    // FX65 (Load V0 to VX)
                    // Does the opposite of Store; it takes the value stored at the memory
                    // addresses and loads them into the variable registers instead.
                    let x = nibble_2 as usize;
                    let i = self.index_register as usize;
                    for idx in 0..=x {
                        self.registers[idx] = self.memory[i + idx];
                    }
                }
            }
        } else {
            eprintln!("Unimplemented opcode: {:#04x?}", opcode);
        }
    }
}
