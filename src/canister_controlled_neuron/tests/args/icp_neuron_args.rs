use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{
    api::icp_governance_api::{MakeProposalRequest, ManageNeuronCommandRequest},
    types::topic::Topic,
};

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum IcpNeuronIdentifier {
    Subaccount([u8; 32]),
    NeuronId(u64),
}

#[allow(clippy::large_enum_variant)]
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
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SetFollowingArgs {
    pub identifier: IcpNeuronIdentifier,
    pub following: Vec<FollowingArgs>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct FollowingArgs {
    pub topic: Topic,
    pub followees: Vec<u64>,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct DisburseArgs {
    pub identifier: IcpNeuronIdentifier,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct VoteArgs {
    pub identifier: IcpNeuronIdentifier,
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
    pub identifier: IcpNeuronIdentifier,
    pub proposal: MakeProposalRequest,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SpawnArgs {
    pub identifier: IcpNeuronIdentifier,
    pub start_dissolving: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct AutoStakeArgs {
    pub identifier: IcpNeuronIdentifier,
    pub auto_stake: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct SetDissolveStateArgs {
    pub identifier: IcpNeuronIdentifier,
    pub start_dissolving: bool,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct AddDissolveDelayArgs {
    pub identifier: IcpNeuronIdentifier,
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
    pub identifier: IcpNeuronIdentifier,
    pub amount_e8s: u64,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct CommandNeuronArgs {
    pub identifier: IcpNeuronIdentifier,
    pub command: ManageNeuronCommandRequest,
}
