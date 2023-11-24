#[allow(non_snake_case)]
pub mod emulator {
    use macroquad::input;
    use macroquad::prelude::*;
    use std::fs::File;
    use std::io;
    use std::io::prelude::*;

    fn nibbles(u: u16) -> (u16, u16, u16, u16) {
        (
            (u & 0xF000) >> 12,
            (u & 0x0F00) >> 8,
            (u & 0x00F0) >> 4,
            u & 0x000F,
        )
    }

    pub fn keycode_from_hex(x: u8) -> input::KeyCode {
        match x {
            0 => input::KeyCode::Key0,
            1 => input::KeyCode::Key1,
            2 => input::KeyCode::Key2,
            3 => input::KeyCode::Key3,
            4 => input::KeyCode::Key4,
            5 => input::KeyCode::Key5,
            6 => input::KeyCode::Key6,
            7 => input::KeyCode::Key7,
            8 => input::KeyCode::Key8,
            9 => input::KeyCode::Key9,
            10 => input::KeyCode::A,
            11 => input::KeyCode::B,
            12 => input::KeyCode::C,
            13 => input::KeyCode::D,
            14 => input::KeyCode::E,
            15 => input::KeyCode::F,
            _ => input::KeyCode::Z,
        }
    }

    #[derive(Default)]
    struct Timer {
        sound: u8,
        delay: u8,
    }

    #[derive(Default)]
    struct Register {
        v: [u8; 16],
        i: u16,
    }
    pub struct Screen {
        pixels: [bool; 2048],
        cols: usize,
        rows: usize,
        pixel_size: usize,
    }
    impl Screen {
        pub fn new() -> Self {
            Screen {
                pixels: [false; 2048],
                cols: 64,
                rows: 32,
                pixel_size: 24,
            }
        }

        pub fn set(&mut self, row: usize, col: usize, val: bool) -> u8 {
            let mut ans = 0;

            let row_ = row % self.rows;
            let col_ = col % self.cols;
            if self.pixels[row_ * self.cols + col_] && val {
                ans = 1;
            }
            self.pixels[row_ * self.cols + col_] ^= val;
            ans
        }

        pub fn draw(&self) {
            for row in 0..self.rows {
                for col in 0..self.cols {
                    if self.pixels[row * self.cols + col] {
                        draw_rectangle(
                            (col * self.pixel_size) as f32,
                            (row * self.pixel_size) as f32,
                            (self.pixel_size) as f32,
                            self.pixel_size as f32,
                            WHITE,
                        )
                    }
                }
            }
        }
    }
    pub struct Keyboard {
        pub keymap: [bool; 16],
    }
    impl Keyboard {
        fn new() -> Self {
            Keyboard {
                keymap: [false; 16],
            }
        }
    }
    pub struct Chip8 {
        registers: Register,
        timers: Timer,
        screen: Screen,
        memory: [u8; 4096],
        stack: Vec<u16>,
        pc: u16,
        pub keyboard: Keyboard,
    }

    impl Chip8 {
        pub fn new() -> Self {
            Chip8 {
                registers: Register::default(),
                timers: Timer::default(),
                screen: Screen::new(),
                memory: [0; 4096],
                stack: Vec::new(),
                pc: 0x200,
                keyboard: Keyboard::new(),
            }
        }

        fn load(&mut self, program: &[u8]) {
            //                self.memory[addr] = program[addr - 0x200];
            self.memory[0x200..0x200 + program.len()].copy_from_slice(program);
            let v = [
                0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
                0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
                0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
                0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
                0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
                0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
            ];

            self.memory[0..80].copy_from_slice(&v);
        }

        pub fn load_from_file(&mut self, file_name: &str) -> Result<(), io::Error> {
            let mut f = File::open(file_name)?;
            let mut buffer = Vec::new();

            f.read_to_end(&mut buffer)?;

            self.load(&buffer);

            Ok(())
        }

        pub fn run(&mut self) {
            for i in 0..16 {
                self.keyboard.keymap[i] = is_key_down(keycode_from_hex(i as u8));
            }

            self.screen.draw();
            let ins = ((self.memory[self.pc as usize] as u16) << 8)
                | (self.memory[self.pc as usize + 1]) as u16;
            self.execute_instruction(ins);

            if self.timers.delay > 0 {
                self.timers.delay -= 1;
            }
            if self.timers.sound > 0 {
                self.timers.sound -= 1;
            }
        }

