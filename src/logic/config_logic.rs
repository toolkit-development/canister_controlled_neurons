use candid::Principal;

use crate::{
    storage::config_storage::config_store,
    traits::cell::CellStorage,
    types::{canister_config::CanisterConfig, result::CanisterResult},
};

pub struct ConfigLogic;

impl ConfigLogic {
    pub fn init(owners: Vec<Principal>) -> CanisterResult<CanisterConfig> {
        config_store().set(CanisterConfig::new(owners))
    }

    pub fn get_config() -> CanisterResult<CanisterConfig> {
        config_store().get()
    }
}
