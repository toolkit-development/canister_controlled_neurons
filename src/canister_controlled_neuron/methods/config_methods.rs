use ic_cdk::query;
use toolkit_utils::{cell::CellStorage, result::CanisterResult, storage::StorageQueryable};

use crate::{
    storage::{config_storage::config_store, log_storage::LogStore},
    types::config::Config,
};

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
