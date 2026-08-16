# Accessory firmware

Zephyr 4.4 workspace app. Implements the GAP half of Stealth Bonding on stock Zephyr: Enrollment Window (Button 1), LE Secure Connections, Stealth Commitment (stop Public Advertising, Directed Knock). Draft SMP Params `0xF0`/`0xF1` are not in this tree.

Default board: `nrf52840dk/nrf52840`.

Buy list, Windows downloads, flash, and on-air demo: [`docs/nrf52840-dk-setup.md`](../docs/nrf52840-dk-setup.md).

## One-time setup

From the git root (requires [west](https://docs.zephyrproject.org/latest/develop/west/index.html) and the [Zephyr SDK](https://docs.zephyrproject.org/latest/develop/getting_started/index.html)):

```bash
pip install west
west init -l firmware
west update
west packages pip --install
west sdk install
```

`zephyr/` and other west projects stay out of git.

## Build and flash

```bash
west build -p always -b nrf52840dk/nrf52840 firmware/accessory
west flash
```

Button 1 opens a 30 s Enrollment Window (LED0 on). After a successful bond the Accessory leaves undirected advertising. Hold Button 1 again to add another Bond Passkey.

The GAP state machine (no radio) can be checked without Zephyr:

```bash
gcc -I firmware/accessory/src firmware/accessory/tests/test_gap_sm.c \
  firmware/accessory/src/gap_sm.c -o /tmp/test_gap_sm && /tmp/test_gap_sm
```

## What this is not

- Not a download for arbitrary speakers
- Not the draft SMP opcodes `0xF0`–`0xF3` (those bytes live in `sim/`; on-air needs a Zephyr host patch and a test Host)
- Not nRF Connect SDK (vanilla Zephyr 4.4.2)
