# Stealth Bonding Protocol Specification

| Field | Value |
| --- | --- |
| Version | 0.1.0-draft |
| Status | Individual draft. Not a Bluetooth SIG adopted specification. |
| Project | IoT Passkey |
| Date | 2026-08-16 |
| Intended venue | Bluetooth Core Specification extension (SMP + GAP), with an optional GATT management service |
| License | Creative Commons Attribution 4.0 International (CC BY 4.0) |

This document is the normative specification. Domain terms match `CONTEXT.md`. Architectural decisions that this draft locks in are recorded in `docs/adr/`.

---

## Abstract

Stealth Bonding is a Bluetooth pairing mode. Two devices that both support it create a **Bond Passkey** (a Host-vault public-key credential; the Accessory stores the public key), encrypt the link with Low Energy Secure Connections, and then apply a **Stealth Commitment**: the Accessory (and, if negotiated, the Host) leaves **Public Advertising**. A normal operating-system scan list SHALL NOT show the Accessory as a named or anonymous device.

The user does not see or click a passkey. Gestures are the Accessory’s existing pairing button and the Host’s normal Bluetooth device list. First enrollment is bound by a **per-unit Factory Attestation Key**, a **Manufacturer Root** on the Host, and a signature that covers the Low Energy Secure Connections transcript plus a fresh Host nonce. Later reconnect is an **Assertion** over a new challenge.

The mode is designed for screenless Accessories that already have a pairing button and an LED. It does not require new hardware. A device may hold multiple Bond Passkeys. A Host that already holds a synced vault credential for this Accessory reconnects without a new Enrollment Window. Additional strangers are enrolled through the same pairing gesture or through **Delegated Enrollment** from an already-bonded Host.

Stealth Bonding is a protocol-level feature. It cannot be installed as an application on an arbitrary Bluetooth device.

---

## 1. Introduction

### 1.1 Motivation

Bluetooth pairing and discoverability are already protocol concerns (Link Layer, Security Manager, GAP). After a typical speaker is paired, many products hide themselves as a vendor policy. That behavior is not interoperable, not mandatory, and not bound to a named, revocable credential.

LE Privacy still transmits undirected advertisements. Those packets often appear as a nameless device in a scan list. Google Fast Pair and FIDO-over-BLE are related prior art; neither is a Core Spec pairing mode that forbids Public Advertising after bond.

WebAuthn/FIDO passkeys are a credential model for authenticators. This mode uses that *shape* (Host-held private key, Accessory-held public key, silent create/assert) as a Core pairing credential. It is not a website dialog and not FIDO-over-BLE.

The gap is a standardized pairing mode that:

1. Creates a Bond Passkey: Host vault private key, Accessory public key, per-unit factory attestation over the Low Energy Secure Connections transcript at enroll, assertion at reconnect.
2. Forces the Accessory out of Public Advertising when pairing succeeds.
3. Still allows bonded Hosts to reconnect without Public Advertising.
4. Works on devices that will not be redesigned, with only the existing pair button.

### 1.2 Goals

- Interoperable pairing across Hosts and Accessories that implement this specification.
- Default one-way stealth: Accessory hidden, Host still shown.
- Optional two-way or no-stealth policies, negotiated per bond.
- Multiple Bond Passkeys on one Accessory.
- Existing pairing-button UX. No new buttons, screens, or LEDs.
- Fallback to existing Bluetooth pairing when the peer does not support this mode.
- Open specification suitable for chip, firmware, and OS implementation.

### 1.3 Non-goals

- A downloadable app, APK, IPA, or driver that enables this on any existing radio.
- A GATT-only profile that tries to own advertising and bonding.
- New *link* mathematics. Low Energy Secure Connections remains the radio cipher.
- A visible passkey sheet, account picker, QR, numeric compare, or typed one-time password.
- Authy / time-based one-time-password recognizers.
- Radio invisibility against a sniffer. Physics is out of scope.
- Replacing WebAuthn for websites, or requiring a browser API on the air.
- Requiring a companion application for first pairing.
- Patent claims. This draft is published so the procedure can be implemented.

### 1.4 Relationship to existing Bluetooth features

| Feature | What it already does | What Stealth Bonding adds |
| --- | --- | --- |
| LE Secure Connections | P-256 ECDH, LTK | Link cipher only. Not the Bond Passkey trust root |
| LE Privacy / IRK / RPA | Bonded peers can resolve rotating addresses | Address rotation for privacy. Not a one-time password. Undirected ads may still blip |
| GAP Limited Discoverable | Time-bounded discovery | Used only for the Enrollment Window |
| Bonding / bond table | Stores LTKs per peer | Bond Table stores public keys + policy; Host Vault stores private keys |
| Filter Accept List | Controller ignores unknown initiators | Allowed as an aid, not a substitute for leaving Public Advertising |
| Bluetooth Passkey Entry | Six-digit association model | Unrelated. Do not confuse with Bond Passkey |
| WebAuthn / platform passkeys | Website credentials | Same credential *class*; silent Bluetooth ceremony, no browser sheet |
| Fast Pair / FIDO BLE | Vendor or authenticator profiles | Out of band to this Core pairing mode |

---

## 2. Conventions

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) and [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174) when, and only when, they appear in all capitals.

Normative text uses those keywords. Examples, appendices marked informative, and notes are non-normative.

This specification refers to the Bluetooth Core Specification, Host volume:

- GAP — Generic Access Profile, [Vol 3] Part C
- SMP — Security Manager Protocol, [Vol 3] Part H
- GATT — Generic Attribute Profile, [Vol 3] Part G

