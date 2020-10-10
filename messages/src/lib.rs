use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    None,
    Run,
    Step,
    Continue,
    Quit,
    AddBreakpoint(usize),
    RemoveBreakpoint(usize),
    Where,
}

impl Command {
    fn parse_argument_command(text: &str) -> Command {
        if text.starts_with("b ") {
            let parts: Vec<&str> = text.split(" ").collect();
            assert_eq!(parts.len(), 2);
            let address = parts[1].parse::<usize>().unwrap();
            return Command::AddBreakpoint(address);
        }
        panic!("Unknown command {:?}", text);
    }
    pub fn parse(text: &str) -> Command {
        match text {
            "run" | "r" => Command::Run,
            "step" | "s" => Command::Step,
            "continue" | "c" => Command::Continue,
            "quit" | "q" => Command::Quit,
            _ => Command::parse_argument_command(text),
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self).unwrap();
        Vec::from(json.as_bytes())
    }
    pub fn deserialize(data: &[u8]) -> Command {
        let text = String::from_utf8_lossy(&data);
        serde_json::from_str(&text).unwrap()
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Request(Command),
    Response(ResponseData),
    Responses(Vec<ResponseData>),
}

impl Message {
    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self).unwrap();
        Vec::from(json.as_bytes())
    }

    pub fn deserialize(data: &[u8]) -> Message {
        let text = String::from_utf8_lossy(&data);
        serde_json::from_str(&text).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseData {
    Empty,
    Text(String),
    State(VmState),
}

impl ResponseData {
    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self).unwrap();
        Vec::from(json.as_bytes())
    }

    pub fn deserialize(data: &[u8]) -> Message {
        let text = String::from_utf8_lossy(&data);
        serde_json::from_str(&text).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VmState {
    pub registers: [u16; 8],
    pub ip: usize,
    pub count: u32,
    pub here: [u16; 20],
}

impl VmState {
    pub fn from(registers: [u16; 8], ip: usize, count: u32, here: &[u16]) -> VmState {
        let mut buff = [0u16; 20];
        for (idx, h) in here.iter().enumerate() {
            buff[idx] = *h;
        }
        VmState {
            registers,
            ip,
            count,
            here: buff,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self).unwrap();
        Vec::from(json.as_bytes())
    }

    pub fn deserialize(data: &[u8]) -> Message {
        let text = String::from_utf8_lossy(&data);
        serde_json::from_str(&text).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Command;

    #[test]
    fn serialize_deserialize() {
        let cmd = Command::Run;
        let json = cmd.serialize();
        let cmd2 = Command::deserialize(&json);
        assert_eq!(cmd, cmd2);
    }
}
