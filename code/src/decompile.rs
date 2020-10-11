use std::fmt;
use std::path;

use crate::opcodes::OpCodes;
use crate::program::Program;

fn parse_arg(value: u16) -> String {
    match value {
        0..=32767 => value.to_string(),
        32768..=32775 => {
            let reg = value - 32768;
            format!("reg{}", reg)
        }
        _ => panic!("invalid value {}", value),
    }
}

#[derive(Debug, Clone)]
pub struct OpData {
    code: OpCodes,
    idx: usize,
    args: Vec<String>,
}

impl OpData {
    pub fn from(code: OpCodes, idx: usize) -> OpData {
        OpData {
            idx,
            code,
            args: Vec::new(),
        }
    }
    pub fn argtext(&self) -> String {
        let mut argtext = String::new();
        if self.code == OpCodes::out {
            for a in &self.args {
                argtext += &a;
            }
        } else {
            for a in &self.args {
                argtext += &format!("{} ", a);
            }
        }
        argtext.trim().to_string()
    }
}

impl fmt::Display for OpData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {} {}", self.idx, self.code, self.argtext())
    }
}

fn parse_op(code: &OpCodes, data: &[u16], idx: usize, argcount: usize) -> OpData {
    let mut retr = OpData::from(code.clone(), idx);
    if argcount == 0 {
        return retr;
    }
    if idx + 1 >= data.len() {
        retr.args.push("<arg0?>".to_string());
        return retr;
    }
    match code {
        OpCodes::out => retr.args.push(format!("{}", data[idx + 1] as u8 as char)),
        _ => retr.args.push(parse_arg(data[idx + 1])),
    };
    if argcount == 1 {
        return retr;
    }
    if idx + 2 >= data.len() {
        retr.args.push("<arg1?>".to_string());
        return retr;
    }
    retr.args.push(parse_arg(data[idx + 2]));
    if argcount == 2 {
        return retr;
    }
    retr.args.push(parse_arg(data[idx + 3]));
    retr
}

pub fn parse_data_offset(data: Vec<u16>, offset: usize) -> Vec<OpData> {
    let mut retr = Vec::new();
    let mut idx = 0;
    while idx < data.len() {
        let value = OpCodes::parse(data[idx]);
        match value {
            OpCodes::unknown(_) => idx += 1,
            _ => {
                retr.push(parse_op(&value, &data, idx + offset, value.argcount()));
                idx += 1 + value.argcount();
            }
        }
    }
    retr
}

pub fn parse_data(data: Vec<u16>) -> Vec<OpData> {
    parse_data_offset(data, 0)
}

pub fn parse_file(file: path::PathBuf) -> Vec<OpData> {
    let program = Program::parse_file(file.as_path());
    parse_data(program.data)
}

pub fn cleanup(input: Vec<OpData>) -> Vec<OpData> {
    let mut output = Vec::new();
    let mut idx = 0;
    while idx < input.len() {
        let curr = input[idx].clone();
        if curr.code != OpCodes::out {
            output.push(curr.clone());
            idx += 1;
            continue;
        }
        let mut newdata = OpData::from(curr.code, curr.idx);
        let mut args = vec![curr.args[0].clone()];
        let mut count = 1;
        while idx + count < input.len() && input[idx + count].code == OpCodes::out {
            args.push(input[idx + count].args[0].clone());
            count += 1;
        }
        idx += count;
        newdata.args = args;
        output.push(newdata);
    }
    output
}

pub fn serialize(input: Vec<OpData>) -> String {
    let mut retr = String::new();
    for i in input {
        retr += &format!("{}\n", i);
    }
    retr
}
