use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum SnsNeuronIdentifier {
    Subaccount([u8; 32]),
    NeuronId(Vec<u8>),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub enum SnsNeuronArgs {
    Create(CreateNeuronArgs),
}

#[derive(Debug, CandidType, Serialize, Deserialize, Clone)]
pub struct CreateNeuronArgs {
    pub amount_e8s: u64,
}
