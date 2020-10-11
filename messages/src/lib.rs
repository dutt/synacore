use serde::{Deserialize, Serialize};

pub mod command;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Request(command::Command),
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
    Dump(Vec<u16>),
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
