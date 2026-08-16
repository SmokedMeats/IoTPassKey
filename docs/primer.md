# Stealth Bonding primer

Training notes from the design conversation. This file teaches the project in plain language. It is not the specification. If this file and the spec disagree, the spec wins.

**Normative spec:** [`spec/stealth-bonding.md`](spec/stealth-bonding.md)
**Project glossary:** [`CONTEXT.md`](../CONTEXT.md)

Every abbreviation is written out the first time it appears. Project terms (Stealth Bonding, Bond Passkey, and the rest) match `CONTEXT.md`. Bluetooth and tooling terms are defined here.

---

## What this project is

**IoT Passkey** is the repository and product name.

**Stealth Bonding** is the protocol: a Bluetooth pairing mode. Two devices that both support it:

1. Create a **Bond Passkey**: the Host vault holds the private key; the Accessory stores the public key. First pair is bound by a **per-unit Factory Attestation Key**, a **Manufacturer Root** on the Host, and a signature over the Low Energy Secure Connections transcript plus a fresh nonce. Later reconnect is an **Assertion** (sign a new challenge). The user does not see or click a passkey.
2. Encrypt the radio with Low Energy Secure Connections (the link cipher, not the trust root).
3. Change the radio so the Accessory is no longer on a normal scan list (**Stealth Commitment**).

It is a **protocol extension**, not an application. You cannot install it on an arbitrary speaker from an app store. Devices get it the same way they get other Bluetooth features: chip firmware and operating-system stack updates.

| Name | What it is | What it is not |
| --- | --- | --- |
| IoT Passkey | This project | A consumer app |
| Stealth Bonding | The pairing mode | “Hide after pair” as a vendor trick |
| Bond Passkey | Host-vault public-key credential (silent) | A typed PIN, Bluetooth Passkey Entry, a labeled Long Term Key, or a website “use passkey?” sheet |

**WebAuthn** (Web Authentication) and **FIDO** (Fast IDentity Online) are web/authenticator standards. This mode uses that *shape* (private key on the Host, public key on the Accessory). It is not a browser dialog.

The **Bluetooth Special Interest Group (SIG)** is the body that adopts Core Specification features. This draft is an individual contribution. It is not a SIG-adopted specification.

---

## The problem

A user pairs a speaker to a laptop. After that, other people’s phones often still see the speaker: sometimes by name, sometimes as a nameless entry. Vendor “hide after pair” is inconsistent. **Low Energy Privacy (LE Privacy)** still sends advertisements that many phones show as a nameless device.

There is no interoperable pairing mode that:

- creates a named credential per peer,
- forces the Accessory off the public scan list,
- still lets bonded Hosts reconnect,
- works on devices that already have a pairing button and a light-emitting diode (LED).

---

## The two roles

**Accessory.** The typically screenless peripheral: speaker, earbuds, keyboard, Internet of Things (IoT) gadget. It stores Bond Passkey public keys (relying party) and applies stealth.

**Host.** The typically screened central: phone, laptop, tablet. It holds the Bond Passkey private key (authenticator), starts pairing, and may authorize adding another peer. No passkey sheet.

**Peer.** The other device in a pairing or reconnect attempt.

Do not use “device,” “client,” or “phone” as role names in spec or code.

---

## Bluetooth layers this project touches

Bluetooth **Low Energy (LE)** is the radio mode used here (not classic Bluetooth audio links).

| Layer | Full name | Job in this project |
| --- | --- | --- |
| **Link Layer** | Controller / radio state machine | Sends advertising packets; can do directed ads |
| **GAP** | Generic Access Profile | Who is discoverable; advertising and inquiry |
| **SMP** | Security Manager Protocol | The pairing conversation (opcodes, keys) |
| **GATT** | Generic Attribute Profile | Services *after* you are connected (battery, audio control, …) |

**Architecture Decision Record (ADR) 0001:** pairing lives in SMP and GAP, not in GATT and not in an app. A GATT service cannot own bonding or advertising. An optional GATT *management* service is allowed only after a Bond Passkey already exists (list, revoke, rename, open enrollment).

**Host stack** (Bluetooth sense) is the software that implements GAP/SMP/GATT on a device. That is a different use of “Host” from the Stealth Bonding **Host** role (the phone/laptop). When this primer means the stack, it says “Bluetooth host stack.”

**Controller** is the Bluetooth radio and Link Layer (often a chip or a firmware block). The host stack talks to it over the **Host Controller Interface (HCI)**.

A **protocol data unit (PDU)** is one on-the-wire message: a Pairing Request, a Stealth Bonding Params packet, one advertising event.

An **opcode** is the first byte of an SMP command that says which message this is.

---

