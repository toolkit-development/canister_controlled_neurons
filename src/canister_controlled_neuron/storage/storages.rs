use ic_stable_structures::memory_manager::MemoryId;
use toolkit_utils::{
    cell::CellStorageRef,
    storage_init::{init_btree, init_cell, init_memory_manager},
    MemoryManagerStorage, StorageRef,
};

use crate::types::{
    config::Config, icp_neuron_reference::IcpNeuronReference,
    sns_neuron_reference::SnsNeuronReference,
};

pub static CONFIG_MEMORY_ID: MemoryId = MemoryId::new(0);
pub static ICP_NEURON_REFERENCES_MEMORY_ID: MemoryId = MemoryId::new(2);
pub static SNS_NEURON_REFERENCES_MEMORY_ID: MemoryId = MemoryId::new(2);

pub static LOG_MEMORY_ID: MemoryId = MemoryId::new(254);

thread_local! {
    pub static MEMORY_MANAGER: MemoryManagerStorage = init_memory_manager();
    pub static CONFIG: CellStorageRef<Config> = init_cell(&MEMORY_MANAGER, "config", CONFIG_MEMORY_ID);
    pub static ICP_NEURON_REFERENCES: StorageRef<u64, IcpNeuronReference> =
        init_btree(&MEMORY_MANAGER, ICP_NEURON_REFERENCES_MEMORY_ID);
    pub static SNS_NEURON_REFERENCES: StorageRef<u64, SnsNeuronReference> =
        init_btree(&MEMORY_MANAGER, SNS_NEURON_REFERENCES_MEMORY_ID);
    pub static LOG: StorageRef<u64, String> = init_btree(&MEMORY_MANAGER, LOG_MEMORY_ID);
}
