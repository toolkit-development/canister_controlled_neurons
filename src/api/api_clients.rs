use ic_ledger_types::MAINNET_GOVERNANCE_CANISTER_ID;

use super::governance_api::GovernanceApi;
pub struct ApiClients;

impl ApiClients {
    pub fn governance() -> GovernanceApi {
        GovernanceApi(MAINNET_GOVERNANCE_CANISTER_ID)
    }
}
