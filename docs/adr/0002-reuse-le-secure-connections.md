# Reuse LE Secure Connections; do not invent pairing math

LE Secure Connections already performs P-256 ECDH and distributes LTK and IRK. Inventing a new key-agreement would add risk and block SIG adoption.

Stealth Bonding is an association and state-machine extension on top of LE SC: Bond Passkey records, Stealth Policy negotiation, and Stealth Commitment. LE Legacy Pairing is forbidden when this mode is selected. “Passkey” here means the stored Bond Passkey, not Bluetooth Passkey Entry and not WebAuthn.
