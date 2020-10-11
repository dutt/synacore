use serde::{Deserialize, Serialize};
use std::fmt;

peg::parser! {
    grammar command_grammar() for str {
        rule number() -> usize
            = digits:$(['0'..='9']+) {
            usize::from_str_radix(digits, 10).unwrap()
        }

        rule run() -> Command
            = "run" {? Ok(Command::Run) }
            / "r" {? Ok(Command::Run) }
        rule step() -> Command
            = "step" {? Ok(Command::Step) }
            / "s" {? Ok(Command::Step) }
        rule continue() -> Command
            = "continue" {? Ok(Command::Continue) }
            / "c" {? Ok(Command::Continue) }
        rule quit() -> Command
            = "quit" {? Ok(Command::Quit) }
            / "q" {? Ok(Command::Quit) }

        rule add_bp() -> Command
            = "b " addr:number() {?
                Ok(Command::AddBreakpoint(addr))
            }
        rule del_bp() -> Command
            = "del " addr:number() {?
                Ok(Command::RemoveBreakpoint(addr))
            }

        rule print_reg() -> Command
            = "p reg " regnum:number() {?
                if regnum <= 8 {
                    Ok(Command::PrintRegister(regnum))
                } else {
                    Err("Not a valid register")
                }
            }
            / "pr " regnum:number() {?
                if regnum <= 8 {
                    Ok(Command::PrintRegister(regnum))
                } else {
                    Err("Not a valid register")
                }
            }
        rule print_mem() -> Command
            = "p mem " addr:number() " " len:number() {?
                Ok(Command::PrintMemory(addr, len))
            }
            / "p mem " addr:number() {?
                Ok(Command::PrintMemory(addr, 1))
            }
            / "pm " addr:number() " " len:number() {?
                Ok(Command::PrintMemory(addr, len))
            }
            / "pm " addr:number() {?
                Ok(Command::PrintMemory(addr, 1))
            }
        rule print() -> Command
            = print_reg()
            / print_mem()
            / expected!("Failed to print command")

        pub rule parse_command() -> Command
            = run()
            / step()
            / continue()
            / quit()
            / add_bp()
            / del_bp()
            / print()
            / expected!("Failed to parse command")
    }
}
use crate::command::command_grammar::parse_command;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
    //common
    None,
    Run,
    Step,
    Continue,
    Quit,
    AddBreakpoint(usize),
    RemoveBreakpoint(usize),
    PrintRegister(usize),
    PrintMemory(usize, usize),
}

impl Command {
    pub fn parse(text: &str) -> Result<Command, String> {
        let parse_result = parse_command(text);
        match parse_result {
            Ok(cmd) => Ok(cmd),
            Err(what) => Err(format!("failed to parse command '{}': {:?}", text, what)),
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

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn serialize_deserialize() {
        let cmd = Command::Run;
        let json = cmd.serialize();
        let cmd2 = Command::deserialize(&json);
        assert_eq!(cmd, cmd2);
    }

    #[test]
    fn test_command_parsing() {
        assert_eq!(Command::parse("r").unwrap(), Command::Run);
        assert_eq!(Command::parse("run").unwrap(), Command::Run);
        assert_eq!(Command::parse("s").unwrap(), Command::Step);
        assert_eq!(Command::parse("step").unwrap(), Command::Step);
        assert_eq!(Command::parse("c").unwrap(), Command::Continue);
        assert_eq!(Command::parse("continue").unwrap(), Command::Continue);
        assert_eq!(Command::parse("q").unwrap(), Command::Quit);
        assert_eq!(Command::parse("quit").unwrap(), Command::Quit);

        assert_eq!(Command::parse("b 5").unwrap(), Command::AddBreakpoint(5));
        assert_eq!(Command::parse("del 5").unwrap(), Command::RemoveBreakpoint(5));

        assert_eq!(Command::parse("p reg 5").unwrap(), Command::PrintRegister(5));
        assert_eq!(Command::parse("pr 5").unwrap(), Command::PrintRegister(5));

        assert_eq!(Command::parse("p mem 5").unwrap(), Command::PrintMemory(5, 1));
        assert_eq!(Command::parse("p mem 5 3").unwrap(), Command::PrintMemory(5, 3));
        assert_eq!(Command::parse("pm 5").unwrap(), Command::PrintMemory(5, 1));
        assert_eq!(Command::parse("pm 5 3").unwrap(), Command::PrintMemory(5, 3));
    }
}
