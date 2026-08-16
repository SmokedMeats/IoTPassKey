use p256::ecdsa::signature::{Signer, Verifier};
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::SecretKey;

pub type Secret = [u8; 32];
pub type PublicKeyBytes = [u8; 33];
pub type SignatureBytes = [u8; 64];
pub type Nonce = [u8; 32];

/// Valid P-256 scalar used as the default Factory Attestation Key in tests.
pub const DEFAULT_FACTORY_SECRET: Secret = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

pub fn public_key(secret: &Secret) -> PublicKeyBytes {
    let key = signing_key(secret);
    let encoded = key.verifying_key().to_encoded_point(true);
    let mut out = [0u8; 33];
    out.copy_from_slice(encoded.as_bytes());
    out
}

pub fn sign(secret: &Secret, message: &[u8]) -> SignatureBytes {
    let key = signing_key(secret);
    let signature: Signature = key.sign(message);
    let bytes = signature.to_bytes();
    let mut out = [0u8; 64];
    out.copy_from_slice(&bytes);
    out
}

pub fn verify(public: &PublicKeyBytes, message: &[u8], signature: &SignatureBytes) -> bool {
    let Ok(key) = VerifyingKey::from_sec1_bytes(public) else {
        return false;
    };
    let Ok(parsed) = Signature::from_slice(signature) else {
        return false;
    };
    key.verify(message, &parsed).is_ok()
}

pub fn attestation_message(
    host_identity: [u8; 6],
    accessory_identity: [u8; 6],
    policy: u8,
    reconnect: u8,
    nonce: &Nonce,
) -> Vec<u8> {
    let mut message = Vec::with_capacity(6 + 6 + 1 + 1 + 32);
    message.extend_from_slice(&host_identity);
    message.extend_from_slice(&accessory_identity);
    message.push(policy);
    message.push(reconnect);
    message.extend_from_slice(nonce);
    message
}

pub fn assertion_message(challenge: &Nonce, accessory_identity: [u8; 6]) -> Vec<u8> {
    let mut message = Vec::with_capacity(32 + 6);
    message.extend_from_slice(challenge);
    message.extend_from_slice(&accessory_identity);
    message
}

pub fn vault_secret_from_identity(identity: [u8; 6]) -> Secret {
    let mut secret = [0u8; 32];
    secret[0] = 0x02;
    secret[26..32].copy_from_slice(&identity);
    secret
}

fn signing_key(secret: &Secret) -> SigningKey {
    let secret_key = SecretKey::from_slice(secret).expect("fixture secret is a valid P-256 scalar");
    SigningKey::from(secret_key)
}
