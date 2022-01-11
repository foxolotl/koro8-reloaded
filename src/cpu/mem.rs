use crate::{arch::Sprite, constants::{HEAP_SIZE, STACK_SIZE}};

pub struct Stack {
    sp: usize,
    stack: [u16; STACK_SIZE]
}

pub struct Heap([u8; HEAP_SIZE]);
pub struct Instr(u16);

impl Stack {
    pub fn new() -> Stack {
        Stack { sp: 0, stack: [0; STACK_SIZE] }
    }

    pub fn reset(&mut self) {
        self.stack = [0; STACK_SIZE]
    }

    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp]
    }

    pub fn push(&mut self, val: u16) {
        self.stack[self.sp] = val;
        self.sp += 1;
    }
}

impl Heap {
    pub fn new() -> Heap {
        Heap([0; HEAP_SIZE])
    }

    pub fn reset(&mut self) {
        self.0 = Heap::new().0
    }

    pub fn write_bytes(&mut self, addr: u16, src: &[u8]) {
        let offset = addr as usize;
        self.0[offset .. offset + src.len()].copy_from_slice(src);
    }

    pub fn write_byte(&mut self, addr: u16, src: u8) {
        self.0[addr as usize] = src;
    }

    pub fn read_instr(&mut self, addr: u16) -> Instr {
        let hi = (self.0[addr as usize] as u16) << 8;
        let lo = self.0[addr as usize + 1] as u16;
        Instr(hi | lo)
    }

    pub fn read_bytes(&self, addr: u16, size: u8) -> &[u8] {
        let address = addr as usize;
        &self.0[address .. (address + size as usize)]
    }

    pub fn read_sprite(&self, addr: u16, size: u8) -> Sprite {
        Sprite(self.read_bytes(addr, size))
    }
}

impl Instr {
    pub fn addr(&self) -> u16 {
        self.0 & 0x0FFF
    }

    pub fn nibble(&self) -> u8 {
        (self.0 & 0x000F) as u8
    }

    pub fn byte(&self) -> u8 {
        self.0 as u8
    }

    pub fn x(&self) -> u8 {
        ((self.0 & 0x0F00) >> 8) as u8
    }

    pub fn y(&self) -> u8 {
        ((self.0 & 0x00F0) >> 4) as u8
    }

    pub fn instr(&self) -> u16 {
        self.0
    }
}