and to the Low Energy Controller volume:

- Link Layer advertising PDUs, [Vol 6] Part B

---

## 3. Terminology

Terms in **bold** are defined in `CONTEXT.md` and are used with those meanings here.

Additional terms used only in this specification:

**Peer.** The other device in a pairing or reconnection attempt.

**Unprovisioned.** An Accessory whose Bond Table is empty.

**Public scan list.** The list of devices an ordinary Host OS shows during Bluetooth discovery (Settings → Bluetooth, and equivalents).

**Controller.** The Bluetooth controller, including the Link Layer.

**Host stack.** The Bluetooth host implementation on either device (not the Stealth Bonding Host role).

**Factory Attestation Key.** The Accessory’s per-unit manufacture-time signing key (see `CONTEXT.md`).

**Manufacturer Root.** The public key the Host trusts for a brand. It verifies per-unit factory certificates. It is not the unit’s attestation key.

**Host Vault.** The Host store of Bond Passkey private keys.

**Assertion.** A signature over a fresh challenge proving possession of the Bond Passkey private key.

**Credential id.** The 16-octet identifier of one Bond Passkey, shared by synced Hosts that hold the same vault credential.

---

## 4. Architecture

### 4.1 Layering

Stealth Bonding is a pairing-mode extension. It SHALL be implemented in the host Security Manager and GAP, with Controller support for the advertising states in Section 7.

```
┌─────────────────────────────────────────────┐
│  Host OS UI / Accessory pairing button      │  existing UX
├─────────────────────────────────────────────┤
│  Stealth Bonding Management Service (GATT)  │  OPTIONAL, post-bond only
├─────────────────────────────────────────────┤
│  GAP  — discoverability, advertising policy │  REQUIRED
│  SMP  — pairing, Bond Passkey, commitment   │  REQUIRED
├─────────────────────────────────────────────┤
│  L2CAP / HCI                                │
├─────────────────────────────────────────────┤
│  Link Layer — Public Advertising off;       │  REQUIRED
│               Directed Knock / silent       │
└─────────────────────────────────────────────┘
```

A GATT service SHALL NOT be used to perform pairing or to apply Stealth Commitment. Those steps happen in SMP and GAP. GATT is permitted only as a management plane after a Bond Passkey exists (Section 15).

User-space applications on a Host MAY present UI. They MUST NOT be the sole implementation of advertising control or key storage.

### 4.2 Roles

During a given procedure a device takes one of these roles:

| Role | Typical product | Typical LE role |
| --- | --- | --- |
| **Accessory** | Speaker, earbuds, keyboard, sensor | Peripheral |
| **Host** | Phone, laptop, tablet | Central |

A device MAY implement both roles (a laptop pairing with another laptop). Requirements that say “Accessory” apply to the device that will leave Public Advertising under PeripheralOnly, which is the default.

### 4.3 No hardware redesign

An Accessory that already supports:

- LE Secure Connections,
- Limited Discoverable Mode,
- a pairing gesture (button hold, power-on window, or equivalent),
- and storage for at least two bonds,

SHALL be able to implement this specification in firmware without new buttons, screens, or indicators.

The Host is the display. Confirmations, names, Stealth Policy choices, and revocation live on the Host.

---

## 5. Bond Passkeys

### 5.1 Definition

A **Bond Passkey** is a discoverable public-key credential. It is not a value the user types, not Bluetooth Passkey Entry, and not a labeled Long Term Key.

The Host is the authenticator. The Accessory is the relying party.

| Held by | What |
| --- | --- |
| Host Vault | Private key, `credential_id`, relying-party id for this Accessory. The operating system MAY store a friendly name in a forgotten-devices or passkey list. The user SHALL NOT be shown a secret or a “use passkey?” sheet at pair or reconnect. |
| Accessory Bond Table | `credential_id`, public key, `stealth_policy`, `reconnect_mode`, `revoked` |

Algorithm: ECDSA on the NIST P-256 curve (the same class as platform passkeys, ES256). Implementations MUST NOT invent a new curve.

A Bond Passkey record SHALL contain at least:

| Field | Type | Notes |
| --- | --- | --- |
| `credential_id` | 16 octets | Identifies this credential. Synced Hosts that share a vault SHALL use the same value. |
| `public_key` | P-256 | Accessory copy. The Host MUST NOT persist the private key on the Accessory. |
| `peer_identity_address` | 6 octets + type | From SMP identity distribution (link / Directed Knock). Not the sole lookup key on reconnect. |
| `ltk` | 16 octets | From Low Energy Secure Connections (link cipher only) |
| `irk` | 16 octets | From identity distribution, if present |
| `stealth_policy` | 1 octet | Section 6 |
| `reconnect_mode` | 1 octet | Section 7.2 |
| `created_counter` | 4 octets | Monotonic on the local device |
| `revoked` | boolean | Set on revocation; public key, LTK, and IRK SHALL be erased |

`bond_passkey_id` in older drafts is `credential_id`.

An Accessory MUST NOT require a screen to store or use a Bond Passkey.

### 5.2 Bond Table

The Accessory SHALL maintain a **Bond Table** of public keys and policy. The Host SHALL maintain corresponding private keys in the Host Vault. Synced Hosts MAY share one `credential_id`.

- An Accessory MUST support at least 2 Bond Passkeys.
- An Accessory SHOULD support at least 8.
- If the table is full, a new Enrollment Window MUST fail until the user revokes an entry or performs factory reset. The Accessory MUST NOT silently evict a Bond Passkey.

