use std::fmt;

// number of arguments for all except unknown
// word value for unknown

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub enum OpCodes {
    halt,
    set,
    push,
    pop,
    eq,
    gt,
    jmp,
    jt,
    jf,
    add,
    mult,
    mod_,
    and,
    or,
    not,
    rmem,
    wmem,
    call,
    ret,
    out,
    in_,
    nop,
    unknown(u16),
}

impl OpCodes {
    pub fn parse(val: u16) -> OpCodes {
        match val {
            0 => OpCodes::halt,
            1 => OpCodes::set,
            2 => OpCodes::push,
            3 => OpCodes::pop,
            4 => OpCodes::eq,
            5 => OpCodes::gt,
            6 => OpCodes::jmp,
            7 => OpCodes::jt,
            8 => OpCodes::jf,
            9 => OpCodes::add,
            10 => OpCodes::mult,
            11 => OpCodes::mod_,
            12 => OpCodes::and,
            13 => OpCodes::or,
            14 => OpCodes::not,
            15 => OpCodes::rmem,
            16 => OpCodes::wmem,
            17 => OpCodes::call,
            18 => OpCodes::ret,
            19 => OpCodes::out,
            20 => OpCodes::in_,
            21 => OpCodes::nop,
            _ => OpCodes::unknown(val),
        }
    }
    pub fn argcount(&self) -> usize {
        match self {
            OpCodes::halt => 0,
            OpCodes::set => 2,
            OpCodes::push => 1,
            OpCodes::pop => 1,
            OpCodes::eq => 3,
            OpCodes::gt => 3,
            OpCodes::jmp => 1,
            OpCodes::jt => 2,
            OpCodes::jf => 2,
            OpCodes::add => 3,
            OpCodes::mult => 3,
            OpCodes::mod_ => 3,
            OpCodes::and => 3,
            OpCodes::or => 3,
            OpCodes::not => 2,
            OpCodes::rmem => 2,
            OpCodes::wmem => 2,
            OpCodes::call => 1,
            OpCodes::ret => 0,
            OpCodes::out => 1,
            OpCodes::in_ => 1,
            OpCodes::nop => 0,
            _ => panic!("argcount on unknown opcode"),
        }
    }
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
