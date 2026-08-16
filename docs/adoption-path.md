# Adoption path and demand

Who is likely to ship Stealth Bonding **after** the Bond Passkey became a real (invisible) Host-vault credential, how we can test that, and what the Development Kit can and cannot prove. This is not normative. The spec still wins.

Shipment backdrop: Bluetooth Special Interest Group / ABI Research — about **5.4 billion** Bluetooth devices in 2025, on a path past **8 billion** by 2030. [Bluetooth market dashboard](https://www.bluetooth.com/blog/now-available-the-new-bluetooth-market-dashboard/). Category counts are order-of-magnitude.

---

## Two features, two proofs

Do not mix these. They ship on different timelines.

| Piece | Job | Who must implement it | What a normal phone pair proves today |
| --- | --- | --- | --- |
| **Stealth Commitment** | Leave Public Advertising or fail the pair | Accessory firmware + a little Host policy | **Yes** — Development Kit + laptop + second phone scan list |
| **Bond Passkey** | Host-vault public-key credential. Per-unit factory cert under a Manufacturer Root. Attestation covers the Low Energy Secure Connections transcript. Fresh challenge + signature at reconnect. User does not see a passkey sheet. | Host operating-system vault (or a test stand-in) **and** the Accessory | **No.** Pairing from Settings is still Just Works. That is not a passkey test. |

The old line “the Bond Passkey is only bookkeeping” is **false** under the corrected model. Attestation and assertion are real cryptography. The hide is still the only thing a stranger notices.

Low Energy Privacy already rotates addresses. That is not stealth. Fast Pair still wants account-data advertising after pair — the opposite of hide, and a fight with a silent platform vault. [Fast Pair certification](https://developers.google.com/nearby/fast-pair/fast-pair-certification-guideline).

---

## How we test (what is actually possible)

There is no iPhone or Android silent-vault ceremony to point at. Test in layers.

| Layer | Setup | Proves | Does not prove |
| --- | --- | --- | --- |
| **0. Simulator** | In-process Host + Accessory in `sim/`. Real P-256 sign/verify fixtures (manufacturer root, per-unit cert, vault key). Golden `0xF0`–`0xF3` bytes. | Attestation required and must cover stubbed SC public keys; forged unit cert fails; replayed nonce fails; assertion reconnect; synced Host (same credential id) skips Enrollment Window; stranger cannot; same Local Name still yields unique scan rows; scan-list / commitment tests still pass. | Radio, a real operating-system vault, a real microphone acoustic check, real LE SC link math. |
| **1. File-vault test Host** | Rust crate `host/`. Key pair in `SBVAULT1` on disk. No WebAuthn dialog. Talks TCP to `sb-accessory` (simulator Accessory). | PDUs on a link we control. Host rejects an untrusted manufacturer root. Stranger vault cannot reconnect. | iCloud / Google / Windows silent pair. Radio. |
| **2. Development Kit hide** | Current Zephyr app + a normal laptop. See [`nrf52840-dk-setup.md`](nrf52840-dk-setup.md). | Scan list empties. Filmable. Label it “Generic Access Profile hide only.” | The Bond Passkey. |
| **3. Kit + test Host** | nRF Accessory with a **test** per-unit cert under a test manufacturer root; Linux file vault. Optional second kit as central. | First on-air attestation + assertion. | Platform vault sync. |
| **4. BlueZ** | Host stack once PDUs are stable. | A real Host shape. | Consumer phone vault. |
| **5. Phone OS** | Only when a vendor implements silent Bluetooth-pairing vault create/assert. | The product as imagined. | — |

**Do not** pair the kit from Windows or iPhone Settings and call that a passkey test.

**Do not** use Authy, a website passkey prompt, or typed one-time digits. Those are the wrong protocol.

---

## Will there be demand?

**Not consumer demand for a pairing mode.** Nobody asks for “passkeys for Bluetooth.” They notice leftover discoverability. Good brands already hide.

**Hide demand** (unchanged, narrow): shared rooms, leftover scan-list embarrassment.

**Passkey demand** (new, even narrower, and blocked on Host OS):

- Information technology revoke that matches the phone’s credential vault.
- Second laptop on the same account just works (synced vault), friend’s phone does not.
- A procurement story that is “authenticated pair,” not only “it disappeared.”

That second story is **not** something Nordic can ship alone. It needs a Host vault.

**Do not expect demand from** true-wireless earbud ODMs, Fast Pair living-room speakers, or Apple first.

Money still sits next to an open spec. See [`commercial.md`](commercial.md).

---

## Devices versus who will adopt

| Category | About / year | Hide | Vault passkey |
| --- | ---: | --- | --- |
| Phones, PCs, tablets | 1.5–2 billion | Host stays shown | **The actual product.** Last and required. Silent ceremony does not exist yet. |
| True-wireless earbuds | ~600 million | Apple yes; Fast Pair fights hide | Do not start here. |
| Headphones / headsets | ~300 million | Mixed | Later. |
| Bluetooth speakers | ~400 million | Premium often | Only shared-space hide. Passkey only with a Host story. |
| PC accessories | ~200 million | Often still public | Hide yes; vault maybe later. |
| UC / pro audio / rooms | Low millions | Inconsistent | **Best first OEM for hide.** Passkey only if they will take a Linux/test Host. |
| Tags, medical, industrial IoT | Fragmented, large | Usually still advertising | Niche later. |

---

## Companies, most likely first

1. **This repo** — simulator and file-vault test Host. Nobody else will invent the PDUs for us.
2. **Zephyr Project** — Accessory side (verify assertion, test factory key, then hide).
3. **BlueZ / Linux** — first Host stack that might grow a vault hook. Not a phone.
4. **Nordic Semiconductor** — sample + factory-attestation key story on nRF. Still not a Host vault.
5. **Shared-room OEMs** (Logitech, HP Poly, Shure, Bose Professional, Crestron) — buy the hide. Passkey only with a Host.
6. **Google / Microsoft** — silent vault next to Fast Pair / Windows pair. Harder than the hide-only path.
7. **Apple** — last or never.
8. **Cheap true-wireless ODMs** — ignore.

---

## Optimal development order

Optimize for a **correct oracle**, then a **hide video**, then a **test Host** that actually asserts. Do not tell an OEM the Development Kit proved the passkey.

| Phase | Work | Stop if |
| --- | --- | --- |
| **A** | Simulator: per-unit cert, SC-transcript attestation, assertion, sync vs stranger, unique scan rows, stealth. | Tests only pass by stubbing “always ok.” |
| **B** | nRF hide demo. Film: laptop pairs, second phone list empty. | Hide fails on a real scan list. |
| **C** | File-vault test Host + Accessory process (`sb-host` / `sb-accessory`). | We cannot check an assertion without inventing a passkey sheet (then the design is stuck). |
| **D** | BlueZ or userspace Host with that vault. | Maintainers refuse unknown opcodes or a vault hook. |
| **E** | Zephyr Accessory sample (test per-unit cert, verify assertions, hide). | — |
| **F** | One room OEM for hide. Mention passkey only with a Linux/test Host. | They only wanted hide. |
| **G** | SIG after D+E+F. Then Android/Windows silent ceremony. Apple last. | We went to SIG with only a hide demo. |

**This month:** rewrite the spec and Phase A simulator. Buy the kit when you want the hide video (B), not because it will show a passkey.
