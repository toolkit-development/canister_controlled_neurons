use candid::Principal;
use ic_cdk::{api::canister_self, init, query, update};
use ic_ledger_types::{
    account_balance, AccountBalanceArgs, AccountIdentifier, DEFAULT_SUBACCOUNT,
    MAINNET_LEDGER_CANISTER_ID,
};

use crate::{
    api::governance_api::{ListNeuronsResponse, NeuronInfo},
    logic::{config_logic::ConfigLogic, neuron_logic::NeuronLogic},
    misc::guards::is_authorized,
    types::{api_error::ApiError, neuron::NeuronReferenceResponse, result::CanisterResult},
};

#[init]
pub fn init(owners: Vec<Principal>) {
    ConfigLogic::init(owners).expect("Failed to initialize config");
}

#[query]
pub fn get_neurons() -> CanisterResult<Vec<NeuronReferenceResponse>> {
    NeuronLogic::get_neurons()
}

#[update]
pub async fn top_up_neuron_by_subaccount(id: u64, amount_e8s: u64) -> CanisterResult<u64> {
    is_authorized()?;
    NeuronLogic::top_up_neuron_by_subaccount(id, amount_e8s).await
}

#[update]
pub async fn list_neurons() -> CanisterResult<ListNeuronsResponse> {
    NeuronLogic::list_controlled_neurons().await
}

#[update]
pub async fn create_neuron(amount_e8s: u64) -> CanisterResult<NeuronReferenceResponse> {
    is_authorized()?;
    NeuronLogic::create_neuron(amount_e8s).await
}

#[update]
pub async fn claim_or_refresh_neuron(id: u64) -> CanisterResult<NeuronReferenceResponse> {
    is_authorized()?;
    NeuronLogic::claim_or_refresh_neuron(id).await
}

#[update]
pub async fn get_neuron_info(id: u64) -> CanisterResult<NeuronInfo> {
    NeuronLogic::get_neuron_info(id).await
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
