use candid::Principal;
use ic_ledger_types::{MAINNET_GOVERNANCE_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID};


use crate::{api::{canister_controlled_neuron_api::CanisterControlledNeuronApi, icp_ledger_api::IcpLedgerApi, icp_ledger_index_api::IcpLedgerIndexApi, icp_root_api::IcpRootApi}, types::service_canisters::{GovernanceCanisterId, RootCanisterId, ServiceCanisterId}};

use super::icp_governance_api::IcpGovernanceApi;

const MAINNET_LEDGER_INDEX_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";
pub struct ApiClients;

impl ApiClients {
    pub fn icp_governance() -> IcpGovernanceApi {
        IcpGovernanceApi(MAINNET_GOVERNANCE_CANISTER_ID)
    }

    pub fn sns_governance(governance_canister_id: GovernanceCanisterId) -> IcpGovernanceApi {
        IcpGovernanceApi(governance_canister_id)
    }

    pub fn icp_ledger() -> IcpLedgerApi {
        IcpLedgerApi(MAINNET_LEDGER_CANISTER_ID)
    }

    pub fn icp_ledger_index() -> IcpLedgerIndexApi {
        IcpLedgerIndexApi(Principal::from_text(MAINNET_LEDGER_INDEX_CANISTER_ID).unwrap())
    }

    pub fn canister_controlled_neuron(service_canister_id: ServiceCanisterId) -> CanisterControlledNeuronApi {
        CanisterControlledNeuronApi(service_canister_id)
    }

    pub fn sns_root(root_canister_id: RootCanisterId) -> IcpRootApi {
        IcpRootApi(root_canister_id)
    }
}
