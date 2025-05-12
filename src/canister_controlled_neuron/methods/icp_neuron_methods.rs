use ic_cdk::{query, update};
use toolkit_utils::result::CanisterResult;

use crate::{
    api::icp_governance_api::Neuron as GovNeuron,
    logic::icp_neuron_logic::NeuronLogic,
    types::{icp_neuron_reference::IcpNeuronReferenceResponse, modules::IcpNeuronIdentifier},
};

#[query]
pub fn get_neuron_references() -> CanisterResult<Vec<IcpNeuronReferenceResponse>> {
    NeuronLogic::get_neurons()
}

#[update]
pub async fn get_full_neuron(identifier: IcpNeuronIdentifier) -> CanisterResult<GovNeuron> {
    NeuronLogic::get_full_neuron(&identifier).await
}