Each Bond Passkey is independent. Revoking one MUST NOT revoke the others.

### 5.3 Lifetime

A Bond Passkey is created only after Stealth Commitment succeeds (Section 9.7).

A Bond Passkey ends when:

- it is revoked by the local user or by a management command from that same peer, or
- the device is factory-reset, or
- pairing is aborted after keys were tentatively stored (those keys SHALL be discarded and no Bond Passkey SHALL remain).

---

## 6. Stealth Policy

Stealth Policy is negotiated during pairing and stored on the Bond Passkey. Each device applies the policy to **its own** advertising. A peer MUST NOT remotely force a device to remain hidden if the device’s owner later sets that device discoverable by a local user action.

| Value | Code | Accessory after commitment | Host after commitment | Default use |
| --- | --- | --- | --- | --- |
| PeripheralOnly | `0x00` | No Public Advertising | Unchanged | Default. Speaker hidden, laptop shown |
| CentralOnly | `0x01` | Unchanged | No Public Advertising | Rare |
| Both | `0x02` | No Public Advertising | No Public Advertising | Shared or public Host |
| Neither | `0x03` | Unchanged | Unchanged | Setup / multi-device bench |

The initiator (normally the Host) proposes a policy. The responder MAY accept or reject. If the responder rejects, pairing of this mode SHALL fail. The devices MAY fall back to non-Stealth Bonding pairing (Section 14).

A Host that proposes `Both` or `CentralOnly` SHOULD warn that other peripherals may be harder to pair until the Host is made discoverable again.

If the pairing messages omit a policy, both sides SHALL use `PeripheralOnly`.

---

## 7. Advertising and discoverability state machine

### 7.1 States

An Accessory that supports Stealth Bonding SHALL implement the following GAP/Link Layer states. Names in this section are local to this specification.

```mermaid
stateDiagram-v2
    [*] --> Unprovisioned
    Unprovisioned --> Enrollment: pairing gesture
    Enrollment --> Unprovisioned: timeout or abort, table empty
    Enrollment --> BondedKnock: commitment, DirectedKnock
    Enrollment --> BondedSilent: commitment, SilentUntilWake
    Enrollment --> PublicBonded: commitment, policy leaves Accessory public
    BondedKnock --> Enrollment: pairing gesture or delegated open
    BondedSilent --> Enrollment: pairing gesture or delegated open
    BondedSilent --> BondedKnock: wake gesture
    BondedKnock --> BondedSilent: optional after reconnect idle
    PublicBonded --> Enrollment: pairing gesture
    BondedKnock --> Unprovisioned: last Bond Passkey gone
    BondedSilent --> Unprovisioned: last Bond Passkey gone
    PublicBonded --> Unprovisioned: last Bond Passkey gone
```

| State | Public Advertising | Connectable | Purpose |
| --- | --- | --- | --- |
| `Unprovisioned` | YES (Limited or General) | YES | Factory / empty Bond Table |
| `Enrollment` | YES, Limited Discoverable only | YES | Add a Bond Passkey |
| `BondedKnock` | NO | Directed Knock only | Hidden; bonded Hosts can connect |
| `BondedSilent` | NO | NO until wake | Hidden; user or timer must wake |
| `PublicBonded` | YES | YES | Only when policy leaves Accessory public |

### 7.2 Reconnect modes

Stored on the Bond Passkey and applied when the Accessory is not in `Enrollment` or `Unprovisioned`.

| Mode | Code | Behavior |
| --- | --- | --- |
| DirectedKnock | `0x00` | Default. Accessory transmits connectable **directed** advertising to bonded identities. |
| SilentUntilWake | `0x01` | No advertising until an existing user gesture (power, play, pairing button). Then Directed Knock for `T_SB_KNOCK`. |
| FilterAcceptList | `0x02` | Connectable advertising MAY be used only if it is not Public Advertising as defined below. Unknown initiators SHALL be ignored. |

If omitted, both sides SHALL use `DirectedKnock`.

### 7.3 Public Advertising (normative definition)

**Public Advertising** is any of the following:

1. Undirected connectable or scannable advertising (`ADV_IND`, `ADV_SCAN_IND`, or equivalent extended undirected events) that a general or limited scanner can report to a public scan list.
2. Undirected advertising that includes a Local Name, Appearance used for general discovery, or Service UUIDs intended for general discovery.
3. BR/EDR inquiry scan in a discoverable mode (Limited or General).
4. Any advertising that uses a public or static device address while the device has distributed an IRK and is not in `Enrollment` or `Unprovisioned`.

The following are **not** Public Advertising:

- Connectable directed advertising (`ADV_DIRECT_IND` or equivalent extended directed events) targeted at a bonded identity.
- No advertising at all (`BondedSilent`).
- Encrypted connected traffic after a link is up.

A nameless undirected advertisement **is** Public Advertising. LE Privacy alone does not satisfy Stealth Commitment.

### 7.4 Directed Knock rules

When in `BondedKnock` the Accessory:

1. MUST NOT perform Public Advertising.
2. MUST use connectable directed advertising addressed to a bonded peer identity, or an equivalent Controller feature that is not reportable on a public scan list.
3. MUST NOT include Local Name or general-discovery Service UUIDs (directed legacy PDUs have no AD payload; extended directed events SHALL omit those fields).
4. SHOULD rotate among bonded identities if more than one Bond Passkey exists.
5. SHOULD use resolvable private addresses when an IRK was distributed.
6. MAY duty-cycle advertising to save power.

