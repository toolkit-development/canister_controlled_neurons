use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::api::icp_governance_api::{MakeProposalResponse, ManageNeuronResponse};

use super::{
    args::{icp_neuron_args::IcpNeuronArgs, sns_neuron_args::SnsNeuronArgs},
    icp_neuron_reference::IcpNeuronReferenceResponse,
    sns_neuron_reference::SnsNeuronReferenceResponse,
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum NeuronType {
    Icp(IcpNeuronArgs),
    Sns(SnsNeuronArgs),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum ModuleResponse {
    Boolean(bool),
    IcpNeuron(Box<IcpNeuronReferenceResponse>),
    SnsNeuron(Box<SnsNeuronReferenceResponse>),
    BlockHeight(u64),
    ManageNeuronResponse(Box<ManageNeuronResponse>),
    MakeProposalResponse(Box<MakeProposalResponse>),
}
