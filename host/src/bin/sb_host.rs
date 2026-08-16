use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use stealth_bonding_host::{connect, pair_as_host, reconnect_as_host, FileVault, DEFAULT_ADDR};
use stealth_bonding_sim::PairingOffer;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut addr = DEFAULT_ADDR.to_string();
    let mut vault_path = PathBuf::from("sb-vault.txt");
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--addr" => addr = args.next().expect("--addr needs a value"),
            "--vault" => vault_path = PathBuf::from(args.next().expect("--vault needs a value")),
            _ => return usage(),
        }
    }

    let result = match command.as_str() {
        "pair" => {
            let mut vault = FileVault::create_or_load(&vault_path).expect("vault");
            let mut stream = connect(&addr).expect("connect");
            pair_as_host(&mut stream, &mut vault, PairingOffer::default()).map(|outcome| {
                println!(
                    "paired {} as {}",
                    hex(&outcome.credential_id),
                    outcome.scan_row
                );
            })
        }
        "reconnect" => {
            let vault = FileVault::load(&vault_path).expect("vault");
            let mut stream = connect(&addr).expect("connect");
            reconnect_as_host(&mut stream, &vault).map(|credential_id| {
                println!("reconnected {}", hex(&credential_id));
            })
        }
        _ => return usage(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("sb-host pair|reconnect [--addr {DEFAULT_ADDR}] [--vault sb-vault.txt]");
    ExitCode::FAILURE
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
