use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use toolkit_utils::impl_storable_for;

pub type RootCanisterId = Principal;
pub type LedgerCanisterId = Principal;
pub type GovernanceCanisterId = Principal;
pub type ServiceCanisterId = Principal;

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct ServiceCanisterModules {
    pub pay_roll: bool,
    pub bookkeeping: bool,
    pub notification: bool,
    pub treasury_management: bool,
    pub governance: bool,
    pub canister: bool
}

impl_storable_for!(ServiceCanisterModules);