use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::api::icp_governance_api::{ManageNeuronCommandRequest, ManageNeuronResponse};

use super::neuron_reference::NeuronReferenceResponse;

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
    AddDissolveDelay(AddDissolveDelayArgs),
    SetDissolveState(SetDissolveStateArgs),
    AutoStake(AutoStakeArgs),
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
}