A Host reconnects by initiating to the stored identity. The Accessory MUST accept a connection from a peer that holds a non-revoked Bond Passkey.

### 7.5 Host discoverability

When the stored Stealth Policy is `PeripheralOnly` or `Neither`, this specification places no new requirement on Host discoverability.

When the policy is `Both` or `CentralOnly`, the Host SHALL leave General Discoverable Mode and SHALL NOT perform Public Advertising for new pairing until a local user action re-enables it. Existing bonds on the Host MUST remain usable.

---

## 8. Feature discovery

A device that supports Stealth Bonding SHALL indicate support so a peer can select the mode.

Until Bluetooth SIG assigned numbers exist, implementations of this draft SHALL use:

- **AD type / Service Data** UUID: `SB_SERVICE_UUID` (Section 21)
- **SMP feature bit** in the Stealth Bonding Params PDU (Section 9.3)
- **LE Features** bit: unassigned; implementations MAY use a manufacturer-specific HCI feature flag locally

A Host that sees `SB_SERVICE_UUID` in Enrollment advertising SHOULD offer Stealth Bonding as the preferred pairing mode. The Host MUST disambiguate the scan row as specified in Section 9.11.

An Accessory in `BondedKnock` or `BondedSilent` MUST NOT include `SB_SERVICE_UUID` in any Public Advertising, because it MUST NOT perform Public Advertising.

---

## 9. Pairing procedure

Stealth Bonding is per-unit factory attestation over the Low Energy Secure Connections transcript, Host-vault registration, the LE SC link cipher, and Stealth Commitment. Just Works is not the trust root. Attestation MUST NOT run before Phase 2 public-key exchange.

```mermaid
sequenceDiagram
    participant H as Host
    participant V as HostVault
    participant A as Accessory
    Note over A: Enrollment Window open
    H->>A: Connect (undirected adv during Enrollment)
    H->>A: Pairing Request (SC, bonding)
    A->>H: Pairing Response
    H->>A: Stealth Bonding Params plus Host nonce
    A->>H: Params Response
    Note over H,A: LE SC Phase 2 (public keys, DHKey)
    A->>H: Per-unit cert plus attestation over SC transcript and nonce
    H->>H: Verify cert under Manufacturer Root, then attestation
    V->>V: Create Bond Passkey silently
    H->>A: Register credential id and public key
    Note over H,A: LE SC Phase 3 (key distribution)
    Note over A: Stealth Commitment
    A->>A: Stop Public Advertising
    A->>H: Pairing Complete (only after commitment)
```

### 9.1 Preconditions

Both devices MUST support LE Secure Connections.

The Accessory MUST be in `Unprovisioned` or `Enrollment`.

The Accessory MUST have a free Bond Table slot.

If any precondition fails, the Accessory SHALL reject Stealth Bonding. The Host MAY attempt ordinary pairing (Section 14).

### 9.2 Phase 1 — SMP pairing feature exchange

Devices SHALL exchange SMP Pairing Request / Pairing Response with:

- Secure Connections bit set
- Bonding flags set (this mode always bonds)
- MITM bit set if either side’s I/O capabilities can support an authenticated association model
- Maximum encryption key size 16

LE Legacy Pairing MUST NOT be used for Stealth Bonding. If either side would fall back to Legacy Pairing, Stealth Bonding SHALL be aborted.

I/O capability mapping is in Section 13. The user-visible association remains the existing pair button and Host device list. Numeric Comparison, typed Passkey Entry, QR, and NFC SHALL NOT be required. Just Works MAY run as the Low Energy Secure Connections association model; it MUST NOT be treated as proof that the Accessory is genuine. That proof is per-unit factory attestation over the LE SC transcript (Section 9.5).

### 9.3 Phase 1b — Params and nonce

After a successful pairing feature exchange that selected LE SC and bonding, and before Phase 2 public-key exchange:

1. The Host SHALL send `Stealth Bonding Params` including a 32-octet random **nonce**. The Host SHALL reject a repeated nonce from a prior attempt with this Accessory (replay).
2. The Accessory SHALL reply with `Stealth Bonding Params Response` only. It MUST NOT attest yet. The Low Energy Secure Connections public keys do not exist until Phase 2.

If either side does not send or accept Params, Stealth Bonding SHALL be aborted.

#### 9.3.1 `Stealth Bonding Params` (SMP command, draft opcode `0xF0`)

| Offset | Size | Field | Description |
| --- | --- | --- | --- |
| 0 | 1 | opcode | `0xF0` (draft; replace when assigned) |
| 1 | 1 | version | `0x01` for this document |
| 2 | 1 | stealth_policy | Section 6 |
| 3 | 1 | reconnect_mode | Section 7.2 |
| 4 | 1 | flags | Bit 0: `require_delegated_confirm` for later enrollments. Bit 1: `allow_peer_policy_change`. Bits 2–7: reserved, MUST be 0 |
| 5 | 1 | max_bonds | Sender’s Bond Table capacity (1–255) |
| 6 | 2 | reserved | MUST be 0 |

Nonce, attestation, and registration SHALL be sent as additional draft SMP commands (`0xF2` Attestation, `0xF3` Register). The 8-octet Params body is unchanged so existing draft traces stay valid. The normative requirement is the signed bytes, the order in this section, and the verify/register steps, not a final opcode assignment.

#### 9.3.2 `Stealth Bonding Params Response` (draft opcode `0xF1`)

Same layout. `stealth_policy` and `reconnect_mode` are the values the responder accepts. They MUST be equal to the initiator’s proposal or pairing of this mode SHALL fail.

