use stealth_bonding_sim::{
    pair, reconnect, Accessory, Advertising, BondPasskeyId, BondedLink, DelegatedError, Host,
    PairingError, PairingMode, PairingOffer, ReconnectError, ReconnectMode, StealthBondingParams,
    StealthPolicy, PARAMS_OPCODE, PARAMS_RESPONSE_OPCODE, T_SB_ENROLL_MS,
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
    assert_eq!(accessory.advertising(), &Advertising::DirectedKnock);
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
    let mut second_host = Host::with_identity([0x33, 0, 0, 0, 0, 0x03]);

    enroll_default(&mut accessory, &mut laptop);
    accessory.open_enrollment().expect("second enrollment");
    pair(&mut second_host, &mut accessory, PairingOffer::default()).expect("second pair");

    assert_eq!(accessory.bond_ids().len(), 2);
    reconnect(&laptop, &mut accessory).expect("laptop reconnect");
    reconnect(&second_host, &mut accessory).expect("second host reconnect");
    assert!(!accessory.appears_on_public_scan_list());
}

#[test]
fn commitment_failure_stores_no_bond_passkey() {
    let mut accessory = Accessory::that_cannot_leave_public_advertising();
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
    assert!(accessory.bond_ids().is_empty());
    assert_eq!(accessory.ordinary_bond_count(), 1);
    assert!(accessory.appears_on_public_scan_list());
    let link = reconnect(&host, &mut accessory).expect("ordinary reconnect");
    assert!(link.bond_passkey_id.is_none());
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
    let mut second_host = Host::with_identity([0x33, 0, 0, 0, 0, 0x03]);
    enroll_default(&mut accessory, &mut laptop);

    let link = reconnect(&laptop, &mut accessory).expect("link");
    accessory
        .open_delegated_enrollment(Some(&link))
        .expect("delegated");
    pair(&mut second_host, &mut accessory, PairingOffer::default()).expect("second host pair");

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
fn both_policy_leaves_neither_on_public_scan_list() {
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

#[test]
fn enrollment_window_expires_to_unprovisioned_when_bond_table_empty() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");

    accessory.advance(T_SB_ENROLL_MS);

    assert_eq!(
        pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("expired"),
        PairingError::NotEnrolling
    );
    assert!(accessory.appears_on_public_scan_list());
    assert!(accessory.bond_ids().is_empty());
}

#[test]
fn enrollment_window_expires_to_directed_knock_when_bond_table_has_entries() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    enroll_default(&mut accessory, &mut host);
    accessory.open_enrollment().expect("second window");

    accessory.advance(T_SB_ENROLL_MS);

    assert!(!accessory.appears_on_public_scan_list());
    assert_eq!(accessory.advertising(), &Advertising::DirectedKnock);
    reconnect(&host, &mut accessory).expect("still bonded");
}

#[test]
fn delegated_enrollment_from_connected_unbonded_peer_is_rejected() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    enroll_default(&mut accessory, &mut host);

    let fake = BondedLink {
        bond_passkey_id: Some(BondPasskeyId([0x09; 16])),
    };
    let error = accessory
        .open_delegated_enrollment(Some(&fake))
        .expect_err("unbonded link");
    assert_eq!(error, DelegatedError::NoBond);
}

#[test]
fn delegated_confirm_is_required_before_a_later_bond_passkey() {
    let mut accessory = Accessory::new();
    let mut laptop = Host::new();
    let mut second_host = Host::with_identity([0x33, 0, 0, 0, 0, 0x03]);
    accessory.open_enrollment().expect("enrollment");
    pair(
        &mut laptop,
        &mut accessory,
        PairingOffer {
            require_delegated_confirm: true,
            ..PairingOffer::default()
        },
    )
    .expect("first pair");

    let link = reconnect(&laptop, &mut accessory).expect("link");
    accessory
        .open_delegated_enrollment(Some(&link))
        .expect("delegated");

    assert_eq!(
        pair(&mut second_host, &mut accessory, PairingOffer::default()).expect_err("no confirm"),
        PairingError::DelegatedConfirmRequired
    );
    assert_eq!(accessory.bond_ids().len(), 1);

    accessory
        .confirm_delegated_enrollment(&link)
        .expect("confirm");
    pair(&mut second_host, &mut accessory, PairingOffer::default()).expect("after confirm");
    assert_eq!(accessory.bond_ids().len(), 2);
}

#[test]
fn registration_fails_without_attestation() {
    let mut accessory = Accessory::without_factory_attestation();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("attest");
    assert_eq!(error, PairingError::AttestationFailed);
    assert!(accessory.bond_ids().is_empty());
}

#[test]
fn registration_fails_when_host_cannot_verify_attestation() {
    let mut accessory = Accessory::new();
    let mut host = Host::that_cannot_verify_attestation();
    accessory.open_enrollment().expect("enrollment");

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("verify");
    assert_eq!(error, PairingError::AttestationFailed);
    assert!(accessory.bond_ids().is_empty());
}

