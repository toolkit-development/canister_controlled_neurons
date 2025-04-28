use ic_stable_structures::memory_manager::MemoryId;

use crate::{
    helpers::storage_init::{init_btree, init_cell, init_memory_manager, MemoryManagerStorage},
    traits::{cell::CellStorageRef, storage::StorageRef},
    types::{canister_config::CanisterConfig, neuron::NeuronReference},
};

pub static CONFIG_MEMORY_ID: MemoryId = MemoryId::new(0);
pub static NEURON_REFERENCES_MEMORY_ID: MemoryId = MemoryId::new(1);
thread_local! {
    pub static MEMORY_MANAGER: MemoryManagerStorage = init_memory_manager();
    pub static CONFIG: CellStorageRef<CanisterConfig> = init_cell(&MEMORY_MANAGER, "config", CONFIG_MEMORY_ID);
    pub static NEURON_REFERENCES: StorageRef<u64, NeuronReference> =
        init_btree(&MEMORY_MANAGER, NEURON_REFERENCES_MEMORY_ID);
}