## Pairing, bonding, and ordinary pairing

**Pairing** is the procedure that authenticates two devices and creates keys.

**Bonding** is storing those keys so the devices remember each other later.

**Low Energy Secure Connections (LE Secure Connections, LE SC)** is the current Bluetooth pairing method: elliptic-curve Diffie–Hellman on the National Institute of Standards and Technology P-256 curve, then a **Long Term Key (LTK)** for encryption. **ADR 0002:** reuse this as the *link cipher*. **ADR 0006:** the Bond Passkey is a separate Host-vault credential; Just Works is not the trust root.

**LE Legacy Pairing** is the older, weaker method. Stealth Bonding forbids it.

**Ordinary pairing** is today’s Bluetooth pairing without Stealth Bonding. If the peer does not speak this mode, devices may fall back to ordinary pairing. That fallback does **not** create a Bond Passkey.

**Bluetooth Passkey Entry** is a six-digit association model in the Core Spec (“type 123456 on the other device”). Unrelated to Bond Passkey. Do not shorten Bond Passkey to “passkey” or “PIN.”

---

## What `0xF0` / `0xF1` are

Bluetooth pairing is a conversation on the Security Manager Protocol (SMP). The Core Spec already owns opcodes `0x01` through `0x0E` (Pairing Request, Pairing Response, public keys, confirm, and so on).

Our spec inserts **Phase 1b** after “shall we pair with Low Energy Secure Connections?” and **before** the link public-key exchange. Attestation (`0xF2`) waits until **after** that exchange so the factory signature can cover both sides’ Secure Connections public keys:

| Opcode | PDU name | Meaning |
| --- | --- | --- |
| `0xF0` | Stealth Bonding Params | “I want this Stealth Policy, this reconnect mode, this many bonds.” |
| `0xF1` | Stealth Bonding Params Response | “I accept those values” (or this mode fails). |
| `0xF2` | Attestation | Host request is the 35-octet nonce. Accessory response is the 204-octet per-unit cert plus signature over the LE SC transcript + nonce. |
| `0xF3` | Register | 51-octet Host `credential_id` and vault public key. |

They are **draft placeholders**, not numbers assigned by the Bluetooth Special Interest Group. A phone, laptop, or stock **BlueZ** (the usual Linux Bluetooth host stack) does not send or understand them.

The eight-byte layout is in the spec and in the Rust simulator. The firmware tree does **not** put those bytes on the air.

If either side does not send or accept these PDUs, Stealth Bonding aborts. Ordinary pairing may continue instead.

### Pairing phases (this mode)

| Phase | What happens |
| --- | --- |
| 1 | Pairing Request / Response: agree on Low Energy Secure Connections and bonding |
| 1b | Params (`0xF0`/`0xF1`), then Attestation request (`0xF2`, nonce). No factory signature yet. |
| 2 | Public-key exchange (LE Secure Connections — link cipher) |
| 2b | Attestation response (`0xF2`) over the SC transcript and nonce; silent vault create; register (`0xF3`) |
| 3 | Key distribution (LTK, identity, optional Identity Resolving Key) |
| 4 | **Stealth Commitment** — change advertising; only then report success |

Keys in Phase 3 are **tentative**. They become a Bond Passkey only after Phase 4 succeeds.

---

## Why “needs a Zephyr host patch”

**Zephyr** is a real-time operating system for microcontrollers. This repo’s first Accessory firmware is a Zephyr 4.4 application.

The pairing engine lives in Zephyr’s Bluetooth host stack (`subsys/bluetooth/host/smp.c`), not in our application. Incoming SMP packets are dispatched from a fixed `handlers[]` table. Unknown opcodes are dropped as reserved. There is no application programming interface to register a new SMP opcode.

So `0xF0` / `0xF1` cannot be added in `firmware/accessory` alone. Someone would change Zephyr itself (a **west** overlay or a private Zephyr branch), extend that table, and splice Phase 1b (params) plus Phase 2b (attestation after the Secure Connections public keys) into the pairing sequence. **west** is Zephyr’s workspace tool: it clones Zephyr and modules next to this git tree.

That is a stack fork, not an app feature.

**Do not do that patch yet.** A patched Accessory still cannot Stealth-Bond to a shipping phone or laptop. Those stacks ignore unknown SMP commands. To see `0xF0` / `0xF1` on the air you need a second patched peer. The Rust simulator is already the oracle for those PDUs. Do the host patch when there is a second patched stack, or when preparing a SIG-facing reference.

**nRF Connect Software Development Kit (nRF Connect SDK, NCS)** is Nordic Semiconductor’s Zephyr-based product software development kit. Research noted NCS version 3.4.0 as a Nordic path. **ADR 0005:** first firmware uses vanilla Zephyr 4.4 in `firmware/`, not NCS.

