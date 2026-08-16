use crate::pdu::{StealthBondingParams, PARAMS_LEN, PARAMS_OPCODE, PARAMS_RESPONSE_OPCODE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StealthPolicy {
    PeripheralOnly,
    CentralOnly,
    Both,
    Neither,
}

impl StealthPolicy {
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0x00 => Some(Self::PeripheralOnly),
            0x01 => Some(Self::CentralOnly),
            0x02 => Some(Self::Both),
            0x03 => Some(Self::Neither),
            _ => None,
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::PeripheralOnly => 0x00,
            Self::CentralOnly => 0x01,
            Self::Both => 0x02,
            Self::Neither => 0x03,
        }
    }

    pub const fn accessory_leaves_public(self) -> bool {
        matches!(self, Self::PeripheralOnly | Self::Both)
    }

    pub const fn host_leaves_public(self) -> bool {
        matches!(self, Self::CentralOnly | Self::Both)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconnectMode {
    DirectedKnock,
    SilentUntilWake,
    FilterAcceptList,
}

impl ReconnectMode {
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0x00 => Some(Self::DirectedKnock),
            0x01 => Some(Self::SilentUntilWake),
            0x02 => Some(Self::FilterAcceptList),
            _ => None,
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::DirectedKnock => 0x00,
            Self::SilentUntilWake => 0x01,
            Self::FilterAcceptList => 0x02,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BondPasskeyId(pub [u8; 16]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingMode {
    StealthBonding,
    Fallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairingOffer {
    pub policy: StealthPolicy,
    pub reconnect: ReconnectMode,
    pub require_delegated_confirm: bool,
    pub allow_peer_policy_change: bool,
}

impl Default for PairingOffer {
    fn default() -> Self {
        Self {
            policy: StealthPolicy::PeripheralOnly,
            reconnect: ReconnectMode::DirectedKnock,
            require_delegated_confirm: false,
            allow_peer_policy_change: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairingOutcome {
    pub mode: PairingMode,
    pub bond_passkey_id: Option<BondPasskeyId>,
    pub stealth_policy: StealthPolicy,
    pub reconnect_mode: ReconnectMode,
    pub params_pdu: Option<[u8; PARAMS_LEN]>,
    pub params_response_pdu: Option<[u8; PARAMS_LEN]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PairingError {
    NotEnrolling,
    TableFull,
    LegacyPairingForbidden,
    PolicyRejected,
    CommitmentFailed,
    ParamsRejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PduError {
    WrongLength,
    UnsupportedVersion,
    UnknownOpcode,
    UnknownPolicy,
    UnknownReconnectMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconnectError {
    NoBond,
    NotConnectable,
    EncryptionFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DelegatedError {
    UnbondedPeer,
    NoBond,
    TableFull,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Advertising {
    Off,
    Undirected { local_name: Option<String> },
    DirectedKnock,
}

impl Advertising {
    pub fn is_public(&self) -> bool {
        match self {
            Self::Off | Self::DirectedKnock => false,
            Self::Undirected { .. } => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GapState {
    Unprovisioned,
    Enrollment,
    BondedKnock,
    BondedSilent,
    PublicBonded,
}

struct StoredBond {
    id: BondPasskeyId,
    peer_identity: [u8; 6],
    ltk: [u8; 16],
    policy: StealthPolicy,
    reconnect: ReconnectMode,
}

pub struct Accessory {
    identity: [u8; 6],
    capacity: u8,
    supports_stealth_bonding: bool,
    supports_le_sc: bool,
    controller_can_leave_public_advertising: bool,
    state: GapState,
    advertising: Advertising,
    bonds: Vec<StoredBond>,
    next_id: u32,
    require_delegated_confirm: bool,
}

pub struct Host {
    identity: [u8; 6],
    supports_stealth_bonding: bool,
    supports_le_sc: bool,
    public_advertising: bool,
    capacity: u8,
    bonds: Vec<StoredBond>,
    next_id: u32,
    ltk_override: Option<[u8; 16]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BondedLink {
    pub bond_passkey_id: BondPasskeyId,
}

impl Accessory {
    pub fn new() -> Self {
        Self::with_capacity(8)
    }

    pub fn with_capacity(capacity: u8) -> Self {
        let capacity = capacity.max(2);
        Self {
            identity: [0x22, 0, 0, 0, 0, 0x02],
            capacity,
            supports_stealth_bonding: true,
            supports_le_sc: true,
            controller_can_leave_public_advertising: true,
            state: GapState::Unprovisioned,
            advertising: Advertising::Undirected {
                local_name: Some("Accessory".into()),
            },
            bonds: Vec::new(),
            next_id: 1,
            require_delegated_confirm: false,
        }
    }

    pub fn that_cannot_leave_public_advertising() -> Self {
        let mut accessory = Self::new();
        accessory.controller_can_leave_public_advertising = false;
        accessory
    }

    pub fn without_stealth_bonding() -> Self {
        let mut accessory = Self::new();
        accessory.supports_stealth_bonding = false;
        accessory
    }

    pub fn that_uses_legacy_pairing() -> Self {
        let mut accessory = Self::new();
        accessory.supports_le_sc = false;
        accessory
    }

    pub fn open_enrollment(&mut self) -> Result<(), PairingError> {
        if self.bonds.len() >= self.capacity as usize {
            return Err(PairingError::TableFull);
        }
        self.state = GapState::Enrollment;
        self.advertising = Advertising::Undirected {
            local_name: Some("Accessory".into()),
        };
        Ok(())
    }

    pub fn appears_on_public_scan_list(&self) -> bool {
        self.advertising.is_public()
    }

    pub fn advertising(&self) -> &Advertising {
        &self.advertising
    }

    pub fn bond_ids(&self) -> Vec<BondPasskeyId> {
        self.bonds.iter().map(|bond| bond.id).collect()
    }

    pub fn stealth_policy_for(&self, id: BondPasskeyId) -> Option<StealthPolicy> {
        self.bonds
            .iter()
            .find(|bond| bond.id == id)
            .map(|bond| bond.policy)
    }

    pub fn reconnect_mode_for(&self, id: BondPasskeyId) -> Option<ReconnectMode> {
        self.bonds
            .iter()
            .find(|bond| bond.id == id)
            .map(|bond| bond.reconnect)
    }

    pub fn requires_delegated_confirm(&self) -> bool {
        self.require_delegated_confirm
    }

    pub fn factory_reset(&mut self) {
        self.bonds.clear();
        self.state = GapState::Unprovisioned;
        self.advertising = Advertising::Undirected {
            local_name: Some("Accessory".into()),
        };
    }

    pub fn revoke(&mut self, id: BondPasskeyId) {
        self.bonds.retain(|bond| bond.id != id);
        if self.bonds.is_empty() {
            self.factory_reset();
        }
    }

    pub fn wake(&mut self) {
        if matches!(self.state, GapState::BondedSilent) {
            self.enter_bonded_reconnect(Some(ReconnectMode::DirectedKnock));
        }
    }

    pub fn open_delegated_enrollment(
        &mut self,
        link: Option<&BondedLink>,
    ) -> Result<(), DelegatedError> {
        let Some(link) = link else {
            return Err(DelegatedError::UnbondedPeer);
        };
        if !self
            .bonds
            .iter()
            .any(|bond| bond.id == link.bond_passkey_id)
        {
            return Err(DelegatedError::NoBond);
        }
        self.open_enrollment().map_err(|err| match err {
            PairingError::TableFull => DelegatedError::TableFull,
            _ => DelegatedError::NoBond,
        })?;
        Ok(())
    }

    fn allocate_id(&mut self) -> BondPasskeyId {
        let id = bond_id_from_counter(self.next_id);
        self.next_id += 1;
        id
    }

    fn enter_bonded_reconnect(&mut self, mode: Option<ReconnectMode>) {
        let mode = mode.unwrap_or(ReconnectMode::DirectedKnock);
        match mode {
            ReconnectMode::SilentUntilWake => {
                self.state = GapState::BondedSilent;
                self.advertising = Advertising::Off;
            }
            ReconnectMode::DirectedKnock | ReconnectMode::FilterAcceptList => {
                self.state = GapState::BondedKnock;
                self.advertising = Advertising::DirectedKnock;
            }
        }
    }
}

impl Default for Accessory {
    fn default() -> Self {
        Self::new()
    }
}

impl Host {
    pub fn new() -> Self {
        Self::with_identity([0x11, 0, 0, 0, 0, 0x01])
    }

    pub fn with_identity(identity: [u8; 6]) -> Self {
        Self {
            identity,
            supports_stealth_bonding: true,
            supports_le_sc: true,
            public_advertising: true,
            capacity: 8,
            bonds: Vec::new(),
            next_id: 1,
            ltk_override: None,
        }
    }

    pub fn without_stealth_bonding() -> Self {
        let mut host = Self::new();
        host.supports_stealth_bonding = false;
        host
    }

    pub fn that_uses_legacy_pairing() -> Self {
        let mut host = Self::new();
        host.supports_le_sc = false;
        host
    }

    pub fn break_ltk(&mut self) {
        self.ltk_override = Some([0xFF; 16]);
    }

    pub fn appears_on_public_scan_list(&self) -> bool {
        self.public_advertising
    }

    pub fn bond_ids(&self) -> Vec<BondPasskeyId> {
        self.bonds.iter().map(|bond| bond.id).collect()
    }

    fn allocate_id(&mut self) -> BondPasskeyId {
        let id = bond_id_from_counter(self.next_id);
        self.next_id += 1;
        id
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}

fn bond_id_from_counter(counter: u32) -> BondPasskeyId {
    let mut id = [0u8; 16];
    id[12..16].copy_from_slice(&counter.to_be_bytes());
    BondPasskeyId(id)
}

fn fixture_ltk(host: [u8; 6], accessory: [u8; 6]) -> [u8; 16] {
    let mut ltk = [0u8; 16];
    ltk[..6].copy_from_slice(&host);
    ltk[6..12].copy_from_slice(&accessory);
    ltk
}

pub fn pair(
    host: &mut Host,
    accessory: &mut Accessory,
    offer: PairingOffer,
) -> Result<PairingOutcome, PairingError> {
    if !matches!(accessory.state, GapState::Enrollment) {
        return Err(PairingError::NotEnrolling);
    }
    if accessory.bonds.len() >= accessory.capacity as usize {
        return Err(PairingError::TableFull);
    }
    if !host.supports_le_sc || !accessory.supports_le_sc {
        return Err(PairingError::LegacyPairingForbidden);
    }
    if !host.supports_stealth_bonding || !accessory.supports_stealth_bonding {
        return Ok(PairingOutcome {
            mode: PairingMode::Fallback,
            bond_passkey_id: None,
            stealth_policy: offer.policy,
            reconnect_mode: offer.reconnect,
            params_pdu: None,
            params_response_pdu: None,
        });
    }

    let params = StealthBondingParams {
        policy: offer.policy,
        reconnect: offer.reconnect,
        require_delegated_confirm: offer.require_delegated_confirm,
        allow_peer_policy_change: offer.allow_peer_policy_change,
        max_bonds: host.capacity,
    };
    let params_pdu = params.encode(PARAMS_OPCODE);
    let (opcode, decoded) =
        StealthBondingParams::decode(&params_pdu).map_err(|_| PairingError::ParamsRejected)?;
    if opcode != PARAMS_OPCODE || decoded.policy != offer.policy {
        return Err(PairingError::PolicyRejected);
    }

    let response = StealthBondingParams {
        policy: decoded.policy,
        reconnect: decoded.reconnect,
        require_delegated_confirm: decoded.require_delegated_confirm,
        allow_peer_policy_change: decoded.allow_peer_policy_change,
        max_bonds: accessory.capacity,
    };
    let params_response_pdu = response.encode(PARAMS_RESPONSE_OPCODE);

    let ltk = fixture_ltk(host.identity, accessory.identity);

    if offer.policy.accessory_leaves_public() {
        if accessory.controller_can_leave_public_advertising {
            accessory.enter_bonded_reconnect(Some(offer.reconnect));
        } else {
            accessory.advertising = Advertising::Undirected { local_name: None };
        }
        if accessory.advertising.is_public() {
            accessory.state = GapState::Enrollment;
            accessory.advertising = Advertising::Undirected {
                local_name: Some("Accessory".into()),
            };
            return Err(PairingError::CommitmentFailed);
        }
    } else {
        accessory.state = GapState::PublicBonded;
        accessory.advertising = Advertising::Undirected {
            local_name: Some("Accessory".into()),
        };
    }

    if offer.policy.host_leaves_public() {
        host.public_advertising = false;
    }

    accessory.require_delegated_confirm = offer.require_delegated_confirm;
    let accessory_id = accessory.allocate_id();
    let host_id = host.allocate_id();
    accessory.bonds.push(StoredBond {
        id: accessory_id,
        peer_identity: host.identity,
        ltk,
        policy: offer.policy,
        reconnect: offer.reconnect,
    });
    host.bonds.push(StoredBond {
        id: host_id,
        peer_identity: accessory.identity,
        ltk,
        policy: offer.policy,
        reconnect: offer.reconnect,
    });

    Ok(PairingOutcome {
        mode: PairingMode::StealthBonding,
        bond_passkey_id: Some(accessory_id),
        stealth_policy: offer.policy,
        reconnect_mode: offer.reconnect,
        params_pdu: Some(params_pdu),
        params_response_pdu: Some(params_response_pdu),
    })
}

pub fn reconnect(host: &Host, accessory: &mut Accessory) -> Result<BondedLink, ReconnectError> {
    let host_bond = host
        .bonds
        .iter()
        .find(|bond| bond.peer_identity == accessory.identity)
        .ok_or(ReconnectError::NoBond)?;
    let accessory_bond = accessory
        .bonds
        .iter()
        .find(|bond| bond.peer_identity == host.identity)
        .ok_or(ReconnectError::NoBond)?;

    match accessory.state {
        GapState::BondedKnock | GapState::PublicBonded | GapState::Enrollment => {}
        GapState::BondedSilent | GapState::Unprovisioned => {
            return Err(ReconnectError::NotConnectable);
        }
    }

    let presented_ltk = host.ltk_override.unwrap_or(host_bond.ltk);
    if presented_ltk != accessory_bond.ltk {
        return Err(ReconnectError::EncryptionFailed);
    }

    Ok(BondedLink {
        bond_passkey_id: accessory_bond.id,
    })
}
