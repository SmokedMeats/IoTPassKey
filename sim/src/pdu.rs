use crate::model::{PduError, ReconnectMode, StealthPolicy};

pub const PARAMS_OPCODE: u8 = 0xF0;
pub const PARAMS_RESPONSE_OPCODE: u8 = 0xF1;
pub const PARAMS_VERSION: u8 = 0x01;
pub const PARAMS_LEN: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StealthBondingParams {
    pub policy: StealthPolicy,
    pub reconnect: ReconnectMode,
    pub require_delegated_confirm: bool,
    pub allow_peer_policy_change: bool,
    pub max_bonds: u8,
}

impl StealthBondingParams {
    pub fn encode(self, opcode: u8) -> [u8; PARAMS_LEN] {
        let mut flags = 0u8;
        if self.require_delegated_confirm {
            flags |= 1 << 0;
        }
        if self.allow_peer_policy_change {
            flags |= 1 << 1;
        }
        [
            opcode,
            PARAMS_VERSION,
            self.policy.code(),
            self.reconnect.code(),
            flags,
            self.max_bonds,
            0,
            0,
        ]
    }

    pub fn decode(bytes: &[u8]) -> Result<(u8, Self), PduError> {
        if bytes.len() != PARAMS_LEN {
            return Err(PduError::WrongLength);
        }
        if bytes[1] != PARAMS_VERSION {
            return Err(PduError::UnsupportedVersion);
        }
        let opcode = bytes[0];
        if opcode != PARAMS_OPCODE && opcode != PARAMS_RESPONSE_OPCODE {
            return Err(PduError::UnknownOpcode);
        }
        let policy = StealthPolicy::from_code(bytes[2]).ok_or(PduError::UnknownPolicy)?;
        let reconnect = ReconnectMode::from_code(bytes[3]).ok_or(PduError::UnknownReconnectMode)?;
        let flags = bytes[4];
        Ok((
            opcode,
            Self {
                policy,
                reconnect,
                require_delegated_confirm: flags & (1 << 0) != 0,
                allow_peer_policy_change: flags & (1 << 1) != 0,
                max_bonds: bytes[5],
            },
        ))
    }
}
