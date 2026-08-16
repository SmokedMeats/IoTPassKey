# Reuse LE Secure Connections for the link; do not invent link math

Low Energy Secure Connections already performs P-256 elliptic-curve Diffie–Hellman and distributes a Long Term Key and Identity Resolving Key. Inventing a new key-agreement for *encrypting the radio* would add risk and block Special Interest Group adoption.

The Bond Passkey is a separate credential: a Host-vault public-key pair (ADR 0006). Just Works is not the trust root for that credential. Factory attestation plus a fresh signed challenge authenticates enrollment and reconnect. Low Energy Secure Connections remains the link cipher after that authorization.

Low Energy Legacy Pairing is forbidden when this mode is selected. “Passkey” means the Bond Passkey, not Bluetooth Passkey Entry. It is the same *class* of credential as a WebAuthn passkey (asymmetric, vault-held, silent here), not a website dialog.
