#[allow(non_camel_case_types)]
#[derive(Debug)]
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
    pub fn parse(val : u16) -> OpCodes {
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
}
