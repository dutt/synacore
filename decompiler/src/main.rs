use std::env;

use std::io::prelude::*;
use std::fs;
use std::path;

use code::program::Program;
use code::opcodes::OpCodes;

fn parse_arg(value : u16) -> String {
    match value {
        0..=32767 => value.to_string(),
        32768..=32775 => {
            let reg = value - 32768;
            format!("reg{}", reg)
        },
        _ => panic!("invalid value {}", value),
    }
}

#[derive(Debug, Clone)]
struct OpData {
    code : OpCodes,
    idx : usize,
    args : Vec<String>
}

impl OpData {
    pub fn from(code : OpCodes, idx : usize) -> OpData {
        OpData {
            idx,
            code,
            args : Vec::new()
        }
    }
}

fn parse_op(code : &OpCodes, data : &[u16], idx : usize, argcount : usize) -> OpData {
    let mut retr = OpData::from(code.clone(), idx);
    if argcount == 0 {
        return retr;
    }
    match code {
        OpCodes::out => retr.args.push(format!("{}", data[idx+1] as u8 as char)),
        _            => retr.args.push(parse_arg(data[idx+1])),
    };
    if argcount == 1 {
        return retr;
    }
    retr.args.push(parse_arg(data[idx+2]));
    if argcount == 2 {
        return retr;
    }
    retr.args.push(parse_arg(data[idx+3]));
    retr
}

fn parse(file: path::PathBuf) -> Vec<OpData> {
    let program = Program::parse_file(file.as_path());
    let mut retr = Vec::new();
    let mut idx = 0;
    while idx < program.data.len() {
        let value = OpCodes::parse(program.data[idx]);
        match value {
            OpCodes::unknown(_) => idx += 1,
            _ => {
                retr.push(parse_op(&value, &program.data, idx, value.argcount()));
                idx += 1 + value.argcount();
            },
        }
    }
    retr
}

fn cleanup(input : Vec<OpData>) -> Vec<OpData> {
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
            args.push(input[idx+count].args[0].clone());
            count += 1;
        }
        idx += count;
        newdata.args = args;
        output.push(newdata);
    }
    output
}

fn serialize(input : Vec<OpData>) -> String {
    let mut retr = String::new();
    for i in input {
        let mut argtext = String::new();
        if i.code == OpCodes::out {
            for a in i.args {
                argtext += &a;
            }
        } else {
            for a in i.args {
                argtext += &format!("{} ", a);
            }

        }
        retr += &format!("{}: {} {}\n", i.idx, i.code, argtext.trim());
    }
    retr
}

fn get_outpath(inpath : &str) -> path::PathBuf {
    let inp = path::Path::new(inpath);
    let inp = inp.canonicalize().unwrap();
    let mut retr = path::PathBuf::from(inp);
    retr.set_extension("decompiled");
    retr
}

fn write(data : String, path : &path::Path) {
    let mut file = match fs::File::create(path) {
        Err(why) => panic!("Could not create file {} : {}", path.display(), why),
        Ok(file) => file,
    };
    match file.write_all(data.as_bytes()) {
        Err(why) => panic!("Failed to write file {}: {}", path.display(), why),
        Ok(_) => println!("done"),
    }
}

pub fn decompile(file : &str) {
    let data = parse(path::PathBuf::from(file.clone()));
    let clean_data = cleanup(data);
    let text = serialize(clean_data);

    let outpath = get_outpath(file);
    println!("writing to file {:?}", outpath);

    write(text, &outpath);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args {:?}", args);
    if args.len() < 2 {
        println!("No input file specified");
        return
    }

    let file = &args[1];
    println!("reading file {:}", file);

    decompile(file);
}
