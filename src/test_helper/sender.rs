use candid::Principal;

use crate::{context::OWNER_PRINCIPAL, utils::generate_principal};

pub enum Sender {
    Owner,
    Other(Principal),
    Unauthorized,
    Anonymous,
}

impl Sender {
    pub fn principal(&self) -> Principal {
        match self {
            Sender::Owner => Principal::from_text(OWNER_PRINCIPAL).unwrap(),
            Sender::Other(principal) => *principal,
            Sender::Unauthorized => generate_principal(),
            Sender::Anonymous => Principal::anonymous(),
        }
    }
}
