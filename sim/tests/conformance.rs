use stealth_bonding_sim::{
    pair, reconnect, Accessory, Advertising, DelegatedError, Host, PairingError, PairingMode,
    PairingOffer, ReconnectError, ReconnectMode, StealthBondingParams, StealthPolicy,
    PARAMS_OPCODE, PARAMS_RESPONSE_OPCODE,
};

fn enroll_default(accessory: &mut Accessory, host: &mut Host) {
    accessory.open_enrollment().expect("enrollment");
    pair(host, accessory, PairingOffer::default()).expect("pair");
}

#[test]
fn enrollment_then_peripheral_only_commitment_leaves_no_public_advertising() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");

    let outcome = pair(&mut host, &mut accessory, PairingOffer::default()).expect("pair");

    assert_eq!(outcome.mode, PairingMode::StealthBonding);
    assert!(!accessory.appears_on_public_scan_list());
    assert!(host.appears_on_public_scan_list());
    assert_eq!(outcome.stealth_policy, StealthPolicy::PeripheralOnly);
    assert_eq!(outcome.reconnect_mode, ReconnectMode::DirectedKnock);
}

#[test]
fn nameless_undirected_advertising_is_public() {
    let advertising = Advertising::Undirected { local_name: None };
    assert!(advertising.is_public());
}

#[test]
fn stealth_bonding_params_match_spec_octets() {
    let params = StealthBondingParams {
        policy: StealthPolicy::PeripheralOnly,
        reconnect: ReconnectMode::DirectedKnock,
        require_delegated_confirm: false,
        allow_peer_policy_change: false,
        max_bonds: 8,
    };

    assert_eq!(
        params.encode(PARAMS_OPCODE),
        [0xF0, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00]
    );
    assert_eq!(
        params.encode(PARAMS_RESPONSE_OPCODE),
        [0xF1, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00]
    );
}

#[test]
fn pairing_exchanges_spec_params_pdus() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");

    let outcome = pair(&mut host, &mut accessory, PairingOffer::default()).expect("pair");

    assert_eq!(
        outcome.params_pdu.expect("params"),
        [0xF0, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00]
    );
    assert_eq!(
        outcome.params_response_pdu.expect("response"),
        [0xF1, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00]
    );
}

#[test]
fn second_bond_passkey_lets_both_hosts_reconnect() {
    let mut accessory = Accessory::new();
    let mut laptop = Host::new();
    let mut phone = Host::with_identity([0x33, 0, 0, 0, 0, 0x03]);

    enroll_default(&mut accessory, &mut laptop);
    accessory.open_enrollment().expect("second enrollment");
    pair(&mut phone, &mut accessory, PairingOffer::default()).expect("second pair");

    assert_eq!(accessory.bond_ids().len(), 2);
    reconnect(&laptop, &mut accessory).expect("laptop reconnect");
    reconnect(&phone, &mut accessory).expect("phone reconnect");
    assert!(!accessory.appears_on_public_scan_list());
}

#[test]
fn commitment_failure_stores_no_bond_passkey() {
    let mut accessory = Accessory::that_cannot_hide();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("commit");

    assert_eq!(error, PairingError::CommitmentFailed);
    assert!(accessory.bond_ids().is_empty());
    assert!(host.bond_ids().is_empty());
}

#[test]
fn unsupported_peer_falls_back_and_does_not_claim_stealth_bonding() {
    let mut accessory = Accessory::new();
    let mut host = Host::without_stealth_bonding();
    accessory.open_enrollment().expect("enrollment");

    let outcome = pair(&mut host, &mut accessory, PairingOffer::default()).expect("fallback");

    assert_eq!(outcome.mode, PairingMode::Fallback);
    assert!(outcome.bond_passkey_id.is_none());
    assert!(outcome.params_pdu.is_none());
}

#[test]
fn legacy_pairing_is_forbidden_in_this_mode() {
    let mut accessory = Accessory::new();
    let mut host = Host::that_uses_legacy_pairing();
    accessory.open_enrollment().expect("enrollment");

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("legacy");
    assert_eq!(error, PairingError::LegacyPairingForbidden);
}

#[test]
fn delegated_enrollment_from_unbonded_peer_is_rejected() {
    let mut accessory = Accessory::new();
    let error = accessory
        .open_delegated_enrollment(None)
        .expect_err("unbonded");
    assert_eq!(error, DelegatedError::UnbondedPeer);
}

#[test]
fn delegated_enrollment_from_bonded_host_opens_window() {
    let mut accessory = Accessory::new();
    let mut laptop = Host::new();
    let mut phone = Host::with_identity([0x33, 0, 0, 0, 0, 0x03]);
    enroll_default(&mut accessory, &mut laptop);

    let link = reconnect(&laptop, &mut accessory).expect("link");
    accessory
        .open_delegated_enrollment(Some(&link))
        .expect("delegated");
    pair(&mut phone, &mut accessory, PairingOffer::default()).expect("phone pair");

    assert_eq!(accessory.bond_ids().len(), 2);
}

#[test]
fn full_bond_table_refuses_a_new_pair() {
    let mut accessory = Accessory::with_capacity(2);
    let mut first = Host::with_identity([0x11, 0, 0, 0, 0, 0x01]);
    let mut second = Host::with_identity([0x12, 0, 0, 0, 0, 0x02]);
    enroll_default(&mut accessory, &mut first);
    accessory.open_enrollment().expect("second");
    pair(&mut second, &mut accessory, PairingOffer::default()).expect("second pair");

    let error = accessory.open_enrollment().expect_err("full");
    assert_eq!(error, PairingError::TableFull);
    assert_eq!(accessory.bond_ids().len(), 2);
}

#[test]
fn factory_reset_returns_to_unprovisioned_public_advertising() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    enroll_default(&mut accessory, &mut host);

    accessory.factory_reset();

    assert!(accessory.bond_ids().is_empty());
    assert!(accessory.appears_on_public_scan_list());
}

#[test]
fn encryption_failure_on_reconnect_does_not_open_public_advertising() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    enroll_default(&mut accessory, &mut host);
    host.break_ltk();

    let error = reconnect(&host, &mut accessory).expect_err("ltk");
    assert_eq!(error, ReconnectError::EncryptionFailed);
    assert!(!accessory.appears_on_public_scan_list());
}

#[test]
fn silent_until_wake_is_not_connectable_until_wake() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");
    pair(
        &mut host,
        &mut accessory,
        PairingOffer {
            reconnect: ReconnectMode::SilentUntilWake,
            ..PairingOffer::default()
        },
    )
    .expect("pair");

    assert!(!accessory.appears_on_public_scan_list());
    assert_eq!(
        reconnect(&host, &mut accessory).expect_err("silent"),
        ReconnectError::NotConnectable
    );
    accessory.wake();
    reconnect(&host, &mut accessory).expect("after wake");
}

#[test]
fn both_policy_hides_host_and_accessory() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");
    pair(
        &mut host,
        &mut accessory,
        PairingOffer {
            policy: StealthPolicy::Both,
            ..PairingOffer::default()
        },
    )
    .expect("pair");

    assert!(!accessory.appears_on_public_scan_list());
    assert!(!host.appears_on_public_scan_list());
}
