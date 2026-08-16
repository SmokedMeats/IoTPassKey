use std::thread;

use stealth_bonding_host::{
    accept, connect, listen, pair_as_host, reconnect_as_host, serve_accessory_pair,
    serve_accessory_reconnect, FileVault, HostError,
};
use stealth_bonding_sim::{
    generate_credential_id, generate_secret, public_key, Accessory, PairingError, PairingOffer,
    ReconnectError,
};

fn temp_vault() -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "sb-vault-{}.txt",
        generate_credential_id()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    ));
    path
}

#[test]
fn vault_round_trip_keeps_the_same_public_key() {
    let path = temp_vault();
    let created = FileVault::create(&path).expect("create");
    let loaded = FileVault::load(&path).expect("load");
    assert_eq!(created.public_key(), loaded.public_key());
    assert_eq!(created.credential_id, loaded.credential_id);
    let _ = std::fs::remove_file(path);
}

#[test]
fn file_vault_host_pairs_and_reconnects_over_tcp() {
    let listener = listen("127.0.0.1:0").expect("listen");
    let addr = listener.local_addr().expect("addr");
    let accessory = thread::spawn(move || {
        let mut accessory = Accessory::new();
        let mut stream = accept(&listener).expect("pair accept");
        serve_accessory_pair(&mut stream, &mut accessory).expect("pair serve");
        let mut stream = accept(&listener).expect("reconnect accept");
        serve_accessory_reconnect(&mut stream, &mut accessory).expect("reconnect serve")
    });

    let path = temp_vault();
    let mut vault = FileVault::create(&path).expect("vault");
    let mut stream = connect(addr).expect("pair connect");
    let paired = pair_as_host(&mut stream, &mut vault, PairingOffer::default()).expect("pair");
    assert!(paired.scan_row.contains('·'));

    let mut stream = connect(addr).expect("reconnect connect");
    let credential_id = reconnect_as_host(&mut stream, &vault).expect("reconnect");
    assert_eq!(credential_id, paired.credential_id);
    accessory.join().expect("accessory");
    let _ = std::fs::remove_file(path);
}

#[test]
fn stranger_vault_cannot_reconnect() {
    let listener = listen("127.0.0.1:0").expect("listen");
    let addr = listener.local_addr().expect("addr");
    let accessory = thread::spawn(move || {
        let mut accessory = Accessory::new();
        let mut stream = accept(&listener).expect("pair accept");
        serve_accessory_pair(&mut stream, &mut accessory).expect("pair serve");
        let mut stream = accept(&listener).expect("reconnect accept");
        serve_accessory_reconnect(&mut stream, &mut accessory)
    });

    let owner_path = temp_vault();
    let stranger_path = temp_vault();
    let mut owner = FileVault::create(&owner_path).expect("owner");
    let stranger = FileVault::create(&stranger_path).expect("stranger");
    let mut stream = connect(addr).expect("pair connect");
    pair_as_host(&mut stream, &mut owner, PairingOffer::default()).expect("pair");

    let mut stream = connect(addr).expect("reconnect connect");
    let error = reconnect_as_host(&mut stream, &stranger).expect_err("stranger");
    assert!(matches!(
        error,
        HostError::Reconnect(ReconnectError::NoBond)
    ));
    accessory.join().expect("accessory").expect_err("no bond");
    let _ = std::fs::remove_file(owner_path);
    let _ = std::fs::remove_file(stranger_path);
}

#[test]
fn host_rejects_a_root_it_does_not_trust() {
    let listener = listen("127.0.0.1:0").expect("listen");
    let addr = listener.local_addr().expect("addr");
    let accessory = thread::spawn(move || {
        let mut accessory = Accessory::new();
        let mut stream = accept(&listener).expect("accept");
        serve_accessory_pair(&mut stream, &mut accessory)
    });

    let path = temp_vault();
    let mut vault = FileVault::create(&path).expect("vault");
    vault.manufacturer_root = public_key(&generate_secret());
    let error = {
        let mut stream = connect(addr).expect("connect");
        pair_as_host(&mut stream, &mut vault, PairingOffer::default()).expect_err("root")
    };
    assert!(matches!(
        error,
        HostError::Pairing(PairingError::AttestationFailed)
    ));
    let _ = accessory.join();
    let _ = std::fs::remove_file(path);
}
