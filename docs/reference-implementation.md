# Reference implementation path

How this repo can prove the spec before anyone flashes a speaker.

## PDU-level simulator

A **PDU** (protocol data unit) is one on-the-wire message: Pairing Request, Stealth Bonding Params, a directed advertising event, and so on.

A **PDU-level simulator** is a program that pretends to be a Host and an Accessory and exchanges those messages according to the spec. There is no Bluetooth radio. Two objects in process (or two processes) send structured PDUs to each other. A test harness asserts the outcomes the spec cares about:

- After Stealth Commitment, the Accessory’s advertising state is not Public Advertising (including no nameless undirected ads).
- Registration without a valid per-unit certificate or transcript-bound attestation fails; a replayed nonce fails; an attestation that omits the Low Energy Secure Connections public keys fails.
- Pairing records golden `0xF0`/`0xF1` Params, `0xF2` attestation request/response, and `0xF3` register bytes.
- Assertion reconnect succeeds; a replayed challenge fails.
- A Host that shares a vault `credential_id` reconnects without a new Enrollment Window; a stranger Host cannot.
- A second Bond Passkey can be enrolled and both peers can reconnect.
- If commitment cannot hide the Accessory, pairing fails and the Bond Table stays empty.
- An unbonded peer cannot open Delegated Enrollment.

That is stricter than a slide-deck state machine (which never checks field layout) and cheaper than firmware on a Nordic board (which needs silicon, a sniffer, and two real devices).

It is the first adapter behind the pairing-plus-advertising state machine. A later C/Zephyr port should speak the same states and the same PDU names so conformance tests stay put.

## What it is not

- Not an app users install on a phone to “enable Stealth Bonding” on random speakers.
- Not a full Bluetooth controller. Link Layer timing, PHYs, and RF are stubbed or recorded as advertising-state facts (“Public Advertising: on/off”).
- Not a SIG qualification suite. It is a spec oracle for this draft.

## Where it lives

**This repo.** Spec, state machine, simulator, and golden traces stay together so the packets cannot drift from [`spec/stealth-bonding.md`](spec/stealth-bonding.md).

Do not start a second repository for the simulator. Split later only if something becomes a different product:

| Later split | When |
| --- | --- |
| Firmware tree (Zephyr / nRF) | The board project is large and has its own SDK/vendor pins |
| Certification / test-tool product | You are selling the badge in [`commercial.md`](commercial.md) |

Until then: `docs/spec/` is the protocol; the simulator is a Rust crate under `sim/`; the file-vault test Host is `host/`; the first chip adapter is `firmware/accessory` (Zephyr 4.4, `nrf52840dk/nrf52840`).

## Language

**Rust** for the simulator. Stealth Policy, advertising states, and pairing phases are enums; type-state can make “paired but still publicly advertising” hard to represent. Golden traces are byte slices. The same crate can later become a shipped test tool.

| Layer | Language | Why |
| --- | --- | --- |
| PDU simulator (this repo, first) | Rust | Binary PDUs + state machine; stays the oracle if you sell a test tool |
| Firmware adapter (later) | C | Nordic / Zephyr / NimBLE stacks |

Do not write the simulator in C. That language is for the chip, after the packets are stable. Python was considered for draft speed and rejected so this crate does not get rewritten.

The crate is `stealth-bonding-sim` in `sim/`. Integration tests live in `sim/tests/conformance.rs` and hit the pairing-plus-advertising seam only. Per-unit factory certificates, transcript-bound attestation (including stubbed Low Energy Secure Connections public keys), and vault assertions use real P-256 signatures. Link math stays stubbed (fixture Long Term Key). Params, attestation, and register PDUs are fixed-length byte arrays. CI runs `cargo fmt`, `clippy -D warnings`, and `cargo test` (not Biome).

## Testing ladder

Pairing a Development Kit from a phone’s Settings app is **not** a Bond Passkey test. Layers:

| Layer | Proves |
| --- | --- |
| Simulator (`sim/`) | Per-unit cert, SC-transcript attestation, assertion, sync vs stranger, unique scan rows, stealth |
| File-vault test Host (`host/`) | Same checks on a TCP link we control; private key on disk |
| nRF52840 Development Kit + laptop | Generic Access Profile hide only |
| Kit + test Host | First on-air attestation + assertion |
| BlueZ, then phone OS | Real Host vault |

Full order and demand notes: [`adoption-path.md`](adoption-path.md).

## Suggested order

1. Simulator + golden PDU traces and P-256 fixtures (this repo).
2. Firmware adapter (`firmware/accessory`) on nRF52840 DK — GAP hide only on stock Zephyr 4.4.
3. File-vault test Host (`host/`, binaries `sb-host` and `sb-accessory`), then BlueZ.
4. Zephyr host patch for draft SMP opcodes, if we want those PDUs on the air.

Commercial products that might sit on top of that path (modules, SDK, certified badge) are listed in [`commercial.md`](commercial.md).