`max_bonds` is the responder’s capacity. It is informational.

Reserved bits MUST be ignored on receipt and MUST be zero on send.

### 9.4 Phase 2 — LE Secure Connections

Devices SHALL perform Low Energy Secure Connections Phase 2 as specified in the Core Spec (public key exchange, DHKey, authentication stage 1 and 2).

This specification does not change those primitives. They encrypt the link. They do not replace factory attestation or the Bond Passkey. The two Phase 2 public keys SHALL be included in the attestation transcript (Section 9.5).

### 9.5 Phase 2b — Per-unit attestation and registration

After Phase 2 public-key exchange, and before Stealth Commitment:

1. The Accessory SHALL send its **per-unit factory certificate** and an **attestation**: a P-256 ECDSA signature by that unit’s Factory Attestation Key over the pairing transcript. The signed bytes MUST include at least: Host identity, Accessory identity, accepted policy, reconnect mode, the Host nonce, the Host’s LE SC public key, and the Accessory’s LE SC public key. An attestation that omits either public key is non-conformant.
2. The Host SHALL verify the unit certificate under a **Manufacturer Root** it already trusts for this brand (Section 9.5.1). It SHALL then verify the attestation under the unit public key from that certificate. If either check fails, or the Host has no trusted root for this Accessory, Stealth Bonding SHALL be aborted. Ordinary pairing MAY proceed instead; that MUST NOT create a Bond Passkey.
3. An audio Accessory SHOULD complete the automatic acoustic check (Section 9.9). A Host MAY apply a closeness threshold (Section 9.10).
4. The Host Vault SHALL create a Bond Passkey **silently** (no passkey sheet). If this Host already holds a `credential_id` for this Accessory’s relying-party id (synced vault), it SHALL reuse that credential and MUST NOT require a new Enrollment Window on a later reconnect.
5. The Host SHALL send the `credential_id` and public key to the Accessory. The Accessory SHALL store them tentatively until Stealth Commitment.

If either side does not send or accept attestation or registration, Stealth Bonding SHALL be aborted.

#### 9.5.1 Per-unit factory certificate

Each Accessory unit MUST have its own Factory Attestation Key and a certificate that binds that unit’s public key to a unique serial. The Host MUST trust a Manufacturer Root (or a short chain ending at one) and MUST NOT treat a single shared model key as sufficient.

A stolen unit key enrolls only that unit’s identity. A stolen Manufacturer Root is a brand-wide incident; products MUST be able to revoke or rotate roots on the Host.

The certificate serial is also the source of the Enrollment scan-row suffix (Section 9.11).

### 9.6 Phase 3 — key distribution

Devices SHALL distribute at least:

- LTK (via the SC procedure)
- Identity Address
- IRK, if the device will use resolvable private addresses (RECOMMENDED for Accessories)

Keys are held tentatively until Stealth Commitment succeeds.

### 9.7 Phase 4 — Stealth Commitment

After Phase 3 succeeds, each device SHALL apply the negotiated Stealth Policy to its own radio.

For an Accessory when policy is `PeripheralOnly` or `Both`:

1. The Host stack SHALL command the Controller to leave Public Advertising.
2. The Accessory SHALL enter `BondedKnock` or `BondedSilent` according to `reconnect_mode`.
3. The Accessory SHALL persist the Bond Passkey only after the Controller confirms the new advertising state.
4. Only then MAY the Accessory signal pairing success to the user (LED pattern, and SMP/HCI completion to the Host).

If the Controller cannot leave Public Advertising, the Accessory SHALL:

- discard tentative keys,
- send pairing failure,
- remain in `Enrollment` or return to `Unprovisioned`,
- MUST NOT leave a Bond Passkey in the table.

A Host MUST NOT present “Connected / Paired” as a Stealth Bonding success until it has received pairing completion after commitment. If the Accessory drops the procedure, the Host SHALL delete tentative keys.

### 9.8 Abort

On any failure after tentative key storage, both sides SHALL erase those keys. Partial Bond Passkeys MUST NOT remain.

### 9.9 Acoustic check (audio products)

An Accessory that can play sound SHOULD, during first enrollment, emit a short tone or ultrasonic sequence derived from the same transcript the attestation covers (at least the Host nonce and both LE SC public keys). The Host SHOULD verify that sequence on its microphone. The check MUST be automatic. It MUST NOT add a sheet, code, or extra tap.

If the Host implements this check and verification fails, Stealth Bonding SHALL be aborted. Products without a speaker are not required to implement it. Success proves the radio peer is the object making sound in the room; it does not identify which of two same-model speakers the user meant if both are playing.

### 9.10 Closeness threshold

A Host MAY refuse Stealth Bonding during an Enrollment Window if the measured signal strength is below a product-defined threshold. This is a convenience filter. It is not authentication. A same-room attacker still satisfies it.

### 9.11 Enrollment scan row

The Bluetooth Local Name MAY be a product name shared by many units. Anyone can advertise any Local Name; the radio has no uniqueness authority. This specification therefore MUST NOT require globally unique advertised names.

A Host MUST NOT present two Enrollment advertisements as the same row. The Host SHALL render the ordinary product name plus a short unique suffix derived from the per-unit certificate serial (for example the last four hexadecimal digits). The Accessory SHOULD print that same suffix on the unit or packaging.

Unique rows remove identical twins from the list. They do not bind the user to the object in their hand. A lookalike suffix, a suffix the user ignores, or a different tempting name still works. See Section 16.

---

## 10. Enrollment

### 10.1 Enrollment Window

