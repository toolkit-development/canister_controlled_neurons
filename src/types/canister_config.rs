use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

use crate::impl_storable_for;

impl_storable_for!(CanisterConfig);

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct CanisterConfig {
    pub owners: Vec<Principal>,
}

impl CanisterConfig {
    pub fn new(owners: Vec<Principal>) -> Self {
        Self { owners }
    }
}
