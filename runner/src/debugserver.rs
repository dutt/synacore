use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use crate::host;
use crate::program;

use messages::{Command, Message, ResponseData};

pub struct Debugserver {
    host : host::Host,
}


fn send(data : &[u8], stream : &mut TcpStream) -> std::io::Result<()> {
    stream.write(&data)?;
    stream.flush()
}

fn send_string(text : &str, stream : &mut TcpStream) -> std::io::Result<()> {
    send(text.as_bytes(), stream)?;
    Ok(())
}

fn send_response(data : ResponseData, stream : &mut TcpStream) -> std::io::Result<()> {
    let response = Message::Response(data);
    let jsondata = response.serialize();
    send(&jsondata, stream)?;
    Ok(())
}

fn _recv(stream : &mut TcpStream) -> std::io::Result<String> {
    let mut buffer = [0; 1024];
    let numbytes = stream.read(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer[..numbytes]).into_owned())
}

fn recv_cmd(stream : &mut TcpStream) -> std::io::Result<Command> {
    let mut buffer = [0; 1024];
    let numbytes = stream.read(&mut buffer)?;
    if numbytes == 0 {
        return Ok(Command::None)
    }
    if let Message::Request(cmd) = Message::deserialize(&buffer[..numbytes]) {
        Ok(cmd)
    } else {
        panic!("command not a request");
    }
}

impl Debugserver {
    pub fn start(program : program::Program) {
        let mut ds = Debugserver {
            host : host::Host::from(program),
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

    fn handle_cmd(&mut self, cmd : Command, stream : &mut TcpStream) -> std::io::Result<bool> {
        match cmd {
            Command::None => Ok(false),
            Command::Quit => self.handle_quit(stream),
            Command::Run => self.handle_run(stream),
            Command::Step => self.handle_step(stream),
            _ => panic!("unknown command {:?}", cmd),
        }
    }

    fn handle_step(&mut self, stream : &mut TcpStream) -> std::io::Result<bool> {
        self.host.step();
        let state = self.host.create_state();
        send_response(ResponseData::State(state), stream)?;
        Ok(false)
    }

    fn handle_run(&mut self, stream : &mut TcpStream) -> std::io::Result<bool> {
        send_string("Running program...", stream)?;
        self.host.run();
        Ok(false)
    }

    fn handle_quit(&mut self, _stream : &mut TcpStream) -> std::io::Result<bool> {
        println!("Client disconnected");
        // client shuts down the stream
        Ok(true)
    }
}
