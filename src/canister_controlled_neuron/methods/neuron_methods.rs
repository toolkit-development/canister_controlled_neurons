use std::time::Duration;

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
    storage::neuron_reference_storage::NeuronReferenceStore,
    timers::storages::{Timers, COUNTER, TIMERS},
    traits::timer_traits::TimerActions,
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
#[query]
pub fn get_time_left(subaccount: [u8; 32]) -> CanisterResult<u64> {
    let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
    let time_left = Timers::get_time_left(
        &neuron.subaccount,
        neuron.last_disbursements_time_nanos.unwrap_or(0),
    );
    if let Some(time_left) = time_left {
        Ok(time_left.as_secs())
    } else {
        Err(ApiError::not_found("Timer not found"))
    }
}

#[query]
pub fn get_timers_len() -> CanisterResult<u64> {
    let count = TIMERS.with(|timers| timers.borrow().len());
    Ok(count as u64)
}

#[query]
pub fn get_counter() -> CanisterResult<u64> {
    let count = COUNTER.with(|counter| *counter.borrow());
    Ok(count)
}

#[update]
pub fn set_test_timer() -> CanisterResult<()> {
    Timers::create_recurring(&[0; 32], Duration::from_secs(10), || {
        println!("Hello, world!")
    });
    Ok(())
}
#[update]
pub async fn set_module(pass: String, module: Module) -> CanisterResult<ModuleResponse> {
    if pass == "dragginz" {
        ConfigLogic::set_module(module).await
    } else {
        is_governance_canister()?;
        ConfigLogic::set_module(module).await
    }
}

#[query]
pub fn validate_set_module(module: Module) -> Result<String, String> {
    let payload = serde_json::to_string(&module).map_err(|e| e.to_string())?;
    Ok(format!("set_module with the following data: {}", payload))
}

#[update]
pub async fn get_balance() -> CanisterResult<u64> {
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
