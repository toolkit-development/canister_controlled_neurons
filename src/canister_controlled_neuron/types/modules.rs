use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::api::icp_governance_api::{
    MakeProposalRequest, MakeProposalResponse, ManageNeuronCommandRequest, ManageNeuronResponse,
};

use super::{neuron_reference::NeuronReferenceResponse, topic::Topic};

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum NeuronType {
    Icp(IcpNeuronArgs),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum IcpNeuronArgs {
    Create(CreateNeuronArgs),
    TopUp(TopUpNeuronArgs),
    AddDissolveDelay(AddDissolveDelayArgs),
    SetDissolveState(SetDissolveStateArgs),
    AutoStake(AutoStakeArgs),
    Spawn(SpawnArgs),
    CreateProposal(CreateProposalArgs),
    Vote(VoteArgs),
    Disburse(DisburseArgs),
    SetFollowing(SetFollowingArgs),
    Command(CommandNeuronArgs),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SetFollowingArgs {
    pub subaccount: [u8; 32],
    pub following: Vec<FollowingArgs>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct FollowingArgs {
    pub topic: Topic,
    pub followees: Vec<u64>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct DisburseArgs {
    pub subaccount: [u8; 32],
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct VoteArgs {
    pub subaccount: [u8; 32],
    pub proposal_id: u64,
    pub vote: Vote,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum Vote {
    Approve,
    Reject,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct CreateProposalArgs {
    pub subaccount: [u8; 32],
    pub proposal: MakeProposalRequest,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SpawnArgs {
    pub parent_subaccount: [u8; 32],
    pub start_dissolving: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct AutoStakeArgs {
    pub subaccount: [u8; 32],
    pub auto_stake: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SetDissolveStateArgs {
    pub subaccount: [u8; 32],
    pub start_dissolving: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct AddDissolveDelayArgs {
    pub subaccount: [u8; 32],
    pub dissolve_delay_seconds: u64,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct CreateNeuronArgs {
    pub amount_e8s: u64,
    pub auto_stake: Option<bool>,
    pub dissolve_delay_seconds: Option<u64>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct TopUpNeuronArgs {
    pub subaccount: [u8; 32],
    pub amount_e8s: u64,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct CommandNeuronArgs {
    pub subaccount: [u8; 32],
    pub command: ManageNeuronCommandRequest,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum ModuleResponse {
    Boolean(bool),
    Neuron(Box<NeuronReferenceResponse>),
    BlockHeight(u64),
    ManageNeuronResponse(Box<ManageNeuronResponse>),
    MakeProposalResponse(Box<MakeProposalResponse>),
}
