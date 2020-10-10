use std::io::prelude::*;

use std::fs;
use std::path;

pub struct Program {
    pub data: Vec<u16>,
    pub path: path::PathBuf,
}

impl Program {
    pub fn new() -> Program {
        Program {
            data: Vec::new(),
            path: path::PathBuf::new(),
        }
    }
    pub fn parse_file(path: &path::Path) -> Program {
        let mut file = match fs::File::open(path) {
            Err(why) => panic!("Failed to open {} : {}", path.display(), why),
            Ok(file) => file,
        };
        let mut buffer = Vec::new();
        match file.read_to_end(&mut buffer) {
            Err(why) => panic!("Failed to read {} : {}", path.display(), why),
            Ok(_) => {}
        }

        let mut data: Vec<u16> = Vec::new();
        for pair in buffer.chunks(2) {
            let mut val: u16 = pair[0] as u16;
            val += (pair[1] as u16) << 8;
            data.push(val)
        }
        Program {
            data,
            path: path::PathBuf::from(path),
        }
    }
}

impl Clone for Program {
    fn clone(&self) -> Program {
        Program {
            data: self.data.clone(),
            path: path::PathBuf::from(&self.path),
        }
    }
}
