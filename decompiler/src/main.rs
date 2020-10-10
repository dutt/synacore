use std::env;

use std::fs;
use std::io::prelude::*;
use std::path;

use code::decompile;

fn get_outpath(inpath: &str) -> path::PathBuf {
    let inp = path::Path::new(inpath);
    let inp = inp.canonicalize().unwrap();
    let mut retr = path::PathBuf::from(inp);
    retr.set_extension("decompiled");
    retr
}

fn write(data: String, path: &path::Path) {
    let mut file = match fs::File::create(path) {
        Err(why) => panic!("Could not create file {} : {}", path.display(), why),
        Ok(file) => file,
    };
    match file.write_all(data.as_bytes()) {
        Err(why) => panic!("Failed to write file {}: {}", path.display(), why),
        Ok(_) => println!("done"),
    }
}

pub fn decompile_file(file: &str) {
    let data = decompile::parse_file(path::PathBuf::from(file.clone()));
    let clean_data = decompile::cleanup(data);
    let text = decompile::serialize(clean_data);

    let outpath = get_outpath(file);
    println!("writing to file {:?}", outpath);

    write(text, &outpath);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args {:?}", args);
    if args.len() < 2 {
        println!("No input file specified");
        return;
    }

    let file = &args[1];
    println!("reading file {:}", file);

    decompile_file(file);
}