#[test]
fn registration_fails_when_attestation_omits_sc_transcript() {
    let mut accessory = Accessory::that_omits_sc_from_attestation();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("sc");
    assert_eq!(error, PairingError::AttestationFailed);
    assert!(accessory.bond_ids().is_empty());
}

#[test]
fn registration_fails_with_forged_unit_cert() {
    let mut accessory = Accessory::with_forged_unit_cert();
    let mut host = Host::new();
    accessory.open_enrollment().expect("enrollment");

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("cert");
    assert_eq!(error, PairingError::AttestationFailed);
    assert!(accessory.bond_ids().is_empty());
}

#[test]
fn enrollment_scan_rows_differ_when_local_names_match() {
    let first = Accessory::new();
    let second = Accessory::with_identity([0x22, 0, 0, 0, 0, 0x03], 8);

    assert_eq!(first.local_name(), second.local_name());
    assert_ne!(first.enrollment_scan_row(), second.enrollment_scan_row());
}

#[test]
fn closeness_threshold_rejects_a_far_accessory() {
    let mut accessory = Accessory::that_is_far_away();
    let mut host = Host::that_requires_closeness(-50);
    accessory.open_enrollment().expect("enrollment");

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("far");
    assert_eq!(error, PairingError::ClosenessFailed);
    assert!(accessory.bond_ids().is_empty());
}

#[test]
fn acoustic_check_rejects_a_silent_audio_accessory() {
    let mut accessory = Accessory::without_acoustic();
    let mut host = Host::that_requires_acoustic();
    accessory.open_enrollment().expect("enrollment");

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("silent");
    assert_eq!(error, PairingError::AcousticFailed);
    assert!(accessory.bond_ids().is_empty());
}

#[test]
fn registration_fails_on_replayed_attestation_nonce() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    enroll_default(&mut accessory, &mut host);
    accessory.revoke(accessory.bond_ids()[0]);
    accessory.open_enrollment().expect("again");
    host.replay_next_attestation_nonce();

    let error = pair(&mut host, &mut accessory, PairingOffer::default()).expect_err("replay");
    assert_eq!(error, PairingError::AttestationReplay);
}

#[test]
fn assertion_reconnect_succeeds() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    enroll_default(&mut accessory, &mut host);

    let link = reconnect(&host, &mut accessory).expect("assert");
    assert!(link.bond_passkey_id.is_some());
}

#[test]
fn replayed_assertion_challenge_fails() {
    let mut accessory = Accessory::new();
    let mut host = Host::new();
    enroll_default(&mut accessory, &mut host);
    reconnect(&host, &mut accessory).expect("first");
    accessory.replay_next_assertion_challenge();

    let error = reconnect(&host, &mut accessory).expect_err("replay");
    assert_eq!(error, ReconnectError::AssertionReplay);
    assert!(!accessory.appears_on_public_scan_list());
}

#[test]
fn synced_host_same_cred_skips_enrollment() {
    let mut accessory = Accessory::new();
    let mut laptop = Host::new();
    enroll_default(&mut accessory, &mut laptop);

    let phone = Host::sharing_vault_with(&laptop, [0x44, 0, 0, 0, 0, 0x04]);
    reconnect(&phone, &mut accessory).expect("synced vault");
    assert_eq!(accessory.bond_ids().len(), 1);
}

#[test]
fn stranger_host_needs_enrollment() {
    let mut accessory = Accessory::new();
    let mut laptop = Host::new();
    let stranger = Host::with_identity([0x99, 0, 0, 0, 0, 0x09]);
    enroll_default(&mut accessory, &mut laptop);

    let error = reconnect(&stranger, &mut accessory).expect_err("stranger");
    assert_eq!(error, ReconnectError::NoBond);
}

#[test]
fn delegated_confirm_timeout_discards_the_new_pair() {
    let mut accessory = Accessory::new();
    let mut laptop = Host::new();
    let mut second_host = Host::with_identity([0x33, 0, 0, 0, 0, 0x03]);
    accessory.open_enrollment().expect("enrollment");
    pair(
        &mut laptop,
        &mut accessory,
        PairingOffer {
            require_delegated_confirm: true,
            ..PairingOffer::default()
        },
    )
    .expect("first pair");

    let link = reconnect(&laptop, &mut accessory).expect("link");
    accessory
        .open_delegated_enrollment(Some(&link))
        .expect("delegated");
    accessory.advance(T_SB_ENROLL_MS);

    assert_eq!(
        pair(&mut second_host, &mut accessory, PairingOffer::default()).expect_err("expired"),
        PairingError::NotEnrolling
    );
    assert_eq!(accessory.bond_ids().len(), 1);
    assert!(!accessory.appears_on_public_scan_list());
}
