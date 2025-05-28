use std::collections::HashMap;

use candid::CandidType;
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};
use toolkit_utils::{api_error::ApiError, impl_storable_for, result::CanisterResult};

use crate::{
    api::{
        api_clients::ApiClients,
        sns_governance_api::{GetProposal, Proposal, ProposalId, Result1},
    },
    logic::sns_neuron_logic::SNSNeuronLogic,
};

impl_storable_for!(SnsChainProposals);

#[derive(Debug, CandidType, Serialize, Deserialize, Clone, Default)]
pub struct SnsChainProposals {
    pub neuron_id: Vec<u8>,
    pub proposals: HashMap<u64, SnsChainProposal>,
    pub active_proposal_id: Option<u64>,
    pub current_index: u64,
    pub created_at: u64,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone, Default)]
pub struct SnsChainProposalsResponse {
    pub id: u64,
    pub neuron_id: Vec<u8>,
    pub proposals: Vec<SnsChainProposalResponse>,
    pub current_index: u64,
    pub active_proposal_id: Option<u64>,
}

impl SnsChainProposals {
    pub fn new(neuron_id: Vec<u8>, proposals: Vec<PostSnsChainProposal>) -> Self {
        SnsChainProposals {
            current_index: 0,
            neuron_id,
            proposals: proposals
                .into_iter()
                .map(|proposal| (proposal.index, SnsChainProposal::new(proposal)))
                .collect(),
            created_at: time(),
            active_proposal_id: None,
        }
    }

    pub fn get_proposal(&self, index: &u64) -> CanisterResult<SnsChainProposal> {
        self.proposals
            .get(index)
            .cloned()
            .ok_or(ApiError::not_found("Proposal not found"))
    }

    pub async fn start_chain(&mut self) -> CanisterResult<SnsChainProposals> {
        let proposal = self
            .proposals
            .get_mut(&self.current_index)
            .ok_or(ApiError::not_found("Proposal not found"))?;

        let active_proposal_id = proposal.create_proposal(self.neuron_id.clone()).await?;
        proposal.proposal_id = Some(active_proposal_id);

        self.active_proposal_id = Some(active_proposal_id);

        Ok(self.clone())
    }

    pub async fn submit_next_proposal(&mut self) -> CanisterResult<()> {
        let current_index = self.current_index;

        // Step 1: Ensure the current (previous) proposal exists
        let previous_proposal = self
            .proposals
            .get(&current_index)
            .ok_or_else(|| ApiError::not_found("Current proposal not found"))?
            .clone();

        // Step 2: Ensure it has a proposal_id
        let active_proposal_id = previous_proposal
            .get_proposal_id()
            .ok_or_else(|| ApiError::bad_request("Current proposal has no proposal_id"))?;

        // Step 3: Check execution status
        let (is_executed, _) = get_latest_proposal_response(active_proposal_id).await?;
        if !is_executed {
            return Err(ApiError::bad_request("Current proposal not yet executed"));
        }

        // Step 4: Advance to the next proposal
        let next_index = current_index + 1;

        if next_index >= self.proposals.len() as u64 {
            ic_cdk::println!(
                "🎉 Chain complete: no next proposal at index {}",
                next_index
            );
            return Ok(()); // gracefully exit
        }

        let next_proposal = self
            .proposals
            .get_mut(&next_index)
            .ok_or_else(|| ApiError::not_found("Next proposal not found"))?;

        // Step 5: Submit the next proposal
        let next_proposal_id = next_proposal
            .create_proposal(self.neuron_id.clone())
            .await?;

        // Step 6: Update state
        next_proposal.proposal_id = Some(next_proposal_id);
        self.active_proposal_id = Some(next_proposal_id);
        self.current_index = next_index;

        Ok(())
    }

    pub fn to_response(&self, id: u64) -> SnsChainProposalsResponse {
        SnsChainProposalsResponse {
            id,
            neuron_id: self.neuron_id.clone(),
            // is not sorted by index
            proposals: self
                .proposals
                .iter()
                .map(|(index, proposal)| proposal.to_response(*index))
                .collect(),
            current_index: self.current_index,
            active_proposal_id: self.active_proposal_id,
        }
    }
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SnsChainProposal {
    pub proposal_id: Option<u64>,
    pub proposal: Proposal,
    pub created_at: u64,
}

impl SnsChainProposal {
    pub fn new(proposal: PostSnsChainProposal) -> Self {
        SnsChainProposal {
            proposal_id: None,
            proposal: proposal.proposal,
            created_at: time(),
        }
    }

    pub fn get_proposal_id(&self) -> Option<u64> {
        self.proposal_id
    }

    pub fn get_proposal(&self) -> &Proposal {
        &self.proposal
    }

    pub fn get_created_at(&self) -> u64 {
        self.created_at
    }

    pub async fn create_proposal(&mut self, neuron_id: Vec<u8>) -> CanisterResult<u64> {
        let create_proposal_response =
            SNSNeuronLogic::create_proposal(neuron_id, self.proposal.clone()).await?;

        Ok(create_proposal_response.proposal_id.unwrap().id)
    }

    pub fn to_response(&self, index: u64) -> SnsChainProposalResponse {
        SnsChainProposalResponse {
            index,
            proposal: self.proposal.clone(),
            proposal_id: self.proposal_id,
        }
    }
}

pub async fn get_latest_proposal_response(proposal_id: u64) -> CanisterResult<(bool, Result1)> {
    let client = ApiClients::sns_governance();

    let args = GetProposal {
        proposal_id: Some(ProposalId { id: proposal_id }),
    };

    let (proposal,) = client
        .get_proposal(args)
        .await
        .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

    let proposal_data = proposal
        .result
        .ok_or(ApiError::not_found("Proposal not found"))?;

    let is_executed = match proposal_data {
        Result1::Proposal(ref proposal) => proposal.executed_timestamp_seconds > 0,
        _ => false,
    };

    Ok((is_executed, proposal_data))
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct PostSnsChainProposal {
    pub index: u64,
    pub proposal: Proposal,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SnsChainProposalResponse {
    pub index: u64,
    pub proposal: Proposal,
    pub proposal_id: Option<u64>,
}
