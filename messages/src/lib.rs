use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    None,
    Run,
    Step,
    Continue,
    Quit,
}

impl Command {
    pub fn parse(text : &str) -> Command {
        match text {
            "run" | "r" => Command::Run,
            "step" | "s" => Command::Step,
            "continue" | "c" => Command::Continue,
            "quit" | "q" => Command::Quit,
            _ => panic!("Unknown command {:?}", text),
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self).unwrap();
        Vec::from(json.as_bytes())
    }
    pub fn deserialize(data : &[u8]) -> Command {
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
}

impl Message {
    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self).unwrap();
        Vec::from(json.as_bytes())
    }

    pub fn deserialize(data : &[u8]) -> Message {
        let text = String::from_utf8_lossy(&data);
        serde_json::from_str(&text).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseData {
    Empty,
    State(VmState),
}

impl ResponseData {
    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self).unwrap();
        Vec::from(json.as_bytes())
    }

    pub fn deserialize(data : &[u8]) -> Message {
        let text = String::from_utf8_lossy(&data);
        serde_json::from_str(&text).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VmState {
    pub registers : [u16;8],
    pub ip : usize,
    pub count : u32,
}

impl VmState {
    pub fn from(registers : [u16;8], ip : usize, count : u32) -> VmState {
        VmState {
            registers,
            ip,
            count
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let json = serde_json::to_string(&self).unwrap();
        Vec::from(json.as_bytes())
    }

    pub fn deserialize(data : &[u8]) -> Message {
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
