use crate::{
    helpers::cell_storage::GenericCellStorage, traits::cell::CellStorage,
    types::canister_config::CanisterConfig,
};

use super::storages::CONFIG;

pub fn config_store() -> impl CellStorage<CanisterConfig> {
    GenericCellStorage::new("config", &CONFIG)
}
