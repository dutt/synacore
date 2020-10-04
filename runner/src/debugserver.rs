use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use crate::host;
use crate::program;

use messages::Command;

pub struct Debugserver {
    program : program::Program,
    host : host::Host,
}


fn send(data : Vec<u8>, stream : &mut TcpStream) -> std::io::Result<()> {
    stream.write(&data)?;
    stream.flush()
}

fn recv(stream : &mut TcpStream) -> std::io::Result<String> {
    let mut buffer = [0; 1024];
    let numbytes = stream.read(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer[..numbytes]).into_owned())
}

fn recv_cmd(stream : &mut TcpStream) -> std::io::Result<Command> {
    let mut buffer = [0; 1024];
    let numbytes = stream.read(&mut buffer)?;
    Ok(Command::deserialize(&buffer[..numbytes]))
}

impl Debugserver {
    pub fn start(program : program::Program) {
        let mut ds = Debugserver {
            host : host::Host::new(),
            program,
        };
        match ds.listen() {
            Ok(_) => {},
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

    fn handle(&mut self, stream : &mut TcpStream) -> std::io::Result<()> {
        println!("New connection {}", stream.peer_addr().unwrap());
        let mut greeting = String::from("Running ");
        greeting += self.program.path.to_str().unwrap();
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

    fn handle_cmd(&mut self, cmd : Command, stream : &mut TcpStream) -> std::io::Result<bool> {
        match cmd {
            Command::Quit => self.handle_quit(stream),
            _ => panic!("unknown command {:?}", cmd),
        }
    }

    fn handle_quit(&mut self, _stream : &mut TcpStream) -> std::io::Result<bool> {
        println!("Client disconnected");
        // client shuts down the stream
        Ok(true)
    }
}