An **Enrollment Window** is Limited Discoverable Mode for at most `T_SB_ENROLL` (default 30 s, maximum 180 s, aligned with GAP limited-discoverable practice).

During the window the Accessory MAY use Public Advertising, including Local Name, so the Host can select it.

When the window ends with no new Bond Passkey:

- if the Bond Table is empty, return to `Unprovisioned`;
- otherwise return to `BondedKnock` or `BondedSilent`.

Only one Enrollment Window SHALL be open at a time.

### 10.2 First enrollment

An Unprovisioned Accessory SHOULD be in Limited Discoverable Mode after power-on or the pairing gesture, for one Enrollment Window.

Procedure: Section 9. The Host is the screen. The Accessory remains `NoInputNoOutput` if it has no keys or display.

### 10.3 Subsequent enrollment (local gesture)

To add another Bond Passkey, the user performs the **same pairing gesture the product already ships with** (typically hold the Bluetooth button until the LED blinks).

The Accessory SHALL open an Enrollment Window. After success, Stealth Commitment runs again. Other Bond Passkeys remain valid.

### 10.4 Delegated Enrollment

A Host that holds a non-revoked Bond Passkey MAY request that the Accessory open an Enrollment Window without a local gesture. This is how a second phone is added when the Accessory is across the room and the user does not want a new button.

1. Host connects via Directed Knock (or an existing connection).
2. Link MUST be encrypted and the Host MUST have just completed a valid Assertion for a non-revoked Bond Passkey.
3. Host writes `Open Enrollment` (Section 15).
4. Accessory opens an Enrollment Window.
5. If `require_delegated_confirm` is set on the Accessory, the authorizing Host MUST confirm the new Bond Passkey before Stealth Commitment on the new pairing. If confirmation is not received within `T_SB_ENROLL`, the new pairing SHALL be aborted and keys discarded.

Delegated Enrollment MUST NOT work from an unbonded peer.

### 10.5 Full table

If no slot is free, the Accessory SHALL refuse new pairing and SHOULD indicate the existing error pattern (LED). The Host SHOULD show that a Bond Passkey must be revoked first.

---

## 11. Reconnection

After Stealth Bonding, a bonded Host reconnects without Public Advertising.

1. Host uses the stored identity / IRK to initiate (Directed Knock).
2. Accessory, if in `BondedKnock`, is performing Directed Knock.
3. Accessory, if in `BondedSilent`, is not connectable until a wake gesture; then it enters `BondedKnock` for `T_SB_KNOCK`.
4. The Accessory SHALL send a fresh 32-octet **challenge**. It MUST reject a repeated challenge (replay).
5. The Host SHALL **assert**: sign the challenge (and Accessory identity) with the Bond Passkey private key, silently. No passkey sheet.
6. The Accessory SHALL verify the assertion against a non-revoked stored public key. Lookup is by public key / `credential_id`, not only by Host address. A synced Host with the same `credential_id` SHALL succeed without a new Enrollment Window. A stranger Host SHALL fail (`NoBond`) unless an Enrollment Window is open.
7. On success, encryption SHALL be started with the stored LTK. If assertion or encryption fails, the Accessory SHALL disconnect and MUST NOT open Public Advertising.

The Accessory MUST accept reconnection from any non-revoked Bond Passkey, not only the most recent one.

---

## 12. Revocation, policy change, and reset

### 12.1 Revoke one Bond Passkey

The owner MAY revoke a Bond Passkey from:

- the Host OS UI (forget device), which SHOULD also notify the Accessory if connected, or
- the Management Service `Revoke` command, or
- Accessory factory reset (all entries).

On revoke, the public key, LTK, IRK, and `credential_id` SHALL be erased from the Accessory. The Host SHOULD delete the vault private key. Leftover keys become useless.

### 12.2 Policy change

A device MAY change **its own** discoverability by a local user action at any time. That local action is outside this pairing mode.

Changing the stored Stealth Policy on a Bond Passkey MUST be a local user action or, if `allow_peer_policy_change` was set, a management write from that peer over an encrypted link. The peer still cannot prevent a later local override.

### 12.3 Factory reset

The product’s existing reset gesture SHALL erase the entire Bond Table and return the Accessory to `Unprovisioned`. This specification MUST NOT require a new reset control.

---

## 13. I/O capabilities and screenless devices

Stealth Bonding MUST use the existing SMP I/O capability exchange for the Low Energy Secure Connections *link*. User-visible pairing SHALL be only:

1. The Accessory’s existing pairing gesture (Enrollment Window).
2. The Host Settings → Bluetooth (or equivalent) row for the Accessory.

The Host MUST NOT show a WebAuthn sheet, account picker, “Use passkey?” button, QR, number compare, or biometric-for-this-passkey prompt during pair or reconnect. If a platform vault API cannot create or assert a credential without such a sheet, that platform is non-conformant for this mode until it grows a silent Bluetooth-pairing ceremony.

| Accessory hardware | IO Cap | Link association | User action |
| --- | --- | --- | --- |
| No keys, no display | NoInputNoOutput | Just Works for LE SC only | Hold pairing button; tap the device in the Host list |
| Yes/No only | DisplayYesNo | Numeric comparison MAY run if the product already has it | Not required by this mode |
| Keyboard, no display | KeyboardOnly | Passkey Entry MAY run if the product already has keys | Not required by this mode |

This specification MUST NOT require Passkey Entry, numeric comparison, QR codes, or biometrics on the Accessory.

The Host SHOULD show, in ordinary Bluetooth or forgotten-device UI (not a passkey ceremony):

