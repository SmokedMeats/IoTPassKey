//! Stealth Bonding PDU simulator.
//!
//! Public seam: pairing, advertising visibility, and the Bond Table.
//! Per-unit factory certificates, transcript-bound attestation (including stubbed
//! Low Energy Secure Connections public keys), and vault assertions are real P-256
//! signatures. Link math stays stubbed. Params, attestation, and register PDUs are
//! real bytes (`0xF0`–`0xF3`).

#![forbid(unsafe_code)]

mod crypto;
mod model;
mod pdu;

pub use crypto::{
    assertion_message, attestation_message, fixture_sc_public, generate_credential_id,
    generate_identity, generate_nonce, generate_secret, public_key, sign, unit_cert_message,
    verify, Nonce, PublicKeyBytes, Secret, SignatureBytes, DEFAULT_MANUFACTURER_SECRET,
};
pub use model::{
    pair, reconnect, Accessory, Advertising, BondPasskeyId, BondedLink, DelegatedError, Host,
    PairingError, PairingMode, PairingOffer, PairingOutcome, PduError, ReconnectError,
    ReconnectMode, StealthPolicy, T_SB_ENROLL_MAX_MS, T_SB_ENROLL_MS,
};
pub use pdu::{
    StealthBondingAssertion, StealthBondingAttestation, StealthBondingAttestationRequest,
    StealthBondingChallenge, StealthBondingParams, StealthBondingRegister, ASSERTION_LEN,
    ASSERTION_OPCODE, ATTESTATION_FLAG_REQUEST, ATTESTATION_FLAG_RESPONSE, ATTESTATION_LEN,
    ATTESTATION_OPCODE, ATTESTATION_REQUEST_LEN, CHALLENGE_LEN, CHALLENGE_OPCODE, PARAMS_LEN,
    PARAMS_OPCODE, PARAMS_RESPONSE_OPCODE, PARAMS_VERSION, REGISTER_LEN, REGISTER_OPCODE,
};
