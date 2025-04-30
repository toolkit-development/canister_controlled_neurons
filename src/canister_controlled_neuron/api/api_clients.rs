use ic_ledger_types::MAINNET_GOVERNANCE_CANISTER_ID;

use super::icp_governance_api::IcpGovernanceApi;
pub struct ApiClients;

impl ApiClients {
    pub fn icp_governance() -> IcpGovernanceApi {
        IcpGovernanceApi(MAINNET_GOVERNANCE_CANISTER_ID)
    }
}
