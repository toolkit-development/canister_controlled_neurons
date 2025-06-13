use candid::{CandidType};
use serde::{Deserialize, Serialize};
use toolkit_utils::impl_storable_for;

use crate::types::service_canisters::{LedgerCanisterId, RootCanisterId, ServiceCanisterModules};

pub type PurchaseModulesProposalId = u64;

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct PurchaseModulesProposalData {
    pub proposal_id: PurchaseModulesProposalId,
    pub modules: ServiceCanisterModules,
    pub amount: u64,
    pub memo: u64,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct ServiceData {
    pub proposals: Vec<PurchaseModulesProposalData>,
    pub root_canister_id: RootCanisterId,
    pub ledger_canister_id: LedgerCanisterId,
}

impl_storable_for!(ServiceData);
