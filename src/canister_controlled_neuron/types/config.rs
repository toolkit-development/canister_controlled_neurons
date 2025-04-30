use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use toolkit_utils::impl_storable_for;

impl_storable_for!(Config);

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct Config {
    pub governance_canister_id: Principal,
    pub sns_ledger_canister_id: Principal,
}

impl Config {
    pub fn new(governance_canister_id: Principal, sns_ledger_canister_id: Principal) -> Self {
        Self {
            governance_canister_id,
            sns_ledger_canister_id,
        }
    }
}
