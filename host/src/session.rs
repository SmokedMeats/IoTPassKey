use std::io;
use std::net::TcpStream;

use stealth_bonding_sim::{
    assertion_message, attestation_message, fixture_sc_public, generate_nonce, sign,
    unit_cert_message, verify, Accessory, PairingError, PairingOffer, ReconnectError,
    StealthBondingAssertion, StealthBondingAttestation, StealthBondingAttestationRequest,
    StealthBondingChallenge, StealthBondingParams, StealthBondingRegister, PARAMS_OPCODE,
};

use crate::vault::FileVault;
use crate::wire::{
    read_frame, write_frame, LinkHello, LinkStatus, STATUS_ASSERTION_FAILED,
    STATUS_COMMITMENT_FAILED, STATUS_NO_BOND, STATUS_OK,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingResult {
    pub credential_id: [u8; 16],
    pub accessory_identity: [u8; 6],
    pub scan_row: String,
}

#[derive(Debug)]
pub enum HostError {
    Io(io::Error),
    Pairing(PairingError),
    Reconnect(ReconnectError),
    Protocol(&'static str),
}

impl From<io::Error> for HostError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Pairing(error) => write!(f, "{error:?}"),
            Self::Reconnect(error) => write!(f, "{error:?}"),
            Self::Protocol(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for HostError {}

pub fn pair_as_host(
    stream: &mut TcpStream,
    vault: &mut FileVault,
    offer: PairingOffer,
) -> Result<PairingResult, HostError> {
    write_frame(
        stream,
        &LinkHello {
            identity: vault.identity,
        }
        .encode(),
    )?;
    let peer = LinkHello::decode(&read_frame(stream)?)?;

    let params = StealthBondingParams {
        policy: offer.policy,
        reconnect: offer.reconnect,
        require_delegated_confirm: offer.require_delegated_confirm,
        allow_peer_policy_change: offer.allow_peer_policy_change,
        max_bonds: 8,
    };
    write_frame(stream, &params.encode(PARAMS_OPCODE))?;
    let response = StealthBondingParams::decode(&read_frame(stream)?)
        .map_err(|_| HostError::Pairing(PairingError::ParamsRejected))?;
    if response.1.policy != offer.policy || response.1.reconnect != offer.reconnect {
        return Err(HostError::Pairing(PairingError::PolicyRejected));
    }

    let nonce = generate_nonce();
    if !vault.remember_nonce(nonce).map_err(HostError::from)? {
        return Err(HostError::Pairing(PairingError::AttestationReplay));
    }
    write_frame(stream, &StealthBondingAttestationRequest { nonce }.encode())?;

    let attestation_frame = read_frame(stream)?;
    if let Ok(status) = LinkStatus::decode(&attestation_frame) {
        return Err(status_to_pairing(status.code));
    }
    let attestation = StealthBondingAttestation::decode(&attestation_frame)
        .map_err(|_| HostError::Pairing(PairingError::AttestationFailed))?;
    if attestation.nonce != nonce {
        return Err(HostError::Pairing(PairingError::AttestationFailed));
    }
    if !verify(
        &vault.manufacturer_root,
        &unit_cert_message(&attestation.unit_public, &attestation.unit_serial),
        &attestation.unit_cert,
    ) {
        return Err(HostError::Pairing(PairingError::AttestationFailed));
    }
    let transcript = attestation_message(
        vault.identity,
        peer.identity,
        offer.policy.code(),
        offer.reconnect.code(),
        &attestation.nonce,
        &fixture_sc_public(vault.identity),
        &fixture_sc_public(peer.identity),
    );
    if !verify(
        &attestation.unit_public,
        &transcript,
        &attestation.attestation,
    ) {
        return Err(HostError::Pairing(PairingError::AttestationFailed));
    }

    write_frame(
        stream,
        &StealthBondingRegister {
            credential_id: vault.credential_id,
            public_key: vault.public_key(),
        }
        .encode(),
    )?;
    let status = LinkStatus::decode(&read_frame(stream)?)?;
    match status.code {
        STATUS_OK => Ok(PairingResult {
            credential_id: vault.credential_id,
            accessory_identity: peer.identity,
            scan_row: format!(
                "Accessory · {:02X}{:02X}",
                attestation.unit_serial[6], attestation.unit_serial[7]
            ),
        }),
        STATUS_COMMITMENT_FAILED => Err(HostError::Pairing(PairingError::CommitmentFailed)),
        _ => Err(HostError::Protocol("unexpected pairing status")),
    }
}

pub fn reconnect_as_host(stream: &mut TcpStream, vault: &FileVault) -> Result<[u8; 16], HostError> {
    write_frame(
        stream,
        &LinkHello {
            identity: vault.identity,
        }
        .encode(),
    )?;
    let peer = LinkHello::decode(&read_frame(stream)?)?;
    let challenge = StealthBondingChallenge::decode(&read_frame(stream)?)
        .map_err(|_| HostError::Reconnect(ReconnectError::AssertionFailed))?;
    let signature = sign(
        &vault.secret,
        &assertion_message(&challenge.challenge, peer.identity),
    );
    write_frame(
        stream,
        &StealthBondingAssertion {
            credential_id: vault.credential_id,
            signature,
        }
        .encode(),
    )?;
    let status = LinkStatus::decode(&read_frame(stream)?)?;
    match status.code {
        STATUS_OK => Ok(vault.credential_id),
        STATUS_NO_BOND => Err(HostError::Reconnect(ReconnectError::NoBond)),
        STATUS_ASSERTION_FAILED => Err(HostError::Reconnect(ReconnectError::AssertionFailed)),
        _ => Err(HostError::Protocol("unexpected reconnect status")),
    }
}

pub fn serve_accessory_pair(
    stream: &mut TcpStream,
    accessory: &mut Accessory,
) -> Result<PairingResult, HostError> {
    if let Err(error) = accessory.open_enrollment() {
        return Err(HostError::Pairing(error));
    }
    let peer = LinkHello::decode(&read_frame(stream)?)?;
    write_frame(
        stream,
        &LinkHello {
            identity: accessory.identity(),
        }
        .encode(),
    )?;

    let params = StealthBondingParams::decode(&read_frame(stream)?)
        .map_err(|_| HostError::Pairing(PairingError::ParamsRejected))?
        .1;
    let response = match accessory.respond_to_params(params) {
        Ok(response) => response,
        Err(error) => {
            write_status(stream, STATUS_NO_BOND)?;
            return Err(HostError::Pairing(error));
        }
    };
    write_frame(
        stream,
        &response.encode(stealth_bonding_sim::PARAMS_RESPONSE_OPCODE),
    )?;

    let request = StealthBondingAttestationRequest::decode(&read_frame(stream)?)
        .map_err(|_| HostError::Pairing(PairingError::AttestationFailed))?;
    let attestation = match accessory.attest_for_host(peer.identity, request.nonce) {
        Ok(attestation) => attestation,
        Err(error) => {
            write_status(stream, STATUS_ASSERTION_FAILED)?;
            return Err(HostError::Pairing(error));
        }
    };
    write_frame(stream, &attestation.encode())?;

    let register = StealthBondingRegister::decode(&read_frame(stream)?)
        .map_err(|_| HostError::Pairing(PairingError::ParamsRejected))?;
    match accessory.accept_register(register, peer.identity) {
        Ok(()) => {
            write_frame(stream, &LinkStatus::ok().encode())?;
            Ok(PairingResult {
                credential_id: register.credential_id,
                accessory_identity: accessory.identity(),
                scan_row: accessory.enrollment_scan_row(),
            })
        }
        Err(PairingError::CommitmentFailed) => {
            write_frame(
                stream,
                &LinkStatus {
                    code: STATUS_COMMITMENT_FAILED,
                }
                .encode(),
            )?;
            Err(HostError::Pairing(PairingError::CommitmentFailed))
        }
        Err(error) => Err(HostError::Pairing(error)),
    }
}

pub fn serve_accessory_reconnect(
    stream: &mut TcpStream,
    accessory: &mut Accessory,
) -> Result<[u8; 16], HostError> {
    let _peer = LinkHello::decode(&read_frame(stream)?)?;
    write_frame(
        stream,
        &LinkHello {
            identity: accessory.identity(),
        }
        .encode(),
    )?;
    let challenge = match accessory.issue_challenge() {
        Ok(challenge) => challenge,
        Err(error) => {
            write_frame(
                stream,
                &LinkStatus {
                    code: status_for_reconnect(&error),
                }
                .encode(),
            )?;
            return Err(HostError::Reconnect(error));
        }
    };
    write_frame(stream, &StealthBondingChallenge { challenge }.encode())?;
    let assertion = StealthBondingAssertion::decode(&read_frame(stream)?)
        .map_err(|_| HostError::Reconnect(ReconnectError::AssertionFailed))?;
    match accessory.accept_assertion(challenge, assertion) {
        Ok(link) => {
            write_frame(stream, &LinkStatus::ok().encode())?;
            Ok(link.bond_passkey_id.expect("bonded").0)
        }
        Err(error) => {
            write_frame(
                stream,
                &LinkStatus {
                    code: status_for_reconnect(&error),
                }
                .encode(),
            )?;
            Err(HostError::Reconnect(error))
        }
    }
}

fn write_status(stream: &mut TcpStream, code: u8) -> Result<(), HostError> {
    write_frame(stream, &LinkStatus { code }.encode()).map_err(HostError::from)
}

fn status_to_pairing(code: u8) -> HostError {
    match code {
        STATUS_COMMITMENT_FAILED => HostError::Pairing(PairingError::CommitmentFailed),
        STATUS_NO_BOND => HostError::Pairing(PairingError::NotEnrolling),
        _ => HostError::Pairing(PairingError::AttestationFailed),
    }
}

fn status_for_reconnect(error: &ReconnectError) -> u8 {
    match error {
        ReconnectError::NoBond | ReconnectError::NotConnectable => STATUS_NO_BOND,
        ReconnectError::AssertionFailed | ReconnectError::AssertionReplay => {
            STATUS_ASSERTION_FAILED
        }
        ReconnectError::EncryptionFailed => STATUS_ASSERTION_FAILED,
    }
}
