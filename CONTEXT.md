# IoT Passkey

Domain language for a Bluetooth pairing-mode extension: per-peer credentials plus mandatory post-bond stealth. This file is a glossary, not a spec.

Plain-language training notes, including Bluetooth and tooling terms: [`docs/primer.md`](docs/primer.md).

## Language

**Stealth Bonding**:
The pairing mode specified by this project: an attested Host-vault Bond Passkey, Low Energy Secure Connections for the radio link, and a required advertising-state change so the Accessory is no longer publicly discoverable.
_Avoid_: Stealthkeys, Private Secure Connections, passkey pairing, hide-after-pair

**Bond Passkey**:
A discoverable public-key credential created when Stealth Bonding succeeds. The Host platform vault holds the private key. The Accessory stores the public key. It is not a typed PIN, not Bluetooth Passkey Entry, and not a labeled Long Term Key. The user does not see or click it during pair.
_Avoid_: passkey (unqualified), PIN, password, account key, labeled LTK

**Host Vault**:
The Host operating-system store for Bond Passkey private keys (the same class of store as platform WebAuthn passkeys). Create and assert are silent inside the Bluetooth stack. A “use passkey?” sheet is non-conformant.
_Avoid_: password manager UI, authenticator app, Authy

**Factory Attestation Key**:
A per-unit asymmetric key burned into that Accessory at manufacture. It signs the pairing transcript after Low Energy Secure Connections Phase 2. The signed bytes include both sides’ SC public keys and a Host nonce. It does not rotate. A stolen unit key enrolls only that unit.
_Avoid_: TOTP seed, pairing PIN, Authy secret, shared model key

**Manufacturer Root**:
The public key the Host already trusts for a brand. It verifies each unit’s factory certificate. It is not the unit’s attestation key. A stolen root is a brand-wide incident.
_Avoid_: model public key (unqualified), factory PIN

**Assertion**:
A signature over a fresh challenge proving the Host still holds the Bond Passkey private key. Proofs rotate; the root credential does not.
_Avoid_: one-time password, rotating digits, TOTP

**Bond Table**:
The Accessory’s store of Bond Passkey public keys and policy. The Host Vault is the Host’s store of private keys. These are not the Filter Accept List.
_Avoid_: paired-device list, allowlist (the Filter Accept List is a Link Layer mechanism, not this store)

**Stealth Policy**:
The negotiated rule for which side(s) leave public discoverability after Stealth Commitment. Values: PeripheralOnly, CentralOnly, Both, Neither.
_Avoid_: hide setting, visibility flag

**Stealth Commitment**:
The required GAP/Link Layer transition that runs after keys are stored and before pairing is reported complete. If it cannot be applied, the Bond Passkey is discarded.
_Avoid_: hide step, stealth toggle

**Enrollment Window**:
A time-bounded Limited Discoverable period used to add a Bond Passkey. Opened by the existing pairing gesture or by Delegated Enrollment. A Host that already holds a synced vault credential for this Accessory does not need a new window.
_Avoid_: pairing mode (unqualified; Bluetooth already uses that phrase), discoverable mode (too broad)

**Delegated Enrollment**:
Opening an Enrollment Window from an already-bonded Host so a screenless Accessory can add another Bond Passkey (a Host that does not already hold that credential) without a new hardware control.
_Avoid_: remote pairing, admin pairing

**Public Advertising**:
Undirected advertising or inquiry-scan that a typical OS scan list can show as a named or anonymous device. Stealth Bonding forbids this after a successful Peripheral-side commitment.
_Avoid_: blip, broadcast, beacon

**Directed Knock**:
Connectable directed advertising to a bonded peer’s identity, with no Local Name and no general-discovery payload. Used to reconnect without Public Advertising.
_Avoid_: ping, private advertising (LE Privacy is a different feature)

**Accessory**:
The typically screenless peripheral (speaker, buds, keyboard, IoT gadget) that stores Bond Passkey public keys and applies stealth. It is the relying party.
_Avoid_: device (unqualified), gadget, provider

**Host**:
The typically screened central (phone, laptop, tablet) that holds the Bond Passkey private key, initiates pairing, and may authorize Delegated Enrollment. It is the authenticator.
_Avoid_: phone, computer, seeker, client (unqualified)
