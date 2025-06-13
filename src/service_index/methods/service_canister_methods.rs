use ic_cdk::{api::msg_caller, query, update};
use toolkit_utils::result::CanisterResult;

use crate::{
    logic::service_canisters_logic::ServiceCanistersLogic,
    misc::utils::check_authorized_principal,
    types::service_canisters::{GovernanceCanisterId, RootCanisterId, ServiceCanisterId, ServiceCanisterModules},
};

#[query]
pub fn get_service_canisters() -> CanisterResult<Vec<ServiceCanisterId>> {
    ServiceCanistersLogic::get_service_canisters()
}

#[update]
pub fn set_service_canisters(
    governance_canister_id: GovernanceCanisterId,
    service_canister_id: ServiceCanisterId,
) -> CanisterResult<()> {
    check_authorized_principal(msg_caller())?;
    ServiceCanistersLogic::set_service_canisters(governance_canister_id, service_canister_id);
    Ok(())
}

#[update]
pub async fn propose_purchase_modules(root_canister_id: RootCanisterId, proposal_id: u64, modules: ServiceCanisterModules) -> CanisterResult<()> {
    // TODO: check that the caller is an authorized principal
    ServiceCanistersLogic::propose_purchase_modules(root_canister_id, proposal_id, modules).await
}

#[update]
pub async fn activate_modules(
    governance_canister_id: GovernanceCanisterId,
) -> CanisterResult<()> {
    // TODO: add some kind of locking mechanism here for each governance canister id
    ServiceCanistersLogic::activate_modules(governance_canister_id).await
}

