use ic_cdk::query;
use toolkit_utils::{cell::CellStorage, result::CanisterResult};

use crate::{storage::config_storage::config_store, types::config::Config};

#[query]
pub fn get_config() -> CanisterResult<Config> {
    config_store().get()
}
