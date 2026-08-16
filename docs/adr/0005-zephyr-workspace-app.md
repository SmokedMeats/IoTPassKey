# First firmware is a Zephyr workspace app, not nRF Connect SDK

The Accessory adapter belongs in `firmware/` as a west workspace application that imports Zephyr 4.4. Default board is `nrf52840dk/nrf52840`.

Vanilla Zephyr is enough for LE Secure Connections and advertising control. NCS would pull a second SDK for no gain on this first board. Custom SMP opcodes are out of scope until we patch or upstream the host stack; this app implements Enrollment, LE SC bonding, and Stealth Commitment at GAP.