        fn op00E0(&mut self) {
            self.screen.pixels = [false; 2048];
            self.pc += 2;
        }
        fn op00EE(&mut self) {
            self.pc = self.stack.pop().unwrap() + 2;
        }
        fn op1nnn(&mut self, nnn: u16) {
            self.pc = nnn;
        }
        fn op2nnn(&mut self, nnn: u16) {
            self.stack.push(self.pc);
            self.pc = nnn;
        }
        fn op3xkk(&mut self, x: usize, kk: u8) {
            self.pc += 2;
            if self.registers.v[x] == kk {
                self.pc += 2;
            }
        }
        fn op4xkk(&mut self, x: usize, kk: u8) {
            self.pc += 2;
            if self.registers.v[x] != kk {
                self.pc += 2;
            }
        }
        fn op5xy0(&mut self, x: usize, y: usize) {
            self.pc += 2;
            if self.registers.v[x] == self.registers.v[y] {
                self.pc += 2;
            }
        }
        fn op6xkk(&mut self, x: usize, kk: u8) {
            self.registers.v[x] = kk;
            self.pc += 2;
        }
        fn op7xkk(&mut self, x: usize, kk: u8) {
            self.registers.v[x] = self.registers.v[x].wrapping_add(kk);
            self.pc += 2;
        }
        fn op8xy0(&mut self, x: usize, y: usize) {
            self.registers.v[x] = self.registers.v[y];
            self.pc += 2;
        }
        fn op8xy1(&mut self, x: usize, y: usize) {
            self.registers.v[x] |= self.registers.v[y];
            self.registers.v[0xf] = 0;
            self.pc += 2;
        }
        fn op8xy2(&mut self, x: usize, y: usize) {
            self.registers.v[x] &= self.registers.v[y];
            self.registers.v[0xf] = 0;
            self.pc += 2
        }
        fn op8xy3(&mut self, x: usize, y: usize) {
            self.registers.v[x] ^= self.registers.v[y];
            self.registers.v[0xf] = 0;
            self.pc += 2;
        }
        fn op8xy4(&mut self, x: usize, y: usize) {
            let val: u16 = (self.registers.v[x] as u16) + (self.registers.v[y] as u16);
            self.registers.v[x] = self.registers.v[x].wrapping_add(self.registers.v[y]);

            if val > 255 {
                self.registers.v[0xf] = 1;
            } else {
                self.registers.v[0xf] = 0;
            }
            self.pc += 2;
        }
        fn op8xy5(&mut self, x: usize, y: usize) {
            let xx = self.registers.v[x];
            let yy = self.registers.v[y];

            self.registers.v[x] = self.registers.v[x].wrapping_sub(self.registers.v[y]);

            if xx < yy {
                self.registers.v[0xf] = 0;
            } else {
                self.registers.v[0xf] = 1;
            }
            self.pc += 2;
        }
        fn op8xy6(&mut self, x: usize, _y: usize) {
            let xx = self.registers.v[x];
            self.registers.v[x] >>= 1;
            self.registers.v[0xf] = xx & 1;
            self.pc += 2;
        }
        fn op8xy7(&mut self, x: usize, y: usize) {
            let xx = self.registers.v[x];
            let yy = self.registers.v[y];

            self.registers.v[x] = self.registers.v[y].wrapping_sub(self.registers.v[x]);

            if yy < xx {
                self.registers.v[15] = 0;
            } else {
                self.registers.v[15] = 1;
            }
            self.pc += 2;
        }
        fn op8xyE(&mut self, x: usize, _y: usize) {
            let xx = self.registers.v[x];
            self.registers.v[x] <<= 1;
            self.registers.v[15] = (xx & 0b10000000) >> 7;
            self.pc += 2;
        }
        fn op9xy0(&mut self, x: usize, y: usize) {
            self.pc += 2;
            if self.registers.v[x] != self.registers.v[y] {
                self.pc += 2;
            }
        }
        fn opAnnn(&mut self, nnn: u16) {
            self.registers.i = nnn;
            self.pc += 2;
        }
        fn opBnnn(&mut self, nnn: u16) {
            self.pc = nnn + (self.registers.v[0] as u16);
        }
        fn opCxkk(&mut self, x: usize, kk: u8) {
            self.registers.v[x] = macroquad::rand::gen_range(0, 255) & kk;
            self.pc += 2;
        }
        fn opDxyn(&mut self, x: usize, y: usize, n: u8) {
            self.registers.v[15] = 0;

            for byte in 0..n {
                for bit in 0..8 {
                    let pixel =
                        (self.memory[self.registers.i as usize + byte as usize] >> (7 - bit)) & 1;
                    self.registers.v[0xf] |= self.screen.set(
                        ((self.registers.v[y] as u16) + byte as u16) as usize,
                        ((self.registers.v[x] as u16) + (bit as u16)) as usize,
                        pixel == 1,
                    );
                }
            }

            self.pc += 2;
        }
        fn opEx9E(&mut self, x: usize) {
            self.pc += 2;

            if self.keyboard.keymap[self.registers.v[x] as usize] {
                self.pc += 2;
            }
        }
        fn opExA1(&mut self, x: usize) {
            self.pc += 2;
            if !self.keyboard.keymap[self.registers.v[x] as usize] {
                self.pc += 2;
            }
        }
        fn opFx07(&mut self, x: usize) {
            self.registers.v[x] = self.timers.delay;
            self.pc += 2;
        }
        fn opFx0A(&mut self, x: usize) {
            for i in 0..16 {
                println!("{}", i);
                if self.keyboard.keymap[i as usize] {
                    println!("HIT {}", i);
                    self.registers.v[x] = i;
                    self.pc += 2;
                    return;
                }
            }
        }
        fn opFx15(&mut self, x: usize) {
            self.timers.delay = self.registers.v[x];
            self.pc += 2;
        }
        fn opFx18(&mut self, x: usize) {
            self.timers.sound = self.registers.v[x];
            self.pc += 2;
        }

        fn opFx1E(&mut self, x: usize) {
            self.registers.i += self.registers.v[x] as u16;
            self.pc += 2;
        }

        fn opFx29(&mut self, x: usize) {
            self.registers.i = (self.registers.v[x] as u16) * 5;
            self.pc += 2;
        }
        fn opFx33(&mut self, x: usize) {
            let xx = self.registers.v[x];
            self.memory[self.registers.i as usize] = xx / 100;
            self.memory[self.registers.i as usize + 1] = (xx / 10) % 10;
            self.memory[self.registers.i as usize + 2] = xx % 10;
            self.pc += 2;
        }
        fn opFx55(&mut self, x: usize) {
            for i in 0..x + 1 {
                self.memory[self.registers.i as usize + i] = self.registers.v[i];
            }
            self.registers.i += x as u16 + 1;
            self.pc += 2;
        }
        fn opFx65(&mut self, x: usize) {
            for i in 0..x + 1 {
                self.registers.v[i] = self.memory[self.registers.i as usize + i];
            }
            self.registers.i += x as u16 + 1;
            self.pc += 2;
        }

        pub fn execute_instruction(&mut self, ins: u16) {
            let x = ((ins & 0x0F00) >> 8) as usize;
            let y = ((ins & 0x00F0) >> 4) as usize;
            let nnn = ins & 0x0FFF;
            let kk = (ins & 0x00FF) as u8;
            let n = (ins & 0x000F) as u8;

            match nibbles(ins) {
                (0x0, 0x0, 0xE, 0xE) => self.op00EE(),
                (0x0, _, _, _) => self.op00E0(),
                (0x1, _, _, _) => self.op1nnn(nnn),
                (0x2, _, _, _) => self.op2nnn(nnn),
                (0x3, _, _, _) => self.op3xkk(x, kk),
                (0x4, _, _, _) => self.op4xkk(x, kk),
                (0x5, _, _, _) => self.op5xy0(x, y),
                (0x6, _, _, _) => self.op6xkk(x, kk),
                (0x7, _, _, _) => self.op7xkk(x, kk),
                (0x8, _, _, 0x0) => self.op8xy0(x, y),
                (0x8, _, _, 0x1) => self.op8xy1(x, y),
                (0x8, _, _, 0x2) => self.op8xy2(x, y),
                (0x8, _, _, 0x3) => self.op8xy3(x, y),
                (0x8, _, _, 0x4) => self.op8xy4(x, y),
                (0x8, _, _, 0x5) => self.op8xy5(x, y),
                (0x8, _, _, 0x6) => self.op8xy6(x, y),
                (0x8, _, _, 0x7) => self.op8xy7(x, y),
                (0x8, _, _, 0xE) => self.op8xyE(x, y),
                (0x9, _, _, _) => self.op9xy0(x, y),
                (0xA, _, _, _) => self.opAnnn(nnn),
                (0xB, _, _, _) => self.opBnnn(nnn),
                (0xC, _, _, _) => self.opCxkk(x, kk),
                (0xD, _, _, _) => self.opDxyn(x, y, n),
                (0xE, _, _, 0xE) => self.opEx9E(x),
                (0xE, _, _, 0x1) => self.opExA1(x),
                (0xF, _, 0x0, 0x7) => self.opFx07(x),
                (0xF, _, 0x0, 0xA) => self.opFx0A(x),
                (0xF, _, 0x1, 0x5) => self.opFx15(x),
                (0xF, _, 0x1, 0x8) => self.opFx18(x),
                (0xF, _, 0x1, 0xE) => self.opFx1E(x),
                (0xF, _, 0x2, _) => self.opFx29(x),
                (0xF, _, 0x3, _) => self.opFx33(x),
                (0xF, _, 0x5, _) => self.opFx55(x),
                (0xF, _, 0x6, _) => self.opFx65(x),

                _ => {
                    panic!("Invalid opcode {}", ins)
                }
            }
        }
    }
}
