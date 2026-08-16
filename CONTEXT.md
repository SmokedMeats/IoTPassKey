# IoT Passkey

Domain language for a Bluetooth pairing-mode extension: per-peer credentials plus mandatory post-bond stealth. This file is a glossary, not a spec.

## Language

**Stealth Bonding**:
The pairing mode specified by this project: LE Secure Connections bonding plus a required advertising-state change so the Accessory is no longer publicly discoverable.
_Avoid_: Stealthkeys, Private Secure Connections, passkey pairing, hide-after-pair

**Bond Passkey**:
The named, revocable credential created for one peer when Stealth Bonding succeeds. It is a stored bond plus policy, not a typed PIN and not a WebAuthn passkey.
_Avoid_: passkey (unqualified), PIN, password, account key

**Bond Table**:
The Accessory’s (and Host’s) store of Bond Passkeys.
_Avoid_: paired-device list, allowlist (the Filter Accept List is a Link Layer mechanism, not this store)

**Stealth Policy**:
The negotiated rule for which side(s) leave public discoverability after Stealth Commitment. Values: PeripheralOnly, CentralOnly, Both, Neither.
_Avoid_: hide setting, visibility flag

**Stealth Commitment**:
The required GAP/Link Layer transition that runs after keys are stored and before pairing is reported complete. If it cannot be applied, the Bond Passkey is discarded.
_Avoid_: hide step, stealth toggle

**Enrollment Window**:
A time-bounded Limited Discoverable period used to add a Bond Passkey. Opened by the existing pairing gesture or by Delegated Enrollment.
_Avoid_: pairing mode (unqualified; Bluetooth already uses that phrase), discoverable mode (too broad)

**Delegated Enrollment**:
Opening an Enrollment Window from an already-bonded Host so a screenless Accessory can add another Bond Passkey without a new hardware control.
_Avoid_: remote pairing, admin pairing

**Public Advertising**:
Undirected advertising or inquiry-scan that a typical OS scan list can show as a named or anonymous device. Stealth Bonding forbids this after a successful Peripheral-side commitment.
_Avoid_: blip, broadcast, beacon

**Directed Knock**:
Connectable directed advertising to a bonded peer’s identity, with no Local Name and no general-discovery payload. Used to reconnect without Public Advertising.
_Avoid_: ping, private advertising (LE Privacy is a different feature)

**Accessory**:
The typically screenless peripheral (speaker, buds, keyboard, IoT gadget) that holds Bond Passkeys and applies stealth.
_Avoid_: device (unqualified), gadget, provider

**Host**:
The typically screened central (phone, laptop, tablet) that initiates pairing, shows UI, and may authorize Delegated Enrollment.
_Avoid_: phone, computer, seeker, client (unqualified)
