use candid::Principal;
use toolkit_utils::{cell::CellStorage, result::CanisterResult};

use crate::{storage::config_storage::config_store, types::config::Config};

pub struct ConfigLogic;

impl ConfigLogic {
    pub fn init(
        governance_canister_id: Principal,
        sns_ledger_canister_id: Principal,
    ) -> CanisterResult<Config> {
        config_store().set(Config::new(governance_canister_id, sns_ledger_canister_id))
    }

    pub fn get_config() -> CanisterResult<Config> {
        config_store().get()
    }
}
