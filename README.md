# IoT Passkey

Open specification for **Stealth Bonding**: a Bluetooth pairing mode that creates a per-peer **Bond Passkey** and then takes the Accessory off **Public Advertising**. After a successful pair, a normal scan list shows nothing — not a name, not a nameless blip. Bonded Hosts still reconnect.

This is a protocol (SMP + GAP), not an app. Devices get it from firmware and OS stacks, the same way they get other Bluetooth features.

## Read this first

- [Primer (training notes)](docs/primer.md) — start here; every term defined
- [Stealth Bonding Protocol Specification](docs/spec/stealth-bonding.md) — normative
- [Domain glossary](CONTEXT.md)
- [Why SMP/GAP, not GATT](docs/adr/0001-protocol-extension-not-profile.md)
- [Why LE Secure Connections](docs/adr/0002-reuse-le-secure-connections.md)
- [Why the Bond Passkey is a Host-vault credential](docs/adr/0006-host-vault-passkey.md)
- [Why stronger than LE Privacy](docs/adr/0003-no-public-advertising-after-bond.md)
- [Reference implementation / PDU-level simulator](docs/reference-implementation.md)
- [nRF52840 DK buy list and setup](docs/nrf52840-dk-setup.md)
- [Adoption path and demand](docs/adoption-path.md)
- [What you can sell](docs/commercial.md)

## Defaults

| Choice | Value |
| --- | --- |
| Stealth Policy | PeripheralOnly (Accessory hidden, Host still shown) |
| Reconnect | Directed Knock |
| First-pair UX | Existing pairing button; Host is the screen |
| Extra Hosts | Another button hold, or Delegated Enrollment |
| Crypto | Host-vault Bond Passkey + per-unit factory attestation over the LE SC transcript; LE Secure Connections is the link cipher |

## Firmware

Zephyr 4.4 Accessory app in [`firmware/`](firmware/). See [`firmware/README.md`](firmware/README.md). Buy list, downloads, and flash steps: [`docs/nrf52840-dk-setup.md`](docs/nrf52840-dk-setup.md). Custom SMP opcodes are not in this tree; the app does Enrollment, LE SC, and Stealth Commitment at GAP.

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
