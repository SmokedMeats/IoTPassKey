# IoT Passkey

Open specification for **Stealth Bonding**: a Bluetooth pairing mode that creates a per-peer **Bond Passkey** and then takes the Accessory off **Public Advertising**. After a successful pair, a normal scan list shows nothing — not a name, not a nameless blip. Bonded Hosts still reconnect.

This is a protocol (SMP + GAP), not an app. Devices get it from firmware and OS stacks, the same way they get other Bluetooth features.

## Read this first

- [Stealth Bonding Protocol Specification](docs/spec/stealth-bonding.md) — normative
- [Domain glossary](CONTEXT.md)
- [Why SMP/GAP, not GATT](docs/adr/0001-protocol-extension-not-profile.md)
- [Why LE Secure Connections](docs/adr/0002-reuse-le-secure-connections.md)
- [Why stronger than LE Privacy](docs/adr/0003-no-public-advertising-after-bond.md)
- [Reference implementation / PDU-level simulator](docs/reference-implementation.md)
- [What you can sell](docs/commercial.md)

## Defaults

| Choice | Value |
| --- | --- |
| Stealth Policy | PeripheralOnly (Accessory hidden, Host still shown) |
| Reconnect | Directed Knock |
| First-pair UX | Existing pairing button; Host is the screen |
| Extra Hosts | Another button hold, or Delegated Enrollment |
| Crypto | LE Secure Connections (not a new cipher) |

## Simulator

Rust crate [`sim/`](sim/) — PDU-level spec oracle, no radio. Lint is `rustfmt` + `clippy` (Biome is a JS/TS tool and does not apply).

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

On Windows without Visual Studio Build Tools, use the GNU toolchain: `rustup default stable-x86_64-pc-windows-gnu`.

## Status

Draft 0.1. Individual contribution. Not a Bluetooth SIG adopted specification.
