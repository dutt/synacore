use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    Run,
    Step,
    Continue,
    Quit,
}

impl Command {
    pub fn parse(text : &str) -> Command {
        match text {
            "run" => Command::Run,
            "step" => Command::Step,
            "continue" => Command::Continue,
            "quit" => Command::Quit,
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
