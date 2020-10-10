use std::collections::vec_deque;
use std::io;

use log::{debug, error, info};

use messages::VmState;

use code::opcodes::OpCodes;
use code::program::Program;

pub struct Host {
    registers: [u16; 8],
    stack: Vec<u16>,
    memory: Vec<u16>,
    ip: usize,
    input_buffer: vec_deque::VecDeque<u16>,
    halted: bool,
    count: u32, //number of instructions execued,
    pub program: Program,
}

fn is_memory(address: u16) -> bool {
    match address {
        0..=32767 => true,
        32768..=32775 => false,
        _ => panic!("invalid address: {}", address),
    }
}

fn is_register(address: u16) -> bool {
    !is_memory(address)
}

impl Host {
    pub fn new() -> Host {
        Host {
            registers: [0, 0, 0, 0, 0, 0, 0, 0],
            stack: Vec::new(),
            memory: Vec::new(),
            ip: 0,
            halted: false,
            input_buffer: vec_deque::VecDeque::new(),
            count: 0,
            program: Program::new(),
        }
    }
    pub fn from(program: Program) -> Host {
        let mut host = Host::new();
        host.memory = Vec::new();
        host.memory.resize(program.data.len(), 0);
        for (idx, val) in program.data.iter().enumerate() {
            host.memory[idx] = *val;
        }
        host.program = program;
        host
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn create_state(&self) -> VmState {
        VmState::from(
            self.registers,
            self.ip,
            self.count,
            &self.memory[self.ip..self.ip + 20],
        )
    }

    fn resolve(&self, value: u16) -> u16 {
        match value {
            0..=32767 => value,
            32768..=32775 => {
                let reg = value - 32768;
                debug!(
                    "  resolved {} => reg{} => {}",
                    value, reg, self.registers[reg as usize]
                );
                self.registers[reg as usize]
            }
            _ => panic!("invalid value {}", value),
        }
    }
    fn write(&mut self, address: u16, value: u16) {
        match address {
            0..=32767 => {
                if address as usize >= self.memory.len() {
                    debug!("  expanding memory to size {}", address);
                    self.memory.resize(address as usize + 1, 0);
                }
                debug!("  write memory[{}] = {}", address, value);
                self.memory[address as usize] = value;
            }
            32768..=32775 => {
                let reg = address - 32768;
                if reg > 7 {
                    panic!("invalid register {}", reg);
                }
                debug!("  write reg[{}] = {}", reg, value);
                self.registers[reg as usize] = value;
            }
            _ => panic!("invalid write target: {}", address),
        }
    }

    fn read(&self, address: u16) -> u16 {
        match address {
            0..=32767 => {
                if address as usize >= self.memory.len() {
                    panic!("read outsize of memory");
                }
                debug!(
                    "  read memory[{}] => {}",
                    address, self.memory[address as usize]
                );
                self.memory[address as usize]
            }
            32768..=32775 => {
                let reg = address - 32768;
                if reg > 7 {
                    panic!("invalid register {}", reg);
                }
                debug!("  read reg[{}] = {}", reg, self.registers[reg as usize]);
                self.registers[reg as usize]
            }
            _ => panic!("invalid read target: {}", address),
        }
    }

    pub fn should_run(&self) -> bool {
        self.ip < self.memory.len() && !self.halted
    }

    pub fn run(&mut self) {
        self.halted = false;
        while self.should_run() {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let inst = OpCodes::parse(self.memory[self.ip]);
        debug!("{}/{}, {:?}", self.ip, self.count, inst);
        self.ip += 1;
        match inst {
            OpCodes::halt => self.exec_halt(),
            OpCodes::set => self.exec_set(),
            OpCodes::push => self.exec_push(),
            OpCodes::pop => self.exec_pop(),
            OpCodes::eq => self.exec_eq(),
            OpCodes::gt => self.exec_gt(),
            OpCodes::jmp => self.exec_jmp(),
            OpCodes::jt => self.exec_jt(),
            OpCodes::jf => self.exec_jf(),
            OpCodes::add => self.exec_add(),
            OpCodes::mult => self.exec_mult(),
            OpCodes::mod_ => self.exec_mod(),
            OpCodes::and => self.exec_and(),
            OpCodes::or => self.exec_or(),
            OpCodes::not => self.exec_not(),
            OpCodes::rmem => self.exec_rmem(),
            OpCodes::wmem => self.exec_wmem(),
            OpCodes::call => self.exec_call(),
            OpCodes::ret => self.exec_ret(),
            OpCodes::out => self.exec_out(),
            OpCodes::in_ => self.exec_in(),
            OpCodes::nop => {}
            OpCodes::unknown(val) => {
                self.exec_halt();
                panic!("unknown instruction {:?}", val)
            }
        }
        self.count += 1;
    }

    fn exec_halt(&mut self) {
        self.halted = false;
        info!("  end memory: {:?}", &self.memory[844]);
        info!("  registers {:?}", self.registers);
        info!("  halt");
    }

    fn exec_set(&mut self) {
        let a = self.memory[self.ip];
        let mut b = self.memory[self.ip + 1];
        debug!("  set a {} b {}", a, b);
        if is_register(b) {
            b = self.resolve(b);
        }

        debug!("  set reg[{}] = {}", a, b);
        self.write(a, b);
        self.ip += 2;
    }

    fn exec_push(&mut self) {
        let a = self.memory[self.ip];
        let ra = self.resolve(a);
        debug!(" push a {} ra {}", a, ra);
        self.stack.push(ra);
        self.ip += 1;
    }

    fn exec_pop(&mut self) {
        if self.stack.is_empty() {
            error!(" pop empty stack");
            self.exec_halt();
        }
        let val = self.stack.pop();
        let a = self.memory[self.ip];
        debug!(" pop a {}", a);
        self.write(a, val.unwrap());
        self.ip += 1;
    }

    fn exec_eq(&mut self) {
        let a = self.memory[self.ip];
        let mut b = self.memory[self.ip + 1];
        let mut c = self.memory[self.ip + 2];
        debug!("  eq a {}, b {}, c {}", a, b, c);
        b = self.resolve(b);
        c = self.resolve(c);
        debug!("  eq final a {} b {} c {}", a, b, c);
        if b == c {
            debug!("  eq reg[{}] = 1", a);
            self.write(a, 1);
        } else {
            debug!("  eq reg[{}] = 0", a);
            self.write(a, 0);
        }
        self.ip += 3;
    }

    fn exec_gt(&mut self) {
        let a = self.memory[self.ip];
        let mut b = self.memory[self.ip + 1];
        let mut c = self.memory[self.ip + 2];
        debug!("  gt a {}, b {}, c {}", a, b, c);
        b = self.resolve(b);
        c = self.resolve(c);
        debug!("  gt final a {} b {} c {}", a, b, c);
        if b > c {
            debug!("  gt write [{}] = 1", a);
            self.write(a, 1);
        } else {
            debug!("  gt write [{}] = 0", a);
            self.write(a, 0);
        }
        self.ip += 3;
    }

    fn exec_jmp(&mut self) {
        let a = self.memory[self.ip];
        debug!("  at {}, jmp to {:?}", self.ip, a);
        self.ip = a as usize;
    }

    fn exec_jt(&mut self) {
        let a = self.memory[self.ip];
        let ra = self.resolve(a);
        let b = self.memory[self.ip + 1];
        let rb = self.resolve(b);
        debug!("  jt a {} ra {} b {} rb {}", a, ra, b, rb);
        if ra != 0 {
            debug!("  at {}, jt to {:?}", self.ip, b);
            self.ip = b as usize;
        } else {
            self.ip += 2;
        }
    }

    fn exec_jf(&mut self) {
        let a = self.memory[self.ip];
        let ra = self.resolve(a);
        let b = self.memory[self.ip + 1];
        debug!("  jf a {} ra {} b {}", a, ra, b);
        if ra == 0 {
            debug!("  at {}, jf to {:?}", self.ip, b);
            self.ip = b as usize;
        } else {
            self.ip += 2;
        }
    }

    fn exec_add(&mut self) {
        let a = self.memory[self.ip];
        let mut b = self.memory[self.ip + 1];
        let mut c = self.memory[self.ip + 2];
        debug!("  add a {}, b {}, c {}", a, b, c);
        b = self.resolve(b);
        c = self.resolve(c);
        debug!("  add final a {}, b {}, c {}", a, b, c);
        self.write(a, (b + c) % 32768);
        self.ip += 3
    }

    fn exec_mult(&mut self) {
        let a = self.memory[self.ip];
        let mut b = self.memory[self.ip + 1];
        let mut c = self.memory[self.ip + 2];
        debug!("  mult a {}, b {}, c {}", a, b, c);
        b = self.resolve(b);
        c = self.resolve(c);
        debug!("  mult final a {}, b {}, c {}", a, b, c);
        let large = b as u32 * c as u32;
        let bounded = large % 32768;
        self.write(a, bounded as u16);
        self.ip += 3
    }

    fn exec_mod(&mut self) {
        let a = self.memory[self.ip];
        let mut b = self.memory[self.ip + 1];
        let mut c = self.memory[self.ip + 2];
        debug!("  mod a {}, b {}, c {}", a, b, c);
        b = self.resolve(b);
        c = self.resolve(c);
        debug!("  mod final a {}, b {}, c {}", a, b, c);
        self.write(a, (b % c) % 32768);
        self.ip += 3
    }

    fn exec_and(&mut self) {
        let a = self.memory[self.ip];
        let b = self.memory[self.ip + 1];
        let c = self.memory[self.ip + 2];
        debug!("  and a {} b {} c {}", a, b, c);
        let rb = self.resolve(b);
        let rc = self.resolve(c);
        debug!("  and rb {}, rc {}", rb, rc);

        let val = rb & rc;
        debug!("  and reg[{}] = {}", a, val);
        self.write(a, val);

        self.ip += 3;
    }

    fn exec_or(&mut self) {
        let a = self.memory[self.ip];
        let b = self.memory[self.ip + 1];
        let c = self.memory[self.ip + 2];
        debug!("  or a {} b {} c {}", a, b, c);
        let rb = self.resolve(b);
        let rc = self.resolve(c);
        debug!("  or rb {}, rc {}", rb, rc);

        let val = rb | rc;
        debug!("  or reg[{}] = {}", a, val);
        self.write(a, val);

        self.ip += 3;
    }

    fn exec_not(&mut self) {
        let a = self.memory[self.ip];
        let b = self.memory[self.ip + 1];
        debug!("  not a {} b {}", a, b);
        let rb = self.resolve(b);
        debug!(" not rb {}", rb);

        let mut val = !rb;
        if val > 32768 {
            val -= 32768;
        }
        debug!("  not reg[{}] = {}", a, val);
        self.write(a, val);

        self.ip += 2;
    }

    fn exec_rmem(&mut self) {
        let a = self.memory[self.ip];
        let mut b = self.memory[self.ip + 1];
        debug!("  rmem a {}, b {}", a, b);

        if !is_memory(b) {
            let b2 = self.resolve(b);
            debug!("  rmem resolve b {} = {}", b, b2);
            b = b2;
        }
        let b = self.read(b);

        if is_memory(a) {
            error!("  rmem only writes to registers, a is memory, {}", b);
            self.exec_halt();
        }
        debug!("  rmem final a {}, b {}", a, b);
        self.write(a, b);
        self.ip += 2;
    }

    fn exec_wmem(&mut self) {
        let a = self.memory[self.ip];
        let b = self.memory[self.ip + 1];
        let ra = self.resolve(a);
        let rb = self.resolve(b);
        if !is_memory(ra) {
            error!("  rmem only writes to memory, a is not, {}", b);
            self.exec_halt();
        }
        debug!("  wmem a {}, b {}, rb {}", a, b, rb);
        self.write(ra, rb);
        self.ip += 2;
    }

    fn exec_call(&mut self) {
        let mut a = self.memory[self.ip];
        debug!("  call a {}", a);
        while !is_memory(a) {
            let a2 = self.resolve(a);
            debug!("  call a {} => {}", a, a2);
            a = a2;
        }
        let next_inst = self.ip + 1;
        if a == 0 {
            error!("  jumping to address 0");
            self.exec_halt();
        }
        debug!(" call push ip {}", next_inst);
        debug!(" call jumping to {}", a);
        self.stack.push(next_inst as u16);
        self.ip = a as usize;
    }

    fn exec_ret(&mut self) {
        if self.stack.is_empty() {
            error!(" ret empty stack");
            self.exec_halt();
        }
        let val = self.stack.pop().unwrap();
        self.ip = val as usize;
    }

    fn exec_out(&mut self) {
        let a = self.resolve(self.memory[self.ip]);
        print!("{}", a as u8 as char);
        self.ip += 1;
    }

    fn exec_in(&mut self) {
        let a = self.memory[self.ip];
        info!("  in a {}", a);

        if self.input_buffer.is_empty() {
            let mut line = String::new();
            let numbytes = io::stdin().read_line(&mut line).unwrap();
            info!("  in line '{}' numbytes {}", line, numbytes);
            for chr in line.chars() {
                self.input_buffer.push_back(chr as u16);
            }
        }
        let val = self.input_buffer.pop_front().unwrap();
        debug!("  writing {}/{} at {}", val, val as u8 as char, a);
        self.ip += 1;
    }
}