Do not confuse SMP (Security Manager Protocol) with **MCUmgr SMP** (Simple Management Protocol), a firmware-update protocol that *can* register custom groups from application code. Unrelated.

---

## Public Advertising, LE Privacy, and reconnect

**Public Advertising** is undirected advertising or inquiry-scan that a typical operating-system scan list can show — named **or** anonymous. A nameless “blip” still counts. After a successful Peripheral-side Stealth Commitment, Public Advertising is forbidden. **ADR 0003.**

**Undirected advertising** is “anyone nearby may see and connect.” **Directed advertising** is “this specific peer may connect.”

**LE Privacy** rotates the device address using an **Identity Resolving Key (IRK)** and a **Resolvable Private Address (RPA)**. Bonded peers can still recognize the device. LE Privacy is not enough: undirected ads with rotating addresses still appear on many scan lists.

**Directed Knock** is connectable directed advertising to a bonded peer’s identity, with no local name and no general-discovery payload. Default reconnect method.

**SilentUntilWake** is no advertising until the user (or a bonded Host) opens a short window, then directed advertising.

**Filter Accept List (FAL)** is a Link Layer allowlist: the controller ignores unknown initiators. It is an aid, not a substitute for leaving Public Advertising. Filter Accept List plus undirected ads can still appear on a public scan list. The **Bond Table** is not the Filter Accept List.

**Radio invisibility against a sniffer is out of scope.** Packets may still exist on the air. The requirement is that a normal Settings → Bluetooth list shows nothing.

---

## Stealth Policy

**Stealth Policy** is the negotiated rule for which side leaves public discoverability after Stealth Commitment.

| Value | Accessory | Host |
| --- | --- | --- |
| **PeripheralOnly** (default) | Hidden | Still shown |
| **CentralOnly** | Still shown | Hidden |
| **Both** | Hidden | Hidden |
| **Neither** | Still shown | Still shown |

Default is PeripheralOnly: the speaker disappears; the laptop does not.

If Stealth Commitment cannot leave Public Advertising when the policy requires it, pairing **fails** and tentative keys are discarded. No Bond Passkey is stored.

---

## Enrollment

**Enrollment Window.** A time-bounded **Limited Discoverable** period used to add a Bond Passkey. Opened by the existing pairing button (or equivalent gesture) or by Delegated Enrollment. Default in this firmware: 30 seconds, Button 1, LED 0 on.

**Limited Discoverable** is a Generic Access Profile mode: the device is findable for a short time, not forever.

Do not call this “pairing mode” in normative text. Bluetooth already uses that phrase.

**Delegated Enrollment.** An already-bonded Host opens an Enrollment Window on a screenless Accessory so a second Host can be added without a new hardware control.

**Unprovisioned.** The Accessory’s Bond Table is empty.

One Accessory may hold **multiple Bond Passkeys** (several trusted Hosts). A full table refuses a new enrollment.

**Factory reset** clears the Bond Table and returns the Accessory to Unprovisioned.

---

## What is in this repository

| Piece | Path | What it proves |
| --- | --- | --- |
| Protocol specification | `docs/spec/stealth-bonding.md` | The rules (normative) |
| Project glossary | `CONTEXT.md` | Allowed names |
| Architecture Decision Records | `docs/adr/` | Locked choices |
| Rust simulator | `sim/` | PDU-level oracle; no radio |
| Accessory firmware | `firmware/accessory` | Generic Access Profile half on stock Zephyr |

**Rust simulator** (`stealth-bonding-sim`). Two in-process objects exchange structured PDUs. Tests assert: after commitment the Accessory is not on Public Advertising; a second Bond Passkey can enroll; commitment failure stores nothing; unbonded Delegated Enrollment is rejected. Low Energy Secure Connections math is stubbed. Params PDUs are real bytes (example: `F0 01 00 00 00 08 00 00`). Lint is `rustfmt` and `clippy`. Biome is a JavaScript/TypeScript tool and does not apply.

**Firmware.** Enrollment Window, Low Energy Secure Connections bonding, stop undirected advertising, Directed Knock. Not the draft SMP Params extension. Not a download for arbitrary speakers. Not nRF Connect SDK.

**Continuous integration (CI)** compiles the firmware for the development kit without attaching hardware. That is a compile check, not a radio test.

---

## The development board

Default hardware is the **nRF52840 Development Kit (DK)** from Nordic Semiconductor (board name PCA10056). Zephyr target string: `nrf52840dk/nrf52840`.

It is a single-board kit: the **nRF52840** system on chip (Arm Cortex-M4F), onboard **J-Link** debugger (SEGGER’s debug probe), four buttons, four LEDs, USB power.

