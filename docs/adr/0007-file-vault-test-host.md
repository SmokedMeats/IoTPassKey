# First Host is a file vault, not a platform ceremony

The first Host that actually holds a Bond Passkey private key is a Rust process in this repo (`host/`). The key is a file. Create and assert are silent. There is no WebAuthn sheet, account picker, or typed secret.

It talks length-prefixed draft SMP PDUs (`0xF0`–`0xF5`) to a second process that is the simulator Accessory. That is a link we control. It is not Bluetooth radio and not iCloud / Google / Windows vault sync.

A later BlueZ or phone stack should keep the same PDUs and the same silent create/assert. It should replace only the store (file → platform vault) and the transport (TCP → HCI/SMP).
