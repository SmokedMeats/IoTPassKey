# After commitment, no Public Advertising — stronger than LE Privacy

LE Privacy still sends undirected advertisements with rotating addresses. Those appear as nameless devices in many scan UIs. The requirement from this project is that a normal scan list shows nothing.

Default Stealth Policy is PeripheralOnly: the Accessory leaves Public Advertising; the Host may stay discoverable. Reconnect uses Directed Knock or SilentUntilWake. Radio packets may still exist on air; sniffers are out of scope. If Stealth Commitment cannot stop Public Advertising, pairing fails and keys are discarded.
