//! Stealth Bonding PDU simulator.
//!
//! Public seam: pairing, advertising visibility, and the Bond Table.
//! Per-unit factory certificates, transcript-bound attestation (including stubbed
//! Low Energy Secure Connections public keys), and vault assertions are real P-256
//! signatures. Link math stays stubbed; Params PDUs are real bytes.

#![forbid(unsafe_code)]

mod crypto;
mod model;
mod pdu;

pub use model::{
    pair, reconnect, Accessory, Advertising, BondPasskeyId, BondedLink, DelegatedError, Host,
    PairingError, PairingMode, PairingOffer, PairingOutcome, PduError, ReconnectError,
    ReconnectMode, StealthPolicy, T_SB_ENROLL_MAX_MS, T_SB_ENROLL_MS,
};
pub use pdu::{
    StealthBondingParams, PARAMS_LEN, PARAMS_OPCODE, PARAMS_RESPONSE_OPCODE, PARAMS_VERSION,
};