- the Enrollment scan row (product name plus per-unit suffix, Section 9.11),
- a named bond (“Kitchen speaker”) after success,
- the Stealth Policy (default PeripheralOnly, not a required extra step),
- later, a list of Bond Passkeys for revocation.

The Host MUST NOT collapse two Enrollment advertisements that share a Local Name into one row.

---

## 14. Fallback and coexistence

If the peer does not support Stealth Bonding, devices SHALL fall back to the pairing methods they already implement. This specification does not forbid vendor hide-after-pair in that case, but that behavior MUST NOT be reported as Stealth Bonding.

A Host SHOULD prefer Stealth Bonding when the Accessory indicates support and the user has not disabled it.

Stealth Bonding MAY coexist with Fast Pair, Swift Pair, or similar. Those procedures MUST NOT reopen Public Advertising after Stealth Commitment except by opening a specified Enrollment Window.

BR/EDR dual-mode Accessories that complete Stealth Bonding with policy `PeripheralOnly` or `Both` SHALL leave BR/EDR discoverable modes as well as LE Public Advertising.

---

## 15. Stealth Bonding Management Service

This GATT service is OPTIONAL. If implemented, it is available only on an encrypted link whose keys belong to a non-revoked Bond Passkey.

Service UUID: `SB_SERVICE_UUID` (Section 21).

| Characteristic | Access | Purpose |
| --- | --- | --- |
| Bond Table List | Read | `credential_id` values and optional names; MUST NOT return private keys, LTK, or IRK |
| Revoke | Write | Erase one `bond_passkey_id` (self, or any if the writer is authorized by implementation policy) |
| Open Enrollment | Write | Delegated Enrollment (Section 10.4) |
| Local Policy | Read / Write | Local reconnect mode; write allowed if `allow_peer_policy_change` or writer is local Host UI |

Writes from an unencrypted link MUST be rejected.

This service MUST NOT start pairing and MUST NOT be the mechanism that applies Stealth Commitment.

---

## 16. Security considerations

**Just Works is not the trust root.** Low Energy Secure Connections association MAY be Just Works. A Bond Passkey MUST NOT be created unless a per-unit factory certificate verifies under a Manufacturer Root and the unit attestation covers the Low Energy Secure Connections transcript plus the Host nonce. A Host that cannot verify attestation MUST refuse Stealth Bonding.

**Per-unit certificate residual.** Attestation proves “this unit’s key, certified by a brand the Host trusts,” not “the object the user meant to tap.” A stolen unit key enrolls that unit. A stolen Manufacturer Root is brand-wide.

**Same-room evil twin.** Two genuine units, or a genuine unit and a lookalike advertisement, can sit in the same room during an Enrollment Window. Unique scan-row suffixes (Section 9.11) mean those rows cannot be identical. Acoustic check (Section 9.9) and closeness (Section 9.10) can reject a silent or distant peer. A same-room evil twin remains residual unless the user notices which row they tapped and, if the suffix is printed on the unit, checks it. Forbidding two devices from advertising the same Local Name does not close this: the radio cannot enforce name uniqueness, and an attacker can use a lookalike or a different tempting name.

**Transcript coverage.** An attestation that is signed before Phase 2, or that omits either LE SC public key, does not bind the factory key to the link being formed. That order is non-conformant.

**Freshness, not one-time passwords.** Nonces and reconnect challenges MUST be random (or a monotonic counter included in the signed bytes). Implementations MUST NOT use time-based one-time passwords or short rotating digits as the recognizer.

**Replay.** A Host MUST reject a repeated attestation nonce. An Accessory MUST reject a repeated assertion challenge.

**Silent vault.** Create and assert happen inside the Bluetooth stack. A visible passkey ceremony is non-conformant. Unlocking the phone to open Settings is ordinary device use.

**Enrollment Window is still sensitive.** Products SHOULD require a physical gesture for the first Bond Passkey from a stranger. A Host that already holds a synced `credential_id` does not need a new window. Delegated Enrollment MUST require a live Assertion.

**Commitment failure.** Reporting success while still publicly advertising would be a false security property. Section 9.7 forbids that.

**Key storage.** The Host Vault private key MUST be stored with the same protection the platform uses for other passkeys. The Accessory public key is not secret; the LTK still is. Home-grown *link* math is not conformant.

**Revocation.** Forgetting the device on one Host does not erase other `credential_id` values. Factory reset clears the Bond Table.

**Management service.** Any GATT write that opens enrollment or revokes a bond is a privileged operation. It MUST require an encrypted link after a valid Assertion.

**Sniffers.** Directed Knock packets can be observed on the air. This mode hides the Accessory from public scan lists, not from a radio that is already next to it.

---

## 17. Privacy considerations

After Stealth Commitment, the Accessory MUST NOT be identifiable from a public scan list.

Directed Knock SHOULD use resolvable private addresses so a passive scanner cannot join Directed Knock events to a stable public address.

A Host UI MUST NOT publish the Accessory’s identity address in an untrusted log as a condition of this mode.

Bond Table List (Section 15) returns identifiers for management. It MUST NOT return raw keys.

---

## 18. Compatibility

| Device class | How it gets Stealth Bonding | Reality |
| --- | --- | --- |
| Phones, computers | OS / Bluetooth stack update | Common path |
| Accessories with OTA | Manufacturer firmware | Hit-or-miss |
| Fixed-firmware Accessories | Ship with it, or never | Most cheap devices |
| Arbitrary existing radio | Not possible | Hardware and stack must already support the controls |

This is the same rollout path as other Core Spec features.

