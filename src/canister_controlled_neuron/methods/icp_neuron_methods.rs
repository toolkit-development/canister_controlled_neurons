use ic_cdk::{query, update};
use toolkit_utils::result::CanisterResult;

use crate::{
    api::icp_governance_api::Neuron as GovNeuron,
    logic::icp_neuron_logic::ICPNeuronLogic,
    types::{
        args::icp_neuron_args::IcpNeuronIdentifier,
        icp_neuron_reference::IcpNeuronReferenceResponse,
    },
};

#[query]
pub fn get_neuron_references() -> CanisterResult<Vec<IcpNeuronReferenceResponse>> {
    ICPNeuronLogic::get_neurons()
}

#[update]
pub async fn get_full_neuron(identifier: IcpNeuronIdentifier) -> CanisterResult<GovNeuron> {
    ICPNeuronLogic::get_full_neuron(&identifier).await
}
