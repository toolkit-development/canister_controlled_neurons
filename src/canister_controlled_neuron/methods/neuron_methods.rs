use ic_cdk::{api::canister_self, query, update};
use ic_ledger_types::{
    account_balance, AccountBalanceArgs, AccountIdentifier, DEFAULT_SUBACCOUNT,
    MAINNET_LEDGER_CANISTER_ID,
};
use toolkit_utils::{api_error::ApiError, result::CanisterResult};

use crate::{
    api::icp_governance_api::Neuron as GovNeuron,
    logic::{config_logic::ConfigLogic, neuron_logic::NeuronLogic},
    misc::guards::is_governance_canister,
    types::{
        modules::{Module, ModuleResponse},
        neuron_reference::NeuronReferenceResponse,
    },
};

#[query]
pub fn get_neuron_references() -> CanisterResult<Vec<NeuronReferenceResponse>> {
    NeuronLogic::get_neurons()
}

#[update]
pub async fn get_full_neuron(subaccount: [u8; 32]) -> CanisterResult<GovNeuron> {
    NeuronLogic::get_full_neuron(subaccount).await
}

#[update]
pub async fn set_module(module: Module) -> CanisterResult<ModuleResponse> {
    is_governance_canister()?;
    ConfigLogic::set_module(module).await
}

#[update]
pub async fn validate_set_module(module: Module) -> Result<String, String> {
    is_governance_canister().map_err(|e| e.to_string())?;
    ConfigLogic::validate_set_module(module)
        .await
        .map_err(|e| e.to_string())
}

#[update]
pub async fn get_canister_icp_balance() -> CanisterResult<u64> {
    let balance = account_balance(
        MAINNET_LEDGER_CANISTER_ID,
        &AccountBalanceArgs {
            account: AccountIdentifier::new(&canister_self(), &DEFAULT_SUBACCOUNT),
        },
    )
    .await
    .map_err(|e| ApiError::external_service_error(e.to_string().as_str()))?;
    Ok(balance.e8s())
}
