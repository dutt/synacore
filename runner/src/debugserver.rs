use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use log::{debug, error, info};

use crate::host;
use code::program;
use messages::command::Command;
use messages::{Message, ResponseData};

fn send(data: &[u8], stream: &mut TcpStream) -> std::io::Result<()> {
    let len: u64 = data.len() as u64;
    stream.write(&len.to_le_bytes())?;
    stream.write(&data)?;
    stream.flush()
}

fn send_string(text: String, stream: &mut TcpStream) -> std::io::Result<()> {
    //send(text.as_bytes(), stream)?;
    send_response(ResponseData::Text(text), stream)?;
    Ok(())
}

fn send_responses(data: Vec<ResponseData>, stream: &mut TcpStream) -> std::io::Result<()> {
    debug!("sending response {:?}", data);
    let response = Message::Responses(data);
    let jsondata = response.serialize();
    send(&jsondata, stream)?;
    Ok(())
}

fn send_response(data: ResponseData, stream: &mut TcpStream) -> std::io::Result<()> {
    debug!("sending response {:?}", data);
    let response = Message::Response(data);
    let jsondata = response.serialize();
    send(&jsondata, stream)?;
    Ok(())
}

fn recv_cmd(stream: &mut TcpStream) -> std::io::Result<Command> {
    // length
    let mut buffer = [0; 8];
    let numbytes = stream.take(8).read(&mut buffer)?;
    if numbytes == 0 {
        return Ok(Command::None);
    }
    assert_eq!(numbytes, 8);
    let len = u64::from_le_bytes(buffer);

    // data
    let mut data = Vec::new();
    stream.take(len).read_to_end(&mut data)?;
    if let Message::Request(cmd) = Message::deserialize(&data) {
        debug!("received request {:?}", cmd);
        Ok(cmd)
    } else {
        panic!("command not a request");
    }
}

pub struct Debugserver {
    host: host::Host,
    breakpoints: Vec<usize>,
    hit_breakpoint: usize,
}

impl Debugserver {
    pub fn start(program: program::Program) {
        let mut ds = Debugserver {
            host: host::Host::from(program),
            breakpoints: Vec::new(),
            hit_breakpoint: 0,
        };
        match ds.listen() {
            Ok(_) => {}
            Err(what) => panic!("Error in network communication {:?}", what),
        }
    }

    fn listen(&mut self) -> std::io::Result<()> {
        let listener = TcpListener::bind("0.0.0.0:6565")?;
        println!("server listening");
        for stream in listener.incoming() {
            self.handle(&mut stream?)?;
        }
        Ok(())
    }

    fn handle(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        println!("New connection {}", stream.peer_addr().unwrap());
        let mut greeting = String::from("Running ");
        greeting += self.host.program.path.to_str().unwrap();
        stream.write(greeting.as_bytes())?;
        stream.flush()?;
        loop {
            if let Ok(cmd) = recv_cmd(stream) {
                if let Ok(do_quit) = self.handle_cmd(cmd, stream) {
                    if do_quit {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_cmd(&mut self, cmd: Command, stream: &mut TcpStream) -> std::io::Result<bool> {
        match cmd {
            Command::None => Ok(false),
            Command::Quit => self.handle_quit(stream),
            Command::Run => self.handle_run(stream),
            Command::Step => self.handle_step(stream),
            Command::Continue => self.handle_continue(stream),
            Command::AddBreakpoint(address) => self.handle_add_breakpoint(address, stream),
            Command::RemoveBreakpoint(address) => self.handle_remove_breakpoint(address, stream),
            Command::PrintRegister(reg) => self.handle_print_register(stream),
            Command::PrintMemory(address, len) => self.handle_print_memory(address, len, stream),
            _ => panic!("unknown command {:?}", cmd),
        }
    }

    fn handle_quit(&mut self, _stream: &mut TcpStream) -> std::io::Result<bool> {
        println!("Client disconnected");
        // client shuts down the stream
        Ok(true)
    }

    fn handle_run(&mut self, stream: &mut TcpStream) -> std::io::Result<bool> {
        send_string("Running program...".to_string(), stream)?;
        self.run();
        let state = self.host.create_state();
        send_response(ResponseData::State(state), stream)?;
        Ok(false)
    }

    fn handle_step(&mut self, stream: &mut TcpStream) -> std::io::Result<bool> {
        self.host.step();
        let state = self.host.create_state();
        send_response(ResponseData::State(state), stream)?;
        Ok(false)
    }

    fn handle_continue(&mut self, stream: &mut TcpStream) -> std::io::Result<bool> {
        send_string("Continuing execution".to_string(), stream)?;
        self.run();
        let state = self.host.create_state();
        let mut responses = Vec::new();
        if self.hit_breakpoint != 0 {
            responses.push(ResponseData::Text(format!(
                "Hit breakpoint at {}",
                self.hit_breakpoint
            )));
        }
        responses.push(ResponseData::State(state));
        send_responses(responses, stream)?;
        Ok(false)
    }

    fn handle_add_breakpoint(
        &mut self,
        address: usize,
        stream: &mut TcpStream,
    ) -> std::io::Result<bool> {
        self.breakpoints.push(address);
        let bp_id = self.breakpoints.len();
        send_string(format!("Breakpint {} added at {}", bp_id, address), stream)?;
        Ok(false)
    }

    fn handle_remove_breakpoint(
        &mut self,
        address: usize,
        stream: &mut TcpStream,
    ) -> std::io::Result<bool> {
        if self.breakpoints.contains(&address) {
            self.breakpoints.remove(address);
            send_string(format!("Breakpint at {} removed", address), stream)?;
        } else {
            send_string(format!("No breakpint at {}", address), stream)?;
        }
        Ok(false)
    }

    fn handle_print_register(&mut self, stream : &mut TcpStream) -> std::io::Result<bool> {
        let state = self.host.create_state();
        send_response(ResponseData::State(state), stream)?;
        Ok(false)
    }

    fn handle_print_memory(&mut self, address : usize, len : usize, stream : &mut TcpStream) -> std::io::Result<bool> {
        let dump = self.host.create_memory_dump(address, address+len);
        send_response(ResponseData::Dump(dump), stream)?;
        Ok(false)
    }

    fn breakpoint_hit(&mut self, _last_ip: usize) -> bool {
        for &bp in &self.breakpoints {
            if bp == self.host.ip() {
                self.hit_breakpoint = bp;
                return true;
            }
            //if last_ip < bp && self.host.ip() > bp {
            //    self.hit_breakpoint = bp;
            //    return true; // ip not on an actual instruction
            //}
        }
        false
    }

    fn run(&mut self) {
        self.hit_breakpoint = 0;
        let mut last_ip = 0;
        while !self.breakpoint_hit(last_ip) && self.host.should_run() {
            last_ip = self.host.ip();
            self.host.step();
        }
    }
}