---

## 19. Conformance

A device claiming Stealth Bonding support SHALL:

1. Verify Host Assertions with stored public keys. Reject replayed challenges.
2. Attest first enrollment with a per-unit Factory Attestation Key over the Low Energy Secure Connections transcript plus Host nonce. Present a unit certificate that verifies under a Manufacturer Root.
3. Implement Low Energy Secure Connections as the link cipher for this mode.
4. Implement the advertising states in Section 7.
5. Apply Stealth Commitment before reporting success (Section 9.7).
6. Refuse to call a bond Stealth Bonding if Public Advertising is still active when policy requires it off.
7. Support at least two Bond Passkeys on Accessories.
8. Open Enrollment only via the existing pairing gesture or Delegated Enrollment (synced vault Hosts excepted, Section 11).
9. Fall back cleanly when the peer lacks support.

A Host claiming support SHALL:

1. Trust a Manufacturer Root; verify the per-unit certificate and the transcript-bound attestation, or refuse Stealth Bonding.
2. Create and assert Bond Passkeys silently in the Host Vault.
3. Prefer this mode when the Accessory indicates it.
4. Provide ordinary Bluetooth UI for success, failure, names, and revocation — not a passkey sheet. Disambiguate Enrollment rows (Section 9.11).
5. Implement Directed Knock initiation using stored identity.
6. Audio Hosts SHOULD run the automatic acoustic check (Section 9.9). Hosts MAY apply a closeness threshold (Section 9.10).

---

## 20. Timers and constants

| Name | Default | Maximum | Meaning |
| --- | --- | --- | --- |
| `T_SB_ENROLL` | 30 s | 180 s | Enrollment Window |
| `T_SB_KNOCK` | 30 s | 120 s | Directed Knock after a wake gesture in SilentUntilWake |
| `N_SB_BONDS_MIN` | 2 | — | Minimum Accessory Bond Table size |
| `N_SB_BONDS_REC` | 8 | — | Recommended Accessory Bond Table size |

---

## 21. Assigned numbers (draft)

These values are provisional for independent implementations of this draft. They MUST be replaced if the Bluetooth SIG assigns official numbers.

| Name | Draft value | Purpose |
| --- | --- | --- |
| `SB_SERVICE_UUID` | `8b2e5c10-4f3a-4a91-9c6d-1e7a0b2d4f50` | Feature discovery and Management Service (random 128-bit; not a SIG 16-bit alias) |
| SMP opcode Params | `0xF0` | Vendor/draft command space |
| SMP opcode Params Response | `0xF1` | Vendor/draft command space |
| SMP opcode Attestation | `0xF2` | Per-unit cert plus signature over LE SC transcript + nonce |
| SMP opcode Register | `0xF3` | Host `credential_id` + public key |
| Company identifier | Unassigned | Not required for this draft |

Independent implementations SHOULD be prepared to configure UUIDs and opcodes.

---

## 22. References

- Bluetooth Core Specification, GAP [Vol 3] Part C; SMP [Vol 3] Part H; Link Layer [Vol 6] Part B
- RFC 2119, RFC 8174
- FIDO U2F / CTAP Bluetooth transport (informative prior art)
- Google Fast Pair specification (informative prior art)
- W3C WebAuthn (informative; not used on the air)

---

## Appendix A. Worked example (informative)

**Pair a speaker to a laptop, speaker hidden, laptop shown.**

1. User holds the speaker’s existing Bluetooth button. LED blinks. Enrollment Window opens.
2. Laptop shows “Kitchen Speaker · 7C21” (product name plus the per-unit suffix printed on the speaker).
3. User selects that row. Both sides support Stealth Bonding. Policy defaults to PeripheralOnly, reconnect DirectedKnock.
4. Laptop sends a fresh nonce. Both sides run Low Energy Secure Connections Phase 2. The speaker presents its unit certificate and attests over the SC transcript plus nonce. The laptop verifies the cert under the manufacturer root, optionally hears the automatic acoustic check, creates a Bond Passkey in the vault silently, and registers the public key.
5. Keys are stored tentatively. Speaker stops all undirected advertising and starts Directed Knock to the laptop. Commitment succeeds.
6. Laptop shows paired. Another phone’s Bluetooth scan shows nothing — no name, no anonymous blip.
7. Laptop reconnects later via Directed Knock. A sniffer next to the speaker may still see directed packets.

**Add a phone.**

8. User holds the speaker button again, or taps “Add device” on the laptop (Delegated Enrollment).
9. Phone pairs during the window. A second Bond Passkey is stored. Speaker returns to Directed Knock for both peers.

**Forget the laptop.**

10. User forgets the speaker on the laptop. That Bond Passkey is revoked. The phone’s Bond Passkey still works. The speaker stays off the public scan list.

---

## Appendix B. Comparison (informative)

| | LE Privacy | Vendor hide-after-pair | Fast Pair | FIDO BLE | Stealth Bonding |
| --- | --- | --- | --- | --- | --- |
| Standardized pairing mode | No (privacy feature) | No | Google profile | Authenticator transport | This document |
| Public scan blip after bond | Often yes | Sometimes no | Account-key filter ads | Pairing-mode bit | No |
| Per-peer named credential | Bond keys | Bond keys | Account keys | Credentials | Host-vault Bond Passkey |
| Screenless first pair | Yes | Yes | Yes | Pairing mode | Yes |
| Protocol-enforced commitment | No | No | No | No | Yes |
| Installable on any device | N/A | Firmware | Firmware + Google | Authenticator | Firmware / OS only |
