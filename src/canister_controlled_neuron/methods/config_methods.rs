use ic_cdk::{init, query};
use toolkit_utils::{cell::CellStorage, result::CanisterResult, storage::StorageQueryable};

use crate::{
    logic::config_logic::ConfigLogic,
    storage::{config_storage::config_store, log_storage::LogStore},
    types::config::Config,
};

#[init]
pub fn init(canisters: Config) {
    let _ = ConfigLogic::init(
        canisters.governance_canister_id,
        canisters.sns_ledger_canister_id,
    );
}

#[query]
pub fn get_config() -> CanisterResult<Config> {
    config_store().get()
}

#[query]
pub fn get_logs() -> Vec<String> {
    LogStore::get_all()
        .into_iter()
        .map(|(_, log)| log)
        .collect()
}
