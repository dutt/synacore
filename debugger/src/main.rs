use std::io;
use std::io::prelude::*;

use std::net::{TcpStream, Shutdown};

use messages::Command;

fn get_line() -> std::io::Result<String> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(String::from(line.trim()))
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

fn handle_quit(stream : &mut TcpStream) -> std::io::Result<bool> {
    send(Command::Quit.serialize(), stream)?;
    stream.shutdown(Shutdown::Both)?;
    Ok(true)
}

fn handle_run(stream : &mut TcpStream) -> std::io::Result<bool> {
    send(Command::Run.serialize(), stream)?;
    Ok(false)
}

fn handle(cmd : Command, stream : &mut TcpStream) -> std::io::Result<bool> {
    match cmd {
        Command::Quit => handle_quit(stream),
        Command::Run => handle_run(stream),
        _ => panic!("unknown command {:?}", cmd),
    }
}

fn run(stream : &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0;1024];
    stream.read(&mut buffer)?;
    let greeting = String::from_utf8_lossy(&buffer[..]);
    println!("{}", greeting);
    loop {
        let line = get_line()?;
        let cmd = Command::parse(&line);
        if let Ok(do_quit) = handle(cmd, stream) {
            if do_quit {
                break;
            }
        }
    }

    Ok(())
}

fn main() {
    match TcpStream::connect("localhost:6565") {
        Ok(mut stream) => {
            println!("connected");
            match run(&mut stream) {
                Ok(_) => {},
                Err(e) => panic!("Error durring network {}", e),
            }
        }
        Err(e) => {
            panic!("failed to connect: {}", e);
        }
    }
}
