use std::env;
use std::fs;
use std::io::prelude::*;
use std::path;

fn parse(input: &str) -> Vec<u16> {
    let parts: Vec<&str> = input.split(",").collect();
    parts
        .iter()
        .map(|p| p.trim().parse::<u16>().unwrap())
        .collect()
}

fn get_outpath(inpath: &str) -> path::PathBuf {
    let inp = path::Path::new(inpath);
    let inp = inp.canonicalize().unwrap();
    let mut retr = path::PathBuf::from(inp);
    retr.set_extension("bin");
    retr
}

fn write(data: Vec<u8>, path: &path::Path) {
    let mut file = match fs::File::create(path) {
        Err(why) => panic!("Could not create file {} : {}", path.display(), why),
        Ok(file) => file,
    };
    match file.write_all(&data) {
        Err(why) => panic!("Failed to write file {}: {}", path.display(), why),
        Ok(_) => println!("done"),
    }
}

fn convert(data: Vec<u16>) -> Vec<u8> {
    let mut buff = Vec::new();
    for d in data.iter() {
        //let mut bytes = [0;2];
        let bytes = d.to_le_bytes();
        //LittleEndian::write_u16(&mut bytes, *d);
        for b in bytes.iter() {
            buff.push(*b);
        }
    }
    buff
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

    let contents = fs::read_to_string(file).expect("Failed to read input file");
    println!("contents {}", contents);

    let data = parse(&contents);
    println!("data {:?}", data);

    let outpath = get_outpath(file);
    println!("writing to file {:?}", outpath);

    let converted = convert(data);

    write(converted, &outpath);
}
