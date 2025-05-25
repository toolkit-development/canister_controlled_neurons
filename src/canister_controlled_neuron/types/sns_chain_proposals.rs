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
    pub current_index: u64,
    pub created_at: u64,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone, Default)]
pub struct SnsChainProposalsResponse {
    pub id: u64,
    pub neuron_id: Vec<u8>,
    pub proposals: Vec<SnsChainProposalResponse>,
    pub current_index: u64,
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
        }
    }

    pub fn get_proposal(&self, index: &u64) -> CanisterResult<SnsChainProposal> {
        self.proposals
            .get(index)
            .cloned()
            .ok_or(ApiError::not_found("Proposal not found"))
    }

    pub async fn start_chain(&mut self) -> CanisterResult<()> {
        let proposal = self
            .proposals
            .get_mut(&self.current_index)
            .ok_or(ApiError::not_found("Proposal not found"))?;

        proposal.create_proposal(self.neuron_id.clone()).await?;
        self.current_index += 1;

        Ok(())
    }

    pub async fn execute_next_proposal(&mut self) -> CanisterResult<SnsChainProposal> {
        let current_proposal = self
            .proposals
            .get_mut(&self.current_index)
            .ok_or(ApiError::not_found("Proposal not found"))?;

        current_proposal
            .create_proposal(self.neuron_id.clone())
            .await?;

        self.current_index += 1;

        Ok(current_proposal.clone())
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
        }
    }
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SnsChainProposal {
    pub proposal_id: Option<u64>,
    pub proposal: Proposal,
    pub proposal_response: Option<Result1>,
    pub created_at: u64,
}

impl SnsChainProposal {
    pub fn new(proposal: PostSnsChainProposal) -> Self {
        SnsChainProposal {
            proposal_id: None,
            proposal: proposal.proposal,
            proposal_response: None,
            created_at: time(),
        }
    }

    pub fn get_proposal_id(&self) -> Option<u64> {
        self.proposal_id
    }

    pub fn get_proposal(&self) -> &Proposal {
        &self.proposal
    }

    pub fn get_proposal_response(&self) -> Option<&Result1> {
        self.proposal_response.as_ref()
    }

    pub fn get_created_at(&self) -> u64 {
        self.created_at
    }

    pub async fn is_proposal_executed(&mut self) -> CanisterResult<bool> {
        let proposal_id = self.get_proposal_id().ok_or(ApiError::not_found(
            "Proposal ID is not set to fetch response",
        ))?;

        if self.proposal_response.is_none() {
            self.fetch_and_set_proposal_response(proposal_id).await?;
        }

        match self.proposal_response {
            Some(Result1::Proposal(ref proposal)) => Ok(proposal.executed_timestamp_seconds > 0),
            _ => Ok(false),
        }
    }

    pub async fn fetch_and_set_proposal_response(
        &mut self,
        proposal_id: u64,
    ) -> CanisterResult<Result1> {
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

        self.proposal_response = Some(proposal_data.clone());
        Ok(proposal_data)
    }

    pub async fn create_proposal(&mut self, neuron_id: Vec<u8>) -> CanisterResult<u64> {
        let create_proposal_response =
            SNSNeuronLogic::create_proposal(neuron_id, self.proposal.clone()).await?;

        let proposal_id = create_proposal_response.proposal_id.unwrap().id;
        self.fetch_and_set_proposal_response(proposal_id).await?;
        self.proposal_id = Some(proposal_id);

        Ok(proposal_id)
    }

    pub fn to_response(&self, index: u64) -> SnsChainProposalResponse {
        SnsChainProposalResponse {
            index,
            proposal: self.proposal.clone(),
            proposal_id: self.proposal_id,
            proposal_response: self.proposal_response.clone(),
        }
    }
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
    pub proposal_response: Option<Result1>,
}
