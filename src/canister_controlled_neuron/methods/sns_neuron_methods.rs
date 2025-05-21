use ic_cdk::query;
use toolkit_utils::result::CanisterResult;

use crate::{
    logic::sns_neuron_logic::SNSNeuronLogic,
    types::sns_neuron_reference::SnsNeuronReferenceResponse,
};

#[query]
pub fn get_sns_neuron_references() -> CanisterResult<Vec<SnsNeuronReferenceResponse>> {
    SNSNeuronLogic::get_neurons()
}

// #[update]
// pub async fn get_full_neuron(identifier: IcpNeuronIdentifier) -> CanisterResult<GovNeuron> {
//     SNSNeuronLogic::get_full_neuron(&identifier).await
// }
