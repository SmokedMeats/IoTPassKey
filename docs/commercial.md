# Commercial layer

The protocol is open. Money, if any, sits next to it — not inside the pairing mode.

Who is likely to adopt, whether the Bond Passkey does any work once hide-after-pair already exists, and the development order: [`adoption-path.md`](adoption-path.md).

This is not normative. It does not change [`stealth-bonding.md`](spec/stealth-bonding.md).

## What does not sell

The pairing procedure itself, a download that “adds this to any Bluetooth device,” and exclusive ownership of a SIG-style mode. Hosts and Accessories only hide interoperably if Apple, Google, Microsoft, and chip vendors can implement the spec. A closed radio protocol stays a vendor feature.

## What you can sell

| Thing | Who pays | Fits this idea? |
| --- | --- | --- |
| First hardware / modules that already do it | Device makers, pro AV, privacy-focused brands | Yes |
| Firmware / SDK for chips that exist today | Nordic, speaker OEMs, IoT vendors | Yes |
| Test tools / “certified stealth bond” badge | OEMs who want a logo | Yes |
| Consulting to get stacks and products compliant | Same OEMs | Yes |
| Patents + licensing | Chip/OS vendors | Possible, slow, often fights SIG adoption |

Open the spec and a reference implementation. A company, if you want one, sells modules, firmware, certification, and support. That is the passkeys pattern: WebAuthn/FIDO is open; vendors sell authenticators and OS support.

Patents are listed for completeness. This project is not pursuing them. A public spec is a defensive publication of the high-level mode. See the protocol document’s non-goals.

## Rollout (who ships the feature)

| Device class | Can it get Stealth Bonding? | How |
| --- | --- | --- |
| Phones / computers | Yes (most modern ones) | OS or Bluetooth stack update |
| Headphones / speakers with OTA | Sometimes | Manufacturer firmware |
| Cheap / fixed-firmware Accessories | Usually no | Must ship with it |
| Arbitrary existing radio | No | Not an app install |

## Recommended split

- **Open:** spec, pairing flows, Stealth Policy, reference state machine, test vectors
- **Commercial (optional):** branded SDK, certification mark, modules, paid compliance help
