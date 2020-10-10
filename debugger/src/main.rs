use std::io;
use std::io::prelude::*;

use std::net::{Shutdown, TcpStream};

use code::decompile;
use messages::{Command, Message, ResponseData, VmState};

fn get_line() -> std::io::Result<String> {
    print!("> ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(String::from(line.trim()))
}

fn send(cmd: Command, stream: &mut TcpStream) -> std::io::Result<()> {
    let req = Message::Request(cmd);
    let data = req.serialize();
    let len: u64 = data.len() as u64;
    stream.write(&len.to_le_bytes())?;
    stream.write(&data)?;
    stream.flush()
}

fn recv_response(stream: &mut TcpStream) -> std::io::Result<Vec<ResponseData>> {
    // length
    let mut buffer = [0; 8];
    let numbytes = stream.take(8).read(&mut buffer)?;
    if numbytes == 0 {
        return Ok(vec![ResponseData::Empty]);
    }
    assert_eq!(numbytes, 8);
    let len = u64::from_le_bytes(buffer);

    // data
    let mut data = Vec::new();
    stream.take(len).read_to_end(&mut data)?;
    let response = Message::deserialize(&data);
    match response {
        Message::Response(data) => Ok(vec![data]),
        Message::Responses(datas) => Ok(datas),
        _ => panic!("Message not a response: {:?}", response),
    }
}

fn print_state(state: VmState) {
    let opdata = decompile::parse_data_offset(Vec::from(state.here), state.ip);
    let text = decompile::serialize(opdata);
    println!("{}/{} : {}", state.ip, state.count, text);
}

fn handle_response(data: ResponseData) {
    match data {
        ResponseData::Text(content) => println!("{}", content),
        ResponseData::State(state) => print_state(state),
        _ => panic!("no response {:?}", data),
    }
}
fn handle_quit(stream: &mut TcpStream) -> std::io::Result<bool> {
    send(Command::Quit, stream)?;
    stream.shutdown(Shutdown::Both)?;
    Ok(true)
}

fn handle_default(
    cmd: Command,
    stream: &mut TcpStream,
    response_count: usize,
) -> std::io::Result<bool> {
    send(cmd, stream)?;
    for _ in 0..response_count {
        for r in recv_response(stream)? {
            handle_response(r);
        }
    }
    Ok(false)
}

fn handle(cmd: Command, stream: &mut TcpStream) -> std::io::Result<bool> {
    match cmd {
        Command::Quit => handle_quit(stream),
        Command::Run => handle_default(cmd, stream, 2),
        Command::Step => handle_default(cmd, stream, 1),
        Command::Continue => handle_default(cmd, stream, 2),
        Command::AddBreakpoint(_) => handle_default(cmd, stream, 1),
        Command::RemoveBreakpoint(_) => handle_default(cmd, stream, 1),
        _ => panic!("unknown command {:?}", cmd),
    }
}

fn run(stream: &mut TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
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
                Ok(_) => {}
                Err(e) => panic!("Error durring network {}", e),
            }
        }
        Err(e) => {
            panic!("failed to connect: {}", e);
        }
    }
}
