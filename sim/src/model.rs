use crate::crypto::{
    assertion_message, attestation_message, fixture_sc_public, issue_unit_cert, public_key, sign,
    unit_cert_message, unit_secret_from_identity, unit_serial_from_identity, unit_serial_suffix,
    vault_secret_from_identity, verify, Nonce, PublicKeyBytes, Secret, SignatureBytes, UnitSerial,
    DEFAULT_MANUFACTURER_SECRET,
};
use crate::pdu::{
    StealthBondingAssertion, StealthBondingAttestation, StealthBondingAttestationRequest,
    StealthBondingParams, StealthBondingRegister, ATTESTATION_LEN, ATTESTATION_REQUEST_LEN,
    PARAMS_LEN, PARAMS_OPCODE, PARAMS_RESPONSE_OPCODE, REGISTER_LEN,
};

pub const T_SB_ENROLL_MS: u32 = 30_000;
pub const T_SB_ENROLL_MAX_MS: u32 = 180_000;

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
    pub attestation_request_pdu: Option<[u8; ATTESTATION_REQUEST_LEN]>,
    pub attestation_pdu: Option<[u8; ATTESTATION_LEN]>,
    pub register_pdu: Option<[u8; REGISTER_LEN]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PairingError {
    NotEnrolling,
    TableFull,
    LegacyPairingForbidden,
    PolicyRejected,
    CommitmentFailed,
    ParamsRejected,
    DelegatedConfirmRequired,
    AttestationFailed,
    AttestationReplay,
    AcousticFailed,
    ClosenessFailed,
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
    AssertionFailed,
    AssertionReplay,
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

#[derive(Clone)]
struct StoredBond {
    id: BondPasskeyId,
    /// Peer identity for Directed Knock. Reconnect looks up by public key.
    #[allow(dead_code)]
    peer_identity: [u8; 6],
    ltk: [u8; 16],
    policy: StealthPolicy,
    reconnect: ReconnectMode,
    public_key: PublicKeyBytes,
}

struct OrdinaryBond {
    peer_identity: [u8; 6],
    ltk: [u8; 16],
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
    ordinary_bonds: Vec<OrdinaryBond>,
    require_delegated_confirm: bool,
    enrollment_remaining_ms: Option<u32>,
    pending_authorizer: Option<BondPasskeyId>,
    delegated_confirmed: bool,
    factory_secret: Option<Secret>,
    unit_serial: UnitSerial,
    unit_cert: Option<SignatureBytes>,
    product_name: String,
    omit_sc_from_attestation: bool,
    enrollment_rssi_dbm: i8,
    plays_acoustic: bool,
    used_assertion_challenges: Vec<Nonce>,
    next_challenge: u32,
    replay_assertion_challenge: bool,
    last_assertion_challenge: Option<Nonce>,
    pending_enrollment: Option<PendingEnrollment>,
}

struct PendingEnrollment {
    host_identity: [u8; 6],
    policy: StealthPolicy,
    reconnect: ReconnectMode,
    nonce: Nonce,
    require_delegated_confirm: bool,
}

pub struct Host {
    identity: [u8; 6],
    supports_stealth_bonding: bool,
    supports_le_sc: bool,
    public_advertising: bool,
    capacity: u8,
    bonds: Vec<StoredBond>,
    ordinary_bonds: Vec<OrdinaryBond>,
    next_id: u32,
    ltk_override: Option<[u8; 16]>,
    vault_secret: Secret,
    credential_id: Option<BondPasskeyId>,
    trusted_manufacturer_public: Option<PublicKeyBytes>,
    used_attestation_nonces: Vec<Nonce>,
    next_nonce: u32,
    replay_attestation_nonce: bool,
    last_attestation_nonce: Option<Nonce>,
    min_enrollment_rssi_dbm: Option<i8>,
    require_acoustic: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BondedLink {
    pub bond_passkey_id: Option<BondPasskeyId>,
}

impl Accessory {
    pub fn new() -> Self {
        Self::with_capacity(8)
    }

    pub fn with_capacity(capacity: u8) -> Self {
        Self::with_identity([0x22, 0, 0, 0, 0, 0x02], capacity)
    }

    pub fn with_identity(identity: [u8; 6], capacity: u8) -> Self {
        let capacity = capacity.max(2);
        let unit_secret = unit_secret_from_identity(identity);
        let unit_serial = unit_serial_from_identity(identity);
        let unit_public = public_key(&unit_secret);
        Self {
            identity,
            capacity,
            supports_stealth_bonding: true,
            supports_le_sc: true,
            controller_can_leave_public_advertising: true,
            state: GapState::Unprovisioned,
            advertising: Advertising::Undirected {
                local_name: Some("Accessory".into()),
            },
            bonds: Vec::new(),
            ordinary_bonds: Vec::new(),
            require_delegated_confirm: false,
            enrollment_remaining_ms: None,
            pending_authorizer: None,
            delegated_confirmed: false,
            factory_secret: Some(unit_secret),
            unit_serial,
            unit_cert: Some(issue_unit_cert(&unit_public, &unit_serial)),
            product_name: "Accessory".into(),
            omit_sc_from_attestation: false,
            enrollment_rssi_dbm: -40,
            plays_acoustic: true,
            used_assertion_challenges: Vec::new(),
            next_challenge: 1,
            replay_assertion_challenge: false,
            last_assertion_challenge: None,
            pending_enrollment: None,
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

    pub fn without_factory_attestation() -> Self {
        let mut accessory = Self::new();
        accessory.factory_secret = None;
        accessory.unit_cert = None;
        accessory
    }

    pub fn that_omits_sc_from_attestation() -> Self {
        let mut accessory = Self::new();
        accessory.omit_sc_from_attestation = true;
        accessory
    }

    pub fn with_forged_unit_cert() -> Self {
        let mut accessory = Self::new();
        let mut rogue = [0u8; 32];
        rogue[31] = 2;
        let unit_public = public_key(accessory.factory_secret.as_ref().expect("unit key"));
        accessory.unit_cert = Some(sign(
            &rogue,
            &unit_cert_message(&unit_public, &accessory.unit_serial),
        ));
        accessory
    }

    pub fn that_is_far_away() -> Self {
        let mut accessory = Self::new();
        accessory.enrollment_rssi_dbm = -90;
        accessory
    }

    pub fn without_acoustic() -> Self {
        let mut accessory = Self::new();
        accessory.plays_acoustic = false;
        accessory
    }

    pub fn local_name(&self) -> &str {
        &self.product_name
    }

    pub fn enrollment_scan_row(&self) -> String {
        format!(
            "{} · {}",
            self.product_name,
            unit_serial_suffix(&self.unit_serial)
        )
    }

    pub fn identity(&self) -> [u8; 6] {
        self.identity
    }

    pub fn respond_to_params(
        &mut self,
        params: StealthBondingParams,
    ) -> Result<StealthBondingParams, PairingError> {
        if !matches!(self.state, GapState::Enrollment) {
            return Err(PairingError::NotEnrolling);
        }
        if self.bonds.len() >= self.capacity as usize {
            return Err(PairingError::TableFull);
        }
        if !self.supports_le_sc {
            return Err(PairingError::LegacyPairingForbidden);
        }
        if !self.supports_stealth_bonding {
            return Err(PairingError::ParamsRejected);
        }
        if self.pending_authorizer.is_some() && !self.delegated_confirmed {
            return Err(PairingError::DelegatedConfirmRequired);
        }
        self.pending_enrollment = Some(PendingEnrollment {
            host_identity: [0; 6],
            policy: params.policy,
            reconnect: params.reconnect,
            nonce: [0; 32],
            require_delegated_confirm: params.require_delegated_confirm,
        });
        Ok(StealthBondingParams {
            policy: params.policy,
            reconnect: params.reconnect,
            require_delegated_confirm: params.require_delegated_confirm,
            allow_peer_policy_change: params.allow_peer_policy_change,
            max_bonds: self.capacity,
        })
    }

    pub fn attest_for_host(
        &mut self,
        host_identity: [u8; 6],
        nonce: Nonce,
    ) -> Result<StealthBondingAttestation, PairingError> {
        let Some(pending) = self.pending_enrollment.as_mut() else {
            return Err(PairingError::NotEnrolling);
        };
        pending.host_identity = host_identity;
        pending.nonce = nonce;
        let Some(factory_secret) = self.factory_secret.as_ref() else {
            return Err(PairingError::AttestationFailed);
        };
        let Some(unit_cert) = self.unit_cert.as_ref() else {
            return Err(PairingError::AttestationFailed);
        };
        let unit_public = public_key(factory_secret);
        let host_sc_public = fixture_sc_public(host_identity);
        let accessory_sc_public = fixture_sc_public(self.identity);
        let signed = if self.omit_sc_from_attestation {
            let mut message = Vec::with_capacity(6 + 6 + 1 + 1 + 32);
            message.extend_from_slice(&host_identity);
            message.extend_from_slice(&self.identity);
            message.push(pending.policy.code());
            message.push(pending.reconnect.code());
            message.extend_from_slice(&nonce);
            message
        } else {
            attestation_message(
                host_identity,
                self.identity,
                pending.policy.code(),
                pending.reconnect.code(),
                &nonce,
                &host_sc_public,
                &accessory_sc_public,
            )
        };
        Ok(StealthBondingAttestation {
            nonce,
            unit_public,
            unit_serial: self.unit_serial,
            unit_cert: *unit_cert,
            attestation: sign(factory_secret, &signed),
        })
    }

    pub fn accept_register(
        &mut self,
        register: StealthBondingRegister,
        host_identity: [u8; 6],
    ) -> Result<(), PairingError> {
        let pending = self
            .pending_enrollment
            .take()
            .ok_or(PairingError::NotEnrolling)?;
        if pending.host_identity != host_identity {
            self.pending_enrollment = Some(pending);
            return Err(PairingError::AttestationFailed);
        }
        let ltk = fixture_ltk(host_identity, self.identity);
        if pending.policy.accessory_leaves_public() {
            if self.controller_can_leave_public_advertising {
                self.enter_bonded_reconnect(Some(pending.reconnect));
            } else {
                self.advertising = Advertising::Undirected { local_name: None };
            }
            if self.advertising.is_public() {
                self.state = GapState::Enrollment;
                self.advertising = Advertising::Undirected {
                    local_name: Some("Accessory".into()),
                };
                return Err(PairingError::CommitmentFailed);
            }
        } else {
            self.state = GapState::PublicBonded;
            self.advertising = Advertising::Undirected {
                local_name: Some("Accessory".into()),
            };
        }
        self.require_delegated_confirm = pending.require_delegated_confirm;
        self.clear_enrollment_clock();
        self.bonds.push(StoredBond {
            id: BondPasskeyId(register.credential_id),
            peer_identity: host_identity,
            ltk,
            policy: pending.policy,
            reconnect: pending.reconnect,
            public_key: register.public_key,
        });
        Ok(())
    }

    pub fn issue_challenge(&mut self) -> Result<Nonce, ReconnectError> {
        if self.bonds.is_empty() {
            return Err(ReconnectError::NoBond);
        }
        match self.state {
            GapState::BondedKnock | GapState::PublicBonded | GapState::Enrollment => {}
            GapState::BondedSilent | GapState::Unprovisioned => {
                return Err(ReconnectError::NotConnectable);
            }
        }
        let challenge = if self.replay_assertion_challenge {
            self.replay_assertion_challenge = false;
            self.last_assertion_challenge
                .ok_or(ReconnectError::AssertionReplay)?
        } else {
            let challenge = nonce_from_counter(self.next_challenge);
            self.next_challenge += 1;
            challenge
        };
        if self.used_assertion_challenges.contains(&challenge) {
            return Err(ReconnectError::AssertionReplay);
        }
        Ok(challenge)
    }

    pub fn accept_assertion(
        &mut self,
        challenge: Nonce,
        assertion: StealthBondingAssertion,
    ) -> Result<BondedLink, ReconnectError> {
        if self.used_assertion_challenges.contains(&challenge) {
            return Err(ReconnectError::AssertionReplay);
        }
        let message = assertion_message(&challenge, self.identity);
        let Some(accessory_bond) = self.bonds.iter().find(|bond| {
            bond.id.0 == assertion.credential_id
                && verify(&bond.public_key, &message, &assertion.signature)
        }) else {
            return Err(ReconnectError::NoBond);
        };
        let id = accessory_bond.id;
        self.used_assertion_challenges.push(challenge);
        self.last_assertion_challenge = Some(challenge);
        Ok(BondedLink {
            bond_passkey_id: Some(id),
        })
    }

    pub fn replay_next_assertion_challenge(&mut self) {
        self.replay_assertion_challenge = true;
    }

    pub fn open_enrollment(&mut self) -> Result<(), PairingError> {
        if self.bonds.len() >= self.capacity as usize {
            return Err(PairingError::TableFull);
        }
        self.state = GapState::Enrollment;
        self.advertising = Advertising::Undirected {
            local_name: Some("Accessory".into()),
        };
        self.enrollment_remaining_ms = Some(T_SB_ENROLL_MS.min(T_SB_ENROLL_MAX_MS));
        self.pending_authorizer = None;
        self.delegated_confirmed = false;
        Ok(())
    }

    pub fn ordinary_bond_count(&self) -> usize {
        self.ordinary_bonds.len()
    }

    pub fn advance(&mut self, ms: u32) {
        let Some(remaining) = self.enrollment_remaining_ms else {
            return;
        };
        if remaining > ms {
            self.enrollment_remaining_ms = Some(remaining - ms);
            return;
        }
        self.close_enrollment_window();
    }

    pub fn confirm_delegated_enrollment(
        &mut self,
        link: &BondedLink,
    ) -> Result<(), DelegatedError> {
        let Some(id) = link.bond_passkey_id else {
            return Err(DelegatedError::UnbondedPeer);
        };
        if self.pending_authorizer != Some(id) {
            return Err(DelegatedError::NoBond);
        }
        if !self.bonds.iter().any(|bond| bond.id == id) {
            return Err(DelegatedError::NoBond);
        }
        self.delegated_confirmed = true;
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
        self.ordinary_bonds.clear();
        self.pending_authorizer = None;
        self.delegated_confirmed = false;
        self.enrollment_remaining_ms = None;
        self.used_assertion_challenges.clear();
        self.last_assertion_challenge = None;
        self.pending_enrollment = None;
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
        let Some(id) = link.bond_passkey_id else {
            return Err(DelegatedError::UnbondedPeer);
        };
        if !self.bonds.iter().any(|bond| bond.id == id) {
            return Err(DelegatedError::NoBond);
        }
        self.open_enrollment().map_err(|err| match err {
            PairingError::TableFull => DelegatedError::TableFull,
            _ => DelegatedError::NoBond,
        })?;
        self.pending_authorizer = Some(id);
        self.delegated_confirmed = !self.require_delegated_confirm;
        Ok(())
    }

    fn close_enrollment_window(&mut self) {
        self.enrollment_remaining_ms = None;
        self.pending_authorizer = None;
        self.delegated_confirmed = false;
        if self.bonds.is_empty() {
            self.state = GapState::Unprovisioned;
            self.advertising = Advertising::Undirected {
                local_name: Some("Accessory".into()),
            };
            return;
        }
        let mode = self.bonds.last().map(|bond| bond.reconnect);
        self.enter_bonded_reconnect(mode);
    }

    fn clear_enrollment_clock(&mut self) {
        self.enrollment_remaining_ms = None;
        self.pending_authorizer = None;
        self.delegated_confirmed = false;
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
            ordinary_bonds: Vec::new(),
            next_id: 1,
            ltk_override: None,
            vault_secret: vault_secret_from_identity(identity),
            credential_id: None,
            trusted_manufacturer_public: Some(public_key(&DEFAULT_MANUFACTURER_SECRET)),
            used_attestation_nonces: Vec::new(),
            next_nonce: 1,
            replay_attestation_nonce: false,
            last_attestation_nonce: None,
            min_enrollment_rssi_dbm: None,
            require_acoustic: false,
        }
    }

    pub fn that_cannot_verify_attestation() -> Self {
        let mut host = Self::new();
        host.trusted_manufacturer_public = None;
        host
    }

    pub fn that_requires_closeness(min_rssi_dbm: i8) -> Self {
        let mut host = Self::new();
        host.min_enrollment_rssi_dbm = Some(min_rssi_dbm);
        host
    }

    pub fn that_requires_acoustic() -> Self {
        let mut host = Self::new();
        host.require_acoustic = true;
        host
    }

    pub fn sharing_vault_with(other: &Self, identity: [u8; 6]) -> Self {
        let mut host = Self::with_identity(identity);
        host.vault_secret = other.vault_secret;
        host.credential_id = other.credential_id;
        host.bonds = other.bonds.clone();
        host
    }

    pub fn replay_next_attestation_nonce(&mut self) {
        self.replay_attestation_nonce = true;
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

fn nonce_from_counter(counter: u32) -> Nonce {
    let mut nonce = [0u8; 32];
    nonce[28..32].copy_from_slice(&counter.to_be_bytes());
    nonce
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
        let ltk = fixture_ltk(host.identity, accessory.identity);
        accessory.ordinary_bonds.push(OrdinaryBond {
            peer_identity: host.identity,
            ltk,
        });
        host.ordinary_bonds.push(OrdinaryBond {
            peer_identity: accessory.identity,
            ltk,
        });
        accessory.close_enrollment_window();
        return Ok(PairingOutcome {
            mode: PairingMode::Fallback,
            bond_passkey_id: None,
            stealth_policy: offer.policy,
            reconnect_mode: offer.reconnect,
            params_pdu: None,
            params_response_pdu: None,
            attestation_request_pdu: None,
            attestation_pdu: None,
            register_pdu: None,
        });
    }

    if accessory.pending_authorizer.is_some() && !accessory.delegated_confirmed {
        return Err(PairingError::DelegatedConfirmRequired);
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

    let nonce = if host.replay_attestation_nonce {
        host.replay_attestation_nonce = false;
        host.last_attestation_nonce
            .ok_or(PairingError::AttestationReplay)?
    } else {
        let nonce = nonce_from_counter(host.next_nonce);
        host.next_nonce += 1;
        nonce
    };
    if host.used_attestation_nonces.contains(&nonce) {
        return Err(PairingError::AttestationReplay);
    }

    if let Some(min_rssi) = host.min_enrollment_rssi_dbm {
        if accessory.enrollment_rssi_dbm < min_rssi {
            return Err(PairingError::ClosenessFailed);
        }
    }

    let attestation_request_pdu = StealthBondingAttestationRequest { nonce }.encode();
    let request = StealthBondingAttestationRequest::decode(&attestation_request_pdu)
        .map_err(|_| PairingError::AttestationFailed)?;

    let host_sc_public = fixture_sc_public(host.identity);
    let accessory_sc_public = fixture_sc_public(accessory.identity);

    let Some(factory_secret) = accessory.factory_secret.as_ref() else {
        return Err(PairingError::AttestationFailed);
    };
    let Some(manufacturer) = host.trusted_manufacturer_public.as_ref() else {
        return Err(PairingError::AttestationFailed);
    };
    let Some(unit_cert) = accessory.unit_cert.as_ref() else {
        return Err(PairingError::AttestationFailed);
    };
    let unit_public = public_key(factory_secret);
    let signed = if accessory.omit_sc_from_attestation {
        let mut message = Vec::with_capacity(6 + 6 + 1 + 1 + 32);
        message.extend_from_slice(&host.identity);
        message.extend_from_slice(&accessory.identity);
        message.push(offer.policy.code());
        message.push(offer.reconnect.code());
        message.extend_from_slice(&request.nonce);
        message
    } else {
        attestation_message(
            host.identity,
            accessory.identity,
            offer.policy.code(),
            offer.reconnect.code(),
            &request.nonce,
            &host_sc_public,
            &accessory_sc_public,
        )
    };
    let attestation_pdu = StealthBondingAttestation {
        nonce: request.nonce,
        unit_public,
        unit_serial: accessory.unit_serial,
        unit_cert: *unit_cert,
        attestation: sign(factory_secret, &signed),
    }
    .encode();
    let attestation = StealthBondingAttestation::decode(&attestation_pdu)
        .map_err(|_| PairingError::AttestationFailed)?;
    if !verify(
        manufacturer,
        &unit_cert_message(&attestation.unit_public, &attestation.unit_serial),
        &attestation.unit_cert,
    ) {
        return Err(PairingError::AttestationFailed);
    }
    let expected = attestation_message(
        host.identity,
        accessory.identity,
        offer.policy.code(),
        offer.reconnect.code(),
        &attestation.nonce,
        &host_sc_public,
        &accessory_sc_public,
    );
    if !verify(
        &attestation.unit_public,
        &expected,
        &attestation.attestation,
    ) {
        return Err(PairingError::AttestationFailed);
    }
    if host.require_acoustic && !accessory.plays_acoustic {
        return Err(PairingError::AcousticFailed);
    }
    host.used_attestation_nonces.push(attestation.nonce);
    host.last_attestation_nonce = Some(attestation.nonce);

    let vault_public = public_key(&host.vault_secret);
    let credential_id = host.credential_id.unwrap_or_else(|| host.allocate_id());
    host.credential_id = Some(credential_id);
    let register_pdu = StealthBondingRegister {
        credential_id: credential_id.0,
        public_key: vault_public,
    }
    .encode();
    let register =
        StealthBondingRegister::decode(&register_pdu).map_err(|_| PairingError::ParamsRejected)?;
    if register.credential_id != credential_id.0 || register.public_key != vault_public {
        return Err(PairingError::ParamsRejected);
    }

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
    accessory.clear_enrollment_clock();
    accessory.bonds.push(StoredBond {
        id: credential_id,
        peer_identity: host.identity,
        ltk,
        policy: offer.policy,
        reconnect: offer.reconnect,
        public_key: vault_public,
    });
    host.bonds.push(StoredBond {
        id: credential_id,
        peer_identity: accessory.identity,
        ltk,
        policy: offer.policy,
        reconnect: offer.reconnect,
        public_key: vault_public,
    });

    Ok(PairingOutcome {
        mode: PairingMode::StealthBonding,
        bond_passkey_id: Some(credential_id),
        stealth_policy: offer.policy,
        reconnect_mode: offer.reconnect,
        params_pdu: Some(params_pdu),
        params_response_pdu: Some(params_response_pdu),
        attestation_request_pdu: Some(attestation_request_pdu),
        attestation_pdu: Some(attestation_pdu),
        register_pdu: Some(register_pdu),
    })
}

pub fn reconnect(host: &Host, accessory: &mut Accessory) -> Result<BondedLink, ReconnectError> {
    if !accessory.bonds.is_empty() {
        match accessory.state {
            GapState::BondedKnock | GapState::PublicBonded | GapState::Enrollment => {}
            GapState::BondedSilent | GapState::Unprovisioned => {
                return Err(ReconnectError::NotConnectable);
            }
        }

        let challenge = if accessory.replay_assertion_challenge {
            accessory.replay_assertion_challenge = false;
            accessory
                .last_assertion_challenge
                .ok_or(ReconnectError::AssertionReplay)?
        } else {
            let challenge = nonce_from_counter(accessory.next_challenge);
            accessory.next_challenge += 1;
            challenge
        };
        if accessory.used_assertion_challenges.contains(&challenge) {
            return Err(ReconnectError::AssertionReplay);
        }

        let message = assertion_message(&challenge, accessory.identity);
        let assertion = sign(&host.vault_secret, &message);
        let Some(accessory_bond) = accessory
            .bonds
            .iter()
            .find(|bond| verify(&bond.public_key, &message, &assertion))
        else {
            return Err(ReconnectError::NoBond);
        };

        accessory.used_assertion_challenges.push(challenge);
        accessory.last_assertion_challenge = Some(challenge);

        let host_ltk = host
            .bonds
            .iter()
            .find(|bond| bond.id == accessory_bond.id)
            .map(|bond| bond.ltk)
            .unwrap_or_else(|| fixture_ltk(host.identity, accessory.identity));
        let presented_ltk = host.ltk_override.unwrap_or(host_ltk);
        if presented_ltk != accessory_bond.ltk {
            return Err(ReconnectError::EncryptionFailed);
        }
        return Ok(BondedLink {
            bond_passkey_id: Some(accessory_bond.id),
        });
    }

    let accessory_ordinary = accessory
        .ordinary_bonds
        .iter()
        .find(|bond| bond.peer_identity == host.identity)
        .ok_or(ReconnectError::NoBond)?;
    let host_ordinary = host
        .ordinary_bonds
        .iter()
        .find(|bond| bond.peer_identity == accessory.identity)
        .ok_or(ReconnectError::NoBond)?;
    let presented_ltk = host.ltk_override.unwrap_or(host_ordinary.ltk);
    if presented_ltk != accessory_ordinary.ltk {
        return Err(ReconnectError::EncryptionFailed);
    }
    Ok(BondedLink {
        bond_passkey_id: None,
    })
}