| | |
| --- | --- |
| Nordic product page | [nRF52840 DK](https://www.nordicsemi.com/Products/Development-hardware/nRF52840-DK) |
| Authorized distributor example | [Digi-Key NRF52840-DK](https://www.digikey.com/en/products/detail/nordic-semiconductor-asa/NRF52840-DK/8593726) (about **$49** USD, one-off, as of 2026-08) |

Nordic does not sell the kit from their own cart. Digi-Key, Mouser, and Farnell are the usual path.

One board is enough for the hide-after-pair demo: pair from a laptop, confirm a second phone no longer sees the Accessory.

Full buy list, download links, and Windows setup: [`nrf52840-dk-setup.md`](nrf52840-dk-setup.md).

---

## What this is not, and what we are not doing yet

- Not an app, Android Package (APK), iOS App Store package (IPA), or driver that enables this on any existing radio.
- Not a GATT-only profile that tries to own advertising and bonding.
- Not new pairing mathematics.
- Not radio invisibility against a sniffer.
- Not a replacement for WebAuthn or FIDO authenticators.
- Not a patent play. The spec is published as a defensive publication of the high-level mode. Money, if any, sits next to the protocol (modules, software development kit, certification badge, consulting). See [`commercial.md`](commercial.md).

**Not next:** Zephyr SMP host patch; phone silent-vault ceremony; SIG paperwork. The file-vault test Host in `host/` is the next protocol proof after the simulator (attestation + assertion on a TCP link). The Development Kit still only proves the Generic Access Profile hide. See [`adoption-path.md`](adoption-path.md).

---

## Terms used in this document

Project terms (full definitions and forbidden synonyms): [`CONTEXT.md`](../CONTEXT.md).

| Term | Meaning |
| --- | --- |
| Accessory | Screenless peripheral role |
| ADR | Architecture Decision Record — a locked design choice in `docs/adr/` |
| Assertion | Host signs a fresh challenge with the vault key |
| Bond Passkey | Host-vault public-key credential; Accessory stores the public key |
| Bond Table | Accessory store of public keys and policy (not the Filter Accept List) |
| Factory Attestation Key | Per-unit manufacture-time Accessory signing key; does not rotate |
| Manufacturer Root | Brand public key on the Host; verifies each unit certificate |
| BlueZ | Common Linux Bluetooth host stack |
| CI | Continuous integration — automated build/test |
| Controller | Bluetooth radio + Link Layer |
| Delegated Enrollment | Bonded Host opens an Enrollment Window for a new peer |
| Directed Knock | Directed advertising to a known peer, no public name |
| DK | Development kit |
| Enrollment Window | Short Limited Discoverable period to add a Bond Passkey |
| FAL | Filter Accept List — Link Layer ignore-unknown-initiators list |
| FIDO | Fast IDentity Online — authenticator standards; not this mode |
| GAP | Generic Access Profile — discoverability and advertising |
| GATT | Generic Attribute Profile — connected services |
| HCI | Host Controller Interface — host stack ↔ controller |
| Host (role) | Screened central; holds the vault private key |
| Host Vault | Silent platform store for Bond Passkey private keys |
| Host stack | Bluetooth software implementing GAP/SMP/GATT |
| IoT | Internet of Things |
| IRK | Identity Resolving Key — used with LE Privacy |
| J-Link | Onboard debugger on the Nordic development kit |
| LE | Low Energy (Bluetooth LE) |
| LE Privacy | Rotating addresses; still advertises undirected |
| LE SC | Low Energy Secure Connections — link cipher, not the Bond Passkey |
| LED | Light-emitting diode |
| LTK | Long Term Key — encryption key from pairing |
| NCS | nRF Connect Software Development Kit (Nordic’s Zephyr product kit) |
| Opcode | First byte of an SMP command |
| Ordinary pairing | Bluetooth pairing without Stealth Bonding; no Bond Passkey |
| OS | Operating system |
| PDU | Protocol data unit — one on-the-wire message |
| Peer | The other device in this attempt |
| Public Advertising | Ads a normal scan list can show (named or nameless) |
| RPA | Resolvable Private Address |
| SIG | Bluetooth Special Interest Group |
| SilentUntilWake | No ads until a short directed window is opened |
| SMP | Security Manager Protocol — pairing conversation |
| SoC | System on chip |
| Stealth Bonding | This pairing mode |
| Stealth Commitment | Required advertising change before pairing is “done” |
| Stealth Policy | Which side(s) leave public discoverability |
| Unprovisioned | Bond Table empty |
| west | Zephyr workspace tool |
| Zephyr | Real-time operating system used for first Accessory firmware |
