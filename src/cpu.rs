mod font;
mod mem;

use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;
use crate::arch::{Display, Keyboard, Buzzer};
use crate::constants::{NUM_REGS, RESET_VECTOR, TIMER_HZ, LAST_REG};

use self::mem::Instr;

struct Regs {
    v: [u8; NUM_REGS as usize],
    i: u16,
    pc: u16,
    dt: u8,
    st: u8
}

impl Regs {
    fn new() -> Regs {
        Regs {
            v: [0; NUM_REGS as usize],
            i: 0,
            pc: RESET_VECTOR,
            dt: 0,
            st: 0
        }
    }
}

pub struct CPU<'t> {
    display: &'t mut dyn Display,
    keyboard: Box<dyn Keyboard>,
    buzzer: Box<dyn Buzzer>,
    rng: Box<dyn rand::RngCore>,
    clock_multiplier: u64,
    cycle_time_nanos: u64,
    cycle_sleep_millis: u64,

    next_cycle_deadline: u64,
    cycles: u64,

    regs: Regs,
    heap: mem::Heap,
    stack: mem::Stack,
    rom: &'t[u8]
}

pub fn new<'t>(
    display: &'t mut dyn Display,
    keyboard: Box<dyn Keyboard>,
    buzzer: Box<dyn Buzzer>,
    rng: Box<dyn rand::RngCore>,
    clock_multiplier: u64
) -> CPU<'t> {
    let cycles_per_second = clock_multiplier * TIMER_HZ;
    let cycle_time_nanos = 1_000_000_000 / cycles_per_second;
    CPU {
        display: display,
        keyboard: keyboard,
        buzzer: buzzer,
        rng: rng,
        clock_multiplier: clock_multiplier,
        cycle_time_nanos: cycle_time_nanos,
        cycle_sleep_millis: std::cmp::max(1, cycle_time_nanos / 1_000_000),
        next_cycle_deadline: 0,
        regs: Regs::new(),
        cycles: 0,
        heap: mem::Heap::new(),
        stack: mem::Stack::new(),
        rom: &[0;0],
    }
}

impl <'t> CPU<'t> {
    pub fn load(&mut self, rom: &'t[u8]) {
        self.rom = rom;
        self.reset()
    }

    pub fn reset(&mut self) {
        self.regs = Regs::new();
        self.cycles = 0;
        self.heap.reset();
        self.stack.reset();
        self.display.reset();
        self.keyboard.reset();
        self.buzzer.reset();
        self.heap.write_bytes(0, &font::FONT_DATA);
        self.heap.write_bytes(RESET_VECTOR, self.rom);
    }

    pub fn run(&mut self) {
        self.next_cycle_deadline = CPU::now();
        self.execute(u64::max_value())
    }

    fn execute(&mut self, mut ticks: u64) {
        while ticks > 0 {
            let now = CPU::now();
            if now >= self.next_cycle_deadline {
                self.step();
                self.next_cycle_deadline += self.cycle_time_nanos;
                ticks -= 1;
            } else {
                thread::sleep(Duration::from_millis(self.cycle_sleep_millis));
                if self.keyboard.power_off_signal() {
                    ticks = 0;
                }
                if self.keyboard.reset_signal() {
                    self.reset();
                }
            }                
        }
    }

    fn step(&mut self) {
        if self.cycles % self.clock_multiplier == 0 {
            if self.regs.dt != 0 {
                self.regs.dt = self.regs.dt - 1
            }
            if self.regs.st != 0 {
                self.regs.st = self.regs.st - 1
            }
        }
        let instr = self.heap.read_instr(self.regs.pc);
        self.regs.pc += 2;
        self.interpret(instr);
        self.cycles += 1;
    }

    fn interpret(&mut self, instruction: Instr) {
        match instruction.instr() & 0xF000 {
            0x0000 if instruction.instr() & 0xFF == 0xE0 => self.display.clear(),
            0x0000 if instruction.instr() & 0xFF == 0xEE => self.regs.pc = self.stack.pop(),
            0x0000 => { }, // SYS addr - ignored
            0x1000 => self.regs.pc = instruction.addr(),
            0x2000 => self.call(instruction.addr()),
            0x3000 => self.skip_if(self.regs.v[instruction.x() as usize] == instruction.byte()),
            0x4000 => self.skip_if(self.regs.v[instruction.x() as usize] != instruction.byte()),
            0x5000 => self.skip_if(self.regs.v[instruction.x() as usize] == self.regs.v[instruction.y() as usize]),
            0x6000 => self.regs.v[instruction.x() as usize] = instruction.byte(),
            0x7000 => self.regs.v[instruction.x() as usize] = self.regs.v[instruction.x() as usize].overflowing_add(instruction.byte()).0,
            0x8000 => self.interpret_0x8xxx(instruction),
            0x9000 => self.skip_if(self.regs.v[instruction.x() as usize] != self.regs.v[instruction.y() as usize]),
            0xA000 => self.regs.i = instruction.addr(),
            0xB000 => self.regs.pc = self.regs.v[0] as u16 + instruction.addr(),
            0xC000 => self.regs.v[instruction.x() as usize] = self.rng.gen::<u8>() & instruction.byte(),
            0xD000 => self.drw(instruction.nibble(), instruction.x(), instruction.y()),
            0xE000 => self.skip_on_key_state(instruction.byte() == 0x9E, instruction.x()),
            0xF000 => self.interpret_0xfxxx(instruction),
            _ => self.invalid_instruction(instruction)
        }
    }

