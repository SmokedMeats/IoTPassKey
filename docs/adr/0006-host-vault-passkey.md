# Bond Passkey is a Host-vault public-key credential

A Bond Passkey is a discoverable public-key credential. The Host platform vault holds the private key. The Accessory stores the public key, like a website stores a WebAuthn credential.

First pair is authenticated by a factory attestation key on the Accessory: it signs the pairing transcript plus a Host nonce. The user does not see a secret, QR, number, or “use passkey?” sheet. Gestures are only the existing pair button and the Host’s normal Bluetooth pick-a-device row.

Low Energy Secure Connections remains the link cipher (Long Term Key). Just Works is not the trust root for a Bond Passkey. Freshness is a random challenge and a signature, not a time-based one-time password.

If a Host cannot verify attestation, it must refuse Stealth Bonding (ordinary pairing may still happen; that is not a Bond Passkey).
