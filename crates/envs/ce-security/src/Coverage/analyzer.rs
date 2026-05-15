use ce_core::Env;
use ce_security::{Input, SecurityEnv};
use std::fs;

fn main() {
    // Expect: --input <path>
    let mut args = std::env::args().skip(1);
    let flag = args.next();
    let path = args.next();

    if flag.as_deref() != Some("--input") || path.is_none() {
        eprintln!("Usage: analyzer --input <input.json>");
        std::process::exit(1);
    }

    let path = path.unwrap();
    let json = fs::read_to_string(&path).expect("failed to read input.json");
    let input: Input = serde_json::from_str(&json).expect("invalid input.json");

    let output = SecurityEnv::run(&input).expect("analysis failed");

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