    fn interpret_0x8xxx(&mut self, instruction: Instr) {
        match instruction.instr() & 0x000F {
            0x0 => self.regs.v[instruction.x() as usize] = self.regs.v[instruction.y() as usize],
            0x1 => self.regs.v[instruction.x() as usize] = self.regs.v[instruction.x() as usize] | self.regs.v[instruction.y() as usize],
            0x2 => self.regs.v[instruction.x() as usize] = self.regs.v[instruction.x() as usize] & self.regs.v[instruction.y() as usize],
            0x3 => self.regs.v[instruction.x() as usize] = self.regs.v[instruction.x() as usize] ^ self.regs.v[instruction.y() as usize],
            0x4 => self.add(instruction.x(), instruction.y()),
            0x5 => self.sub(instruction.x(), instruction.x(), instruction.y()),
            0x6 => self.shr(instruction.x()),
            0x7 => self.sub(instruction.x(), instruction.y(), instruction.x()),
            0xE => self.shl(instruction.x()),
            _ => self.invalid_instruction(instruction)
        }
    }

    fn interpret_0xfxxx(&mut self, instruction: Instr) {
        match instruction.instr() & 0x00FF {
            0x07 => self.regs.v[instruction.x() as usize] = self.regs.dt,
            0x0A => self.regs.v[instruction.x() as usize] = self.wait_for_key(),
            0x15 => self.regs.dt = self.regs.v[instruction.x() as usize],
            0x18 => self.ldst(instruction.x()),
            0x1E => self.regs.i += self.regs.v[instruction.x() as usize] as u16,
            0x29 => self.regs.i = 5 * self.regs.v[instruction.x() as usize] as u16,
            0x33 => self.bcd(instruction.x()),
            0x55 => self.heap.write_bytes(self.regs.i, &self.regs.v),
            0x65 => self.regs.v.copy_from_slice(self.heap.read_bytes(self.regs.i, NUM_REGS)),
            _ => self.invalid_instruction(instruction)
        }
    }

    fn invalid_instruction(&self, instruction: Instr) {
        panic!("invalid instruction: {:x}", instruction.instr())
    }

    fn drw(&mut self, nibble: u8, x: u8, y: u8) {
        let sprite = self.heap.read_sprite(self.regs.i, nibble);
        let collision = self.display.draw(&sprite, self.regs.v[x as usize], self.regs.v[y as usize]);
        self.regs.v[LAST_REG] = collision as u8
    }

    fn call(&mut self, addr: u16) {
        self.stack.push(self.regs.pc);
        self.regs.pc = addr;
    }

    fn bcd(&mut self, x: u8) {
        self.heap.write_byte(self.regs.i, self.regs.v[x as usize] / 100);
        self.heap.write_byte(self.regs.i + 1, (self.regs.v[x as usize] / 10) % 10);
        self.heap.write_byte(self.regs.i + 2, self.regs.v[x as usize] % 10);
    }

    fn skip_on_key_state(&mut self, skip_on_state: bool, x: u8) {
        let key_state = self.keyboard.pressed(self.regs.v[x as usize]);
        self.skip_if(key_state == skip_on_state)
    }

    fn wait_for_key(&mut self) -> u8 {
        let t0 = CPU::now();
        let key = self.keyboard.wait_key();
        let t1 = CPU::now();
        self.next_cycle_deadline += t1 - t0;
        key
    }

    fn skip_if(&mut self, skip: bool) {
        if skip {
            self.regs.pc += 2
        }
    }

    fn add(&mut self, x: u8, y: u8) {
        let (result, overflow) = self.regs.v[x as usize].overflowing_add(self.regs.v[y as usize]);
        self.regs.v[LAST_REG] = overflow as u8;
        self.regs.v[x as usize] = result;
    }

    fn sub(&mut self, r: u8, x: u8, y: u8) {
        let rx = self.regs.v[x as usize];
        let ry = self.regs.v[y as usize];
        self.regs.v[LAST_REG] = (rx > ry) as u8;
        self.regs.v[r as usize] = rx - ry;
    }

    fn shr(&mut self, x: u8) {
        let rx = self.regs.v[x as usize];
        self.regs.v[LAST_REG] = rx & 1;
        self.regs.v[x as usize] = rx >> 1;
    }

    fn shl(&mut self, x: u8) {
        let rx = self.regs.v[x as usize];
        self.regs.v[LAST_REG] = rx >> 7;
        self.regs.v[x as usize] = rx << 1;
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64
    }

    fn ldst(&mut self, x: u8) {
        let val = self.regs.v[x as usize];
        if self.regs.st == 0 && val > 0 {
            self.buzzer.start();
        }
        self.regs.st = val;
    }
}
