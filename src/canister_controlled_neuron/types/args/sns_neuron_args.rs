use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::api::sns_governance_api::{Proposal, Topic};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum SnsNeuronArgs {
    Create(CreateSnsNeuronArgs),
    TopUp(TopUpSnsNeuronArgs),
    AddDissolveDelay(AddSnsNeuronDissolveDelayArgs),
    SetDissolveState(SetSnsNeuronDissolveStateArgs),
    AutoStake(AutoStakeSnsNeuronArgs),
    Spawn(SpawnSnsNeuronArgs),
    CreateProposal(CreateSnsNeuronProposalArgs),
    Vote(VoteSnsNeuronArgs),
    Disburse(DisburseSnsNeuronArgs),
    SetFollowing(SetSnsNeuronFollowingArgs),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct CreateSnsNeuronArgs {
    pub amount_e8s: u64,
    pub auto_stake: Option<bool>,
    pub dissolve_delay_seconds: Option<u64>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct TopUpSnsNeuronArgs {
    pub neuron_id: Vec<u8>,
    pub amount_e8s: u64,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SetSnsNeuronFollowingArgs {
    pub neuron_id: Vec<u8>,
    pub following: Vec<SnsNeuronFollowingArgs>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SnsNeuronFollowingArgs {
    pub topic: Topic,
    pub followees: Vec<Vec<u8>>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct DisburseSnsNeuronArgs {
    pub neuron_id: Vec<u8>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct VoteSnsNeuronArgs {
    pub neuron_id: Vec<u8>,
    pub proposal_id: u64,
    pub vote: SnsNeuronVote,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum SnsNeuronVote {
    Approve,
    Reject,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct CreateSnsNeuronProposalArgs {
    pub neuron_id: Vec<u8>,
    pub proposal: Proposal,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SpawnSnsNeuronArgs {
    pub neuron_id: Vec<u8>,
    pub start_dissolving: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct AutoStakeSnsNeuronArgs {
    pub neuron_id: Vec<u8>,
    pub auto_stake: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SetSnsNeuronDissolveStateArgs {
    pub neuron_id: Vec<u8>,
    pub start_dissolving: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct AddSnsNeuronDissolveDelayArgs {
    pub neuron_id: Vec<u8>,
    pub dissolve_delay_seconds: u64,
}
