use ic_ledger_types::MAINNET_GOVERNANCE_CANISTER_ID;
use toolkit_utils::cell::CellStorage;

use crate::storage::config_storage::config_store;

use super::{
    icp_governance_api::IcpGovernanceApi, sns_governance_api::SnsGovernanceApi,
    sns_ledger_api::SnsLedgerApi,
};
pub struct ApiClients;

impl ApiClients {
    pub fn icp_governance() -> IcpGovernanceApi {
        IcpGovernanceApi(MAINNET_GOVERNANCE_CANISTER_ID)
    }

    pub fn sns_governance() -> SnsGovernanceApi {
        SnsGovernanceApi(
            config_store()
                .get()
                .expect("Config not found")
                .governance_canister_id,
        )
    }

    pub fn sns_ledger() -> SnsLedgerApi {
        SnsLedgerApi(
            config_store()
                .get()
                .expect("Config not found")
                .sns_ledger_canister_id,
        )
    }
}
