use ic_cdk::query;
use toolkit_utils::result::CanisterResult;

use crate::{
    logic::sns_neuron_logic::SNSNeuronLogic,
    types::{
        sns_chain_proposals::SnsChainProposalsResponse,
        sns_neuron_reference::SnsNeuronReferenceResponse,
    },
};

#[query]
pub fn get_sns_neuron_references() -> CanisterResult<Vec<SnsNeuronReferenceResponse>> {
    SNSNeuronLogic::get_neurons()
}

#[query]
pub async fn get_sns_chain_proposals(id: u64) -> CanisterResult<SnsChainProposalsResponse> {
    SNSNeuronLogic::get_sns_chain_proposals(id).await
}

// #[update]
// pub async fn create_chain_proposals(
//     neuron_id: Vec<u8>,
//     proposals: Vec<PostSnsChainProposal>,
//     start_chain: bool,
// ) -> CanisterResult<SnsChainProposalsResponse> {
//     SNSNeuronLogic::create_chain_proposals(neuron_id, proposals, start_chain).await
// }

// #[update]
// pub async fn start_chain(id: u64) -> CanisterResult<SnsChainProposalsResponse> {
//     SNSNeuronLogic::start_chain(id).await
// }

// #[update]
// pub async fn execute_next_proposal(id: u64) -> CanisterResult<SnsChainProposalsResponse> {
//     SNSNeuronLogic::submit_next_proposal(id).await
// }
