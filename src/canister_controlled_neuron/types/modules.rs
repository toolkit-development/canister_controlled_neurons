use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::api::icp_governance_api::{ManageNeuronCommandRequest, ManageNeuronResponse};

use super::neuron_reference::{
    NeuronReferenceResponse, PostCustomTargetmaturityDisbursement,
    PostTreasuryTargetmaturityDisbursement,
};

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum Module {
    TreasuryManagement(TreasuryManagementModuleType),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum TreasuryManagementModuleType {
    Neuron(NeuronType),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum NeuronType {
    Icp(IcpNeuronArgs),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum IcpNeuronArgs {
    Create(CreateNeuronArgs),
    TopUp(TopUpNeuronArgs),
    Command(CommandNeuronArgs),
    DisburseMaturity(DisburseMaturityArgs),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct DisburseMaturityArgs {
    pub subaccount: [u8; 32],
    pub args: DisburseMaturityType,
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum DisburseMaturityType {
    Stop,
    StartCustomTarget(PostCustomTargetmaturityDisbursement),
    StartTreasuryTarget(PostTreasuryTargetmaturityDisbursement),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct CreateNeuronArgs {
    pub amount_e8s: u64,
    pub maturity_disbursement_target: Option<PostCustomTargetmaturityDisbursement>,
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
}
