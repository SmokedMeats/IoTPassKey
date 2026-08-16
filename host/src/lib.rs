//! File-vault test Host for Stealth Bonding.
//!
//! The private key lives on disk. Create and assert are silent. There is no
//! passkey sheet. The Accessory peer is another process (or thread) speaking
//! the same length-prefixed PDUs. This is not a phone operating-system vault.

#![forbid(unsafe_code)]

mod session;
mod vault;
mod wire;

pub use session::{
    pair_as_host, reconnect_as_host, serve_accessory_pair, serve_accessory_reconnect, HostError,
    PairingResult,
};
pub use vault::FileVault;
pub use wire::{accept, connect, listen, DEFAULT_ADDR};
