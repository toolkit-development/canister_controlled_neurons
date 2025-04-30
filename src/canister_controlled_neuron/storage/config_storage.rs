use toolkit_utils::{cell::CellStorage, GenericCellStorage};

use crate::types::config::Config;

use super::storages::CONFIG;

pub fn config_store() -> impl CellStorage<Config> {
    GenericCellStorage::new("config", &CONFIG)
}
