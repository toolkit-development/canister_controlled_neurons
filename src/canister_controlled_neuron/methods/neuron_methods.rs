use ic_cdk::{query, update};
use toolkit_utils::result::CanisterResult;

use crate::{
    api::icp_governance_api::Neuron as GovNeuron,
    logic::neuron_logic::NeuronLogic,
    misc::guards::is_governance_canister,
    types::{
        modules::{ModuleResponse, NeuronType},
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
pub async fn tk_service_manage_neuron(args: NeuronType) -> CanisterResult<ModuleResponse> {
    is_governance_canister()?;
    NeuronLogic::tk_service_manage_neuron(args).await
}

#[update]
pub async fn tk_service_validate_manage_neuron(args: NeuronType) -> Result<String, String> {
    is_governance_canister().map_err(|e| e.to_string())?;
    NeuronLogic::tk_service_validate_manage_neuron(args)
        .await
        .map_err(|e| e.to_string())
}
