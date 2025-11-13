use ic_cdk::{query, update};
use toolkit_utils::result::CanisterResult;

use crate::{
    logic::{proposal_logic::ProposalLogic},
    types::{proposal::PurchaseModulesProposalData, service_canisters::{GovernanceCanisterId, RootCanisterId, ServiceCanisterModules}},
};


#[query]
pub fn get_proposals(governance_canister_id: GovernanceCanisterId) -> CanisterResult<Vec<PurchaseModulesProposalData>> {
    ProposalLogic::get_proposals(governance_canister_id)
}

#[update]
pub async fn propose_purchase_modules(root_canister_id: RootCanisterId, proposal_id: u64, modules: ServiceCanisterModules) -> CanisterResult<()> {
    // TODO: check that the caller is an authorized principal
    ProposalLogic::propose_purchase_modules(root_canister_id, proposal_id, modules).await
}