use std::env;
use std::process::ExitCode;

use stealth_bonding_host::{
    accept, listen, serve_accessory_pair, serve_accessory_reconnect, DEFAULT_ADDR,
};
use stealth_bonding_sim::Accessory;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let mut addr = DEFAULT_ADDR.to_string();
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--addr" => addr = args.next().expect("--addr needs a value"),
            _ => {
                eprintln!("sb-accessory [--addr {DEFAULT_ADDR}]");
                return ExitCode::FAILURE;
            }
        }
    }

    let listener = listen(&addr).expect("listen");
    println!("listening on {addr}");
    let mut accessory = Accessory::new();
    let mut stream = accept(&listener).expect("accept");
    match serve_accessory_pair(&mut stream, &mut accessory) {
        Ok(outcome) => println!("paired {}", outcome.scan_row),
        Err(error) => {
            eprintln!("pair failed: {error}");
            return ExitCode::FAILURE;
        }
    }

    let mut stream = accept(&listener).expect("accept reconnect");
    match serve_accessory_reconnect(&mut stream, &mut accessory) {
        Ok(credential_id) => {
            println!("reconnected {:02x?}", credential_id);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("reconnect failed: {error}");
            ExitCode::FAILURE
        }
    }
}
