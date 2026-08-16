# Pairing lives in SMP and GAP, not GATT or an app

Stealth Bonding has to change bonding and advertising, which already live in the Security Manager and Generic Access Profile. A GATT service cannot own those transitions; a user-space app cannot install them on arbitrary radios.

The interoperable form is a Core Spec pairing-mode extension. A GATT management service is allowed only after a Bond Passkey exists, for list/revoke/rename/open-enrollment. Implementations ship as firmware and OS stack updates, not as a download for any Bluetooth device.
