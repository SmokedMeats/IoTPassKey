use crate::model::{PduError, ReconnectMode, StealthPolicy};

pub const PARAMS_OPCODE: u8 = 0xF0;
pub const PARAMS_RESPONSE_OPCODE: u8 = 0xF1;
pub const ATTESTATION_OPCODE: u8 = 0xF2;
pub const REGISTER_OPCODE: u8 = 0xF3;
pub const CHALLENGE_OPCODE: u8 = 0xF4;
pub const ASSERTION_OPCODE: u8 = 0xF5;
pub const PARAMS_VERSION: u8 = 0x01;
pub const PARAMS_LEN: usize = 8;
pub const ATTESTATION_REQUEST_LEN: usize = 35;
pub const ATTESTATION_LEN: usize = 204;
pub const REGISTER_LEN: usize = 51;
pub const CHALLENGE_LEN: usize = 34;
pub const ASSERTION_LEN: usize = 82;
pub const ATTESTATION_FLAG_REQUEST: u8 = 0x00;
pub const ATTESTATION_FLAG_RESPONSE: u8 = 0x01;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StealthBondingAttestationRequest {
    pub nonce: [u8; 32],
}

impl StealthBondingAttestationRequest {
    pub fn encode(self) -> [u8; ATTESTATION_REQUEST_LEN] {
        let mut out = [0u8; ATTESTATION_REQUEST_LEN];
        out[0] = ATTESTATION_OPCODE;
        out[1] = PARAMS_VERSION;
        out[2] = ATTESTATION_FLAG_REQUEST;
        out[3..35].copy_from_slice(&self.nonce);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PduError> {
        if bytes.len() != ATTESTATION_REQUEST_LEN {
            return Err(PduError::WrongLength);
        }
        if bytes[0] != ATTESTATION_OPCODE {
            return Err(PduError::UnknownOpcode);
        }
        if bytes[1] != PARAMS_VERSION {
            return Err(PduError::UnsupportedVersion);
        }
        if bytes[2] != ATTESTATION_FLAG_REQUEST {
            return Err(PduError::UnknownOpcode);
        }
        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&bytes[3..35]);
        Ok(Self { nonce })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StealthBondingAttestation {
    pub nonce: [u8; 32],
    pub unit_public: [u8; 33],
    pub unit_serial: [u8; 8],
    pub unit_cert: [u8; 64],
    pub attestation: [u8; 64],
}

impl StealthBondingAttestation {
    pub fn encode(self) -> [u8; ATTESTATION_LEN] {
        let mut out = [0u8; ATTESTATION_LEN];
        out[0] = ATTESTATION_OPCODE;
        out[1] = PARAMS_VERSION;
        out[2] = ATTESTATION_FLAG_RESPONSE;
        out[3..35].copy_from_slice(&self.nonce);
        out[35..68].copy_from_slice(&self.unit_public);
        out[68..76].copy_from_slice(&self.unit_serial);
        out[76..140].copy_from_slice(&self.unit_cert);
        out[140..204].copy_from_slice(&self.attestation);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PduError> {
        if bytes.len() != ATTESTATION_LEN {
            return Err(PduError::WrongLength);
        }
        if bytes[0] != ATTESTATION_OPCODE {
            return Err(PduError::UnknownOpcode);
        }
        if bytes[1] != PARAMS_VERSION {
            return Err(PduError::UnsupportedVersion);
        }
        if bytes[2] != ATTESTATION_FLAG_RESPONSE {
            return Err(PduError::UnknownOpcode);
        }
        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&bytes[3..35]);
        let mut unit_public = [0u8; 33];
        unit_public.copy_from_slice(&bytes[35..68]);
        let mut unit_serial = [0u8; 8];
        unit_serial.copy_from_slice(&bytes[68..76]);
        let mut unit_cert = [0u8; 64];
        unit_cert.copy_from_slice(&bytes[76..140]);
        let mut attestation = [0u8; 64];
        attestation.copy_from_slice(&bytes[140..204]);
        Ok(Self {
            nonce,
            unit_public,
            unit_serial,
            unit_cert,
            attestation,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StealthBondingRegister {
    pub credential_id: [u8; 16],
    pub public_key: [u8; 33],
}

impl StealthBondingRegister {
    pub fn encode(self) -> [u8; REGISTER_LEN] {
        let mut out = [0u8; REGISTER_LEN];
        out[0] = REGISTER_OPCODE;
        out[1] = PARAMS_VERSION;
        out[2..18].copy_from_slice(&self.credential_id);
        out[18..51].copy_from_slice(&self.public_key);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PduError> {
        if bytes.len() != REGISTER_LEN {
            return Err(PduError::WrongLength);
        }
        if bytes[0] != REGISTER_OPCODE {
            return Err(PduError::UnknownOpcode);
        }
        if bytes[1] != PARAMS_VERSION {
            return Err(PduError::UnsupportedVersion);
        }
        let mut credential_id = [0u8; 16];
        credential_id.copy_from_slice(&bytes[2..18]);
        let mut public_key = [0u8; 33];
        public_key.copy_from_slice(&bytes[18..51]);
        Ok(Self {
            credential_id,
            public_key,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StealthBondingChallenge {
    pub challenge: [u8; 32],
}

impl StealthBondingChallenge {
    pub fn encode(self) -> [u8; CHALLENGE_LEN] {
        let mut out = [0u8; CHALLENGE_LEN];
        out[0] = CHALLENGE_OPCODE;
        out[1] = PARAMS_VERSION;
        out[2..34].copy_from_slice(&self.challenge);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PduError> {
        if bytes.len() != CHALLENGE_LEN {
            return Err(PduError::WrongLength);
        }
        if bytes[0] != CHALLENGE_OPCODE {
            return Err(PduError::UnknownOpcode);
        }
        if bytes[1] != PARAMS_VERSION {
            return Err(PduError::UnsupportedVersion);
        }
        let mut challenge = [0u8; 32];
        challenge.copy_from_slice(&bytes[2..34]);
        Ok(Self { challenge })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StealthBondingAssertion {
    pub credential_id: [u8; 16],
    pub signature: [u8; 64],
}

impl StealthBondingAssertion {
    pub fn encode(self) -> [u8; ASSERTION_LEN] {
        let mut out = [0u8; ASSERTION_LEN];
        out[0] = ASSERTION_OPCODE;
        out[1] = PARAMS_VERSION;
        out[2..18].copy_from_slice(&self.credential_id);
        out[18..82].copy_from_slice(&self.signature);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, PduError> {
        if bytes.len() != ASSERTION_LEN {
            return Err(PduError::WrongLength);
        }
        if bytes[0] != ASSERTION_OPCODE {
            return Err(PduError::UnknownOpcode);
        }
        if bytes[1] != PARAMS_VERSION {
            return Err(PduError::UnsupportedVersion);
        }
        let mut credential_id = [0u8; 16];
        credential_id.copy_from_slice(&bytes[2..18]);
        let mut signature = [0u8; 64];
        signature.copy_from_slice(&bytes[18..82]);
        Ok(Self {
            credential_id,
            signature,
        })
    }
}
