use jsode::prelude::*;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} file.json", args[0]);
	    std::process::exit(1);
    }

    let path = &args[1];
    let mut s = String::new();
    let mut f = std::fs::File::open(path).expect("Unable to open file");

    match std::io::Read::read_to_string(&mut f, &mut s) {
        Err(_) => std::process::exit(1),
        Ok(_) => println!("{}", s),
    }

    match JsonParser::new(&s).parse() {
        Ok(_) => std::process::exit(0),
        _ => std::process::exit(1),
    }
}