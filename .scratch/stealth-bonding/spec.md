# Stealth Bonding

Status: ready-for-agent

Canonical protocol text: [`docs/spec/stealth-bonding.md`](../../docs/spec/stealth-bonding.md)

This file is the project spec for building against that protocol (reference implementation, tests, docs). It synthesizes the design thread. It does not replace the protocol document.

## Problem Statement

A user can pair a Bluetooth Accessory (speaker, buds, keyboard, IoT gadget) to a Host (laptop or phone), but after that the Accessory often still appears on other people’s scan lists — sometimes as a name, sometimes as a nameless blip. Vendor “hide after pair” is inconsistent. LE Privacy still advertises. There is no interoperable pairing mode that creates a named per-peer credential, forces the Accessory off Public Advertising, still allows several trusted Hosts, and works on screenless products that will not get new buttons.

The user cannot install an app to fix this on arbitrary radios. The behavior has to live in the Bluetooth stack.

## Solution

Treat Stealth Bonding as a Core Spec pairing-mode extension (SMP + GAP), not a GATT profile and not an app.

Devices that both support it negotiate the mode, run Low Energy Secure Connections, verify a per-unit factory certificate under a Manufacturer Root and an attestation over that SC transcript, create a Bond Passkey in the Host vault (Accessory stores the public key), then apply Stealth Commitment. Default Stealth Policy is PeripheralOnly. The user does not see a passkey sheet. A synced Host reuses `credential_id` without a new Enrollment Window. Strangers enroll with the pairing button or Delegated Enrollment. Reconnect is an Assertion over a fresh challenge, then Directed Knock. If commitment cannot stop Public Advertising, pairing fails and keys are discarded.

This repo’s job is the open specification plus, later, a reference implementation and conformance tests. Rollout to real products is firmware and OS updates.

## User Stories

1. As a laptop user, I want to pair a speaker with Stealth Bonding, so that other phones no longer see that speaker in Bluetooth settings.
2. As a laptop user, I want my laptop to stay visible after that pair, so that I can still pair a mouse or headphones.
3. As a laptop user, I want the speaker gone from the scan list entirely, so that there is not even a nameless blip.
4. As a speaker owner, I want pairing to be “hold the button I already have,” so that I do not need a new product.
5. As a speaker owner, I want the laptop to be the screen, so that a button-only speaker can still complete pairing.
6. As a speaker owner, I want to add a phone later with the same button hold, so that two Hosts can both be trusted.
7. As a speaker owner, I want to add a tablet from the already-paired laptop, so that I do not have to walk over and hold the speaker button.
8. As a speaker owner, I want a friend’s phone to get its own Bond Passkey, so that forgetting the laptop does not kick that friend off. My second laptop on the same vault should just work.
9. As a speaker owner, I want factory reset to wipe every Bond Passkey, so that a sold or lost speaker is not still trusted.
10. As a speaker owner, I want a full Bond Table to refuse a new pair instead of silently dropping an old one, so that I do not lose a device I still use.
11. As a Host user, I want to name a Bond Passkey (“Kitchen speaker → Jacob’s laptop”), so that I can revoke the right one later.
12. As a Host user, I want to forget the speaker in OS settings and have that Bond Passkey revoked, so that trust matches what the UI says.
13. As a user of a shared laptop, I want to choose Both as Stealth Policy, so that the laptop also stops being publicly discoverable.
14. As a bench technician, I want Neither as Stealth Policy, so that I can keep seeing the Accessory while I set up several Hosts.
15. As a privacy-conscious user, I want CentralOnly available, so that a Host can hide while an Accessory stays joinable.
16. As a Host user, I want PeripheralOnly to be the default, so that ordinary speaker pairing does the safe obvious thing.
17. As a Host user, I want a warning when I pick Both, so that I am not surprised I can no longer pair a mouse.
18. As an Accessory owner, I want Directed Knock so the laptop reconnects without putting the speaker back on the public list.
19. As a battery-device owner, I want SilentUntilWake, so that the radio stays quiet until I press power or play.
20. As a Host user, I want reconnection to work for every Bond Passkey, not only the last one, so that the phone still connects after I used the laptop.
21. As a Host user, I want encryption failure on reconnect to drop the link without opening Public Advertising, so that a bad key cannot un-hide the speaker.
22. As a user pairing an old speaker, I want fallback to ordinary Bluetooth pairing, so that the laptop still works when Stealth Bonding is missing.
23. As a user pairing a new speaker to an old phone, I want the same fallback, so that one side lacking support is not a brick.
24. As a Host user, I want the OS to prefer Stealth Bonding when both sides support it, so that I do not have to know the protocol name.
25. As an Accessory implementer, I want LE Legacy Pairing forbidden in this mode, so that we do not ship a weak path under a strong name.
26. As an Accessory implementer, I want LE Secure Connections reused, so that I do not invent pairing math.
27. As an Accessory implementer, I want Stealth Commitment to fail the pair if advertising cannot be stopped, so that I never report a hidden device that is still public.
28. As an Accessory implementer, I want draft UUIDs and SMP opcodes that can be swapped for SIG numbers, so that a prototype can ship before assignment.
29. As a chip firmware engineer, I want the feature in SMP and GAP, so that a GATT app cannot pretend to own bonding.
30. As a Host OS engineer, I want a GATT management service only after a bond exists, so that list/revoke/open-enrollment have a standard surface.
31. As a Host OS engineer, I want Bond Table List to omit LTK and IRK, so that a debug screen cannot leak keys.
32. As a security reviewer, I want per-unit factory attestation over the Low Energy Secure Connections transcript plus a fresh nonce, so that Just Works is not the trust root and the factory key is bound to this link.
33. As a security reviewer, I want Delegated Enrollment to require an existing encrypted Bond Passkey, so that strangers cannot open pairing.
34. As a security reviewer, I want an optional confirm-on-authorizing-Host for additional enrollments, so that a stolen session cannot quietly add a peer.
35. As a product designer, I want no new buttons, screens, or LEDs required, so that existing SKUs can take a firmware update.
36. As a product designer, I want the existing LED blink to mean Enrollment Window, so that UX does not change.
37. As a dual-mode Accessory implementer, I want BR/EDR inquiry scan turned off with LE Public Advertising, so that Classic discovery does not undo stealth.
38. As a Fast Pair implementer, I want coexistence rules that forbid reopening Public Advertising except via an Enrollment Window, so that helper profiles cannot un-hide the device.
39. As a conformance tester, I want a public-scan-list test after commitment, so that a nameless blip is a fail.
40. As a conformance tester, I want a test that a second Host can enroll and reconnect, so that multi-Bond-Passkey is real.
41. As a conformance tester, I want a test that commitment failure discards keys, so that the false-success case cannot ship.
42. As an open-source maintainer, I want the protocol spec to be the source of truth, so that implementations do not fork behavior in issues.
43. As a SIG reader, I want RFC 2119 language and layering that matches Core Spec volumes, so that this can be offered as an extension draft.
44. As a user, I want to understand I cannot download this onto a random cheap speaker, so that expectations match firmware/OS rollout.
45. As a laptop user walking into a cafe, I want already-bonded speakers around me to stay off my scan list, so that I am not looking at other people’s accessories.
46. As an Accessory owner, I want to revoke one Bond Passkey from the laptop UI while the speaker is connected, so that I do not need the speaker’s reset.
47. As an Accessory owner, I want local discoverability override on the Host, so that a protocol peer cannot trap my laptop in hidden mode.
48. As a reference-implementation developer, I want a single test seam at the pairing-plus-advertising state machine, so that tests do not poke Controller internals.
49. As a Host user, I want two Enrollment ads with the same product name to show different rows (name plus unit suffix), so that I am not looking at identical twins.
50. As a speaker owner, I want the phone to hear an automatic tone from the speaker during pair, so that I do not get a new sheet or code.
51. As a Host implementer, I want an optional closeness threshold during the Enrollment Window, so that a distant advertisement can be ignored without calling that authentication.

## Implementation Decisions

- The protocol document in `docs/spec/stealth-bonding.md` is normative. This file is the build plan for the repo.
- Language is the `CONTEXT.md` glossary. Do not say “passkey” without “Bond,” and do not call Public Advertising a “blip” in normative text.
- Module to build later: a Stealth Bonding state machine that owns Enrollment, pairing params, Bond Table, and advertising-state commands. That is the only external seam.
- The state machine’s interface is: start enrollment, accept/reject a peer, apply commitment, reconnect, revoke, reset. Callers do not set raw advertising PDUs.
- Link crypto is LE Secure Connections. The Bond Passkey is a Host-vault P-256 credential (ADR 0006). This repo does not invent a new link cipher or use TOTP.
- Factory attestation and reconnect assertions are real signatures in the simulator, not “always ok” stubs. Attestation covers stubbed LE SC public keys. Each Accessory fixture has its own unit key and a cert under a manufacturer root.
- `0xF2` (35-octet nonce request / 204-octet attestation) and `0xF3` (51-octet register) are encoded and decoded on the pairing seam. The Long Term Key remains a fixture.
- Default Stealth Policy is PeripheralOnly. Default reconnect mode is DirectedKnock.
- Accessories expose at least two Bond Table slots; eight is recommended; no silent eviction.
- GATT Management Service is post-bond only and optional for a first reference implementation.
- Draft UUID `8b2e5c10-4f3a-4a91-9c6d-1e7a0b2d4f50` and SMP opcodes `0xF0`–`0xF3` are placeholders.
- First reference implementation: a Rust PDU simulator in this repo (`sim/`) that speaks the state machine and records PDUs. The file-vault test Host is `host/` (`sb-host` / `sb-accessory`). Firmware C on a real controller is a later adapter behind the same interface.
- Rollout path is spec → simulator → firmware/OS. Not an installable app.

## Testing Decisions

A good test asserts externally visible protocol behavior: advertising state, pairing success or failure, Bond Table contents as identifiers (not raw keys), and scan-list visibility. Tests do not assert internal function names, HCI command ordering, or log strings.

The module under test is the Stealth Bonding state machine. Prior art in this repo: none yet. First tests should be:

- Enrollment then PeripheralOnly commitment → no Public Advertising.
- Nameless undirected advertising after commitment → fail (LE Privacy is not enough).
- Second Bond Passkey via a second Enrollment Window → both peers reconnect.
- Commitment failure → no Bond Passkey stored.
- Unsupported peer → fallback, no Stealth Bonding claim.
- Delegated Enrollment from unbonded peer → rejected.
- Full Bond Table → new pair refused.
- Factory reset → Unprovisioned and Public Advertising allowed again.
- Registration without factory attestation → fail; no Bond Passkey.
- Attestation that omits the LE SC transcript → fail.
- Forged per-unit certificate → fail.
- Two Accessories with the same Local Name → different Enrollment scan rows.
- Far Accessory under a closeness threshold → fail.
- Silent audio Accessory when the Host requires acoustic → fail.
- Replayed attestation nonce → fail.
- Assertion reconnect → success.
- Replayed assertion challenge → fail; still not public.
- Synced Host (same `credential_id`) reconnects without a new Enrollment Window.
- Stranger Host → no bond; needs enrollment.

Golden PDU traces and a “would this appear on a public scan list?” oracle are the preferred fixtures.

## Out of Scope

- Filing or pursuing patents.
- A consumer app that claims to enable this on any Bluetooth device.
- New Accessory hardware.
- WebAuthn/FIDO login, except as cited prior art.
- Making Directed Knock invisible to a sniffer.
- Bluetooth SIG submission paperwork (the spec is written so it can be submitted later).
- Production firmware for a specific chip vendor in this first spec drop.

## Further Notes

Seam check (from `/to-spec`): there is no existing codebase seam. The highest seam to add is the pairing-plus-advertising state machine. GATT, HCI, and UI are adapters behind it.

Related ADRs: `0001` protocol extension, `0002` reuse LE SC for the link, `0003` no Public Advertising after bond, `0006` Host-vault Bond Passkey.

What a PDU-level simulator is, and the implementation order: [`docs/reference-implementation.md`](../../docs/reference-implementation.md).

What can be sold around an open spec (not in the protocol text): [`docs/commercial.md`](../../docs/commercial.md).
