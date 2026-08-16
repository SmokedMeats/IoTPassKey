# PDU simulator is Rust, not Python

The first implementation is a spec oracle: binary PDUs and an advertising-state machine. Rust models those as enums and makes an illegal “bonded but still on Public Advertising” state hard to represent. Python would be faster to sketch and would likely be rewritten if the simulator becomes a test-tool product.

Firmware on a chip stays C. The simulator does not.
