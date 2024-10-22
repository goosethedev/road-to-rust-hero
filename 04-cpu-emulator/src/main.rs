// CPU: CHIP-8
struct CPU {
    registers: [u8; 16],
    position_in_mem: usize,
    memory: [u8; 0x1000],
    stack: [u16; 16],
    stack_pointer: usize,
}

impl CPU {
    fn read_opcode(&self) -> u16 {
        let b1 = self.memory[self.position_in_mem] as u16;
        let b2 = self.memory[self.position_in_mem + 1] as u16;

        (b1 << 8) | b2
    }

    fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            self.position_in_mem += 2;

            let c = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let d = ((opcode & 0x000F) >> 0) as u8;

            let nnn = opcode & 0x0FFF;

            match (c, x, y, d) {
                (0, 0, 0, 0) => return,
                (0, 0, 0xE, 0xE) => self.ret(),    // Return
                (2, _, _, _) => self.call(nnn),    // Caller
                (8, _, _, 4) => self.add_xy(x, y), // Adder
                _ => todo!("Unhandled opcode: {:?}", opcode),
            }
        }
    }

    // Perform a subroutine execution at addr
    fn call(&mut self, addr: u16) {
        let stack = &mut self.stack;

        if self.stack_pointer > stack.len() {
            panic!("Stack overflow!");
        }

        stack[self.stack_pointer] = self.position_in_mem as u16;
        self.stack_pointer += 1;
        self.position_in_mem = addr as usize;
    }

    // Return a function
    fn ret(&mut self) {
        let stack = &mut self.stack;

        if self.stack_pointer <= 0 {
            panic!("Stack underflow!");
        }

        self.stack_pointer -= 1;
        self.position_in_mem = stack[self.stack_pointer] as usize;
    }

    // Perform sum and update registers
    fn add_xy(&mut self, xi: u8, yi: u8) {
        let x = self.registers[xi as usize];
        let y = self.registers[yi as usize];
        let (res, overflow) = x.overflowing_add(y);

        self.registers[xi as usize] = res;
        self.registers[0xF] = overflow.into();
    }
}

fn main() {
    // CPU initialization
    let mut cpu = CPU {
        registers: [0; 16],
        memory: [0; 0x1000],
        position_in_mem: 0,
        stack: [0; 16],
        stack_pointer: 0,
    };

    // Opcodes for function calls
    let mem = &mut cpu.memory;
    (mem[0x0000], mem[0x0001]) = (0x21, 0x00);
    (mem[0x0002], mem[0x0003]) = (0x21, 0x00);
    (mem[0x0004], mem[0x0005]) = (0x00, 0x00);

    // Opcodes of the function "add_twice"
    (mem[0x0100], mem[0x0101]) = (0x80, 0x14);
    (mem[0x0102], mem[0x0103]) = (0x80, 0x14);
    (mem[0x0104], mem[0x0105]) = (0x00, 0xEE);

    // Load operands in registers
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    // Execute the cpu
    cpu.run();

    // Obtain and print the result
    let result = cpu.registers[0];
    println!("5 + (2*10) + (2*10) = {}", result);
}
