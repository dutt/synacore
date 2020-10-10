use std::env;
use std::io::Write;
use std::path;

#[macro_use]
extern crate log;

use env_logger::{Builder, Target};

mod debugserver;
mod host;

use code::program;

struct Config {
    filename: path::PathBuf,
}

impl Config {
    fn new(args: &[String]) -> Config {
        if args.len() < 2 {
            panic!("No binary supplied");
        }
        let filename = args[1].clone();
        Config {
            filename: path::PathBuf::from(filename),
        }
    }
}

fn main() {
    // init logging
    Builder::from_default_env()
        .target(Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);
    info!("Running file {}", config.filename.display());

    let program = program::Program::parse_file(&config.filename);

    debugserver::Debugserver::start(program);
    //let mut host = host::Host::new();
    //host.run(program);
}
