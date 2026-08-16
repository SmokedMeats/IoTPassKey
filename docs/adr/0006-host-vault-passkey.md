# Bond Passkey is a Host-vault public-key credential

A Bond Passkey is a discoverable public-key credential. The Host platform vault holds the private key. The Accessory stores the public key, like a website stores a WebAuthn credential.

First pair is authenticated by a **per-unit** factory certificate and a Factory Attestation Key on that Accessory. The Host trusts a **Manufacturer Root**. Attestation is signed **after** Low Energy Secure Connections Phase 2 and MUST cover both sides’ SC public keys plus a Host nonce. The user does not see a secret, QR, number, or “use passkey?” sheet. Gestures are only the existing pair button and the Host’s normal Bluetooth pick-a-device row. Audio products SHOULD add an automatic acoustic check (no new UI). Hosts MAY apply a closeness threshold during the Enrollment Window.

Low Energy Secure Connections remains the link cipher (Long Term Key). Just Works is not the trust root for a Bond Passkey. Freshness is a random challenge and a signature, not a time-based one-time password.

If a Host cannot verify the unit certificate or the transcript-bound attestation, it must refuse Stealth Bonding (ordinary pairing may still happen; that is not a Bond Passkey).

A same-room evil twin remains residual unless the user notices which unique scan row they tapped. Forbidding two devices from advertising the same Local Name is not enforceable and does not close that residual.
