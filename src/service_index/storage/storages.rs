use ic_stable_structures::memory_manager::MemoryId;
use toolkit_utils::{
    storage_init::{init_btree, init_memory_manager},
    MemoryManagerStorage, StorageRef,
};

use crate::types::{proposal::{ServiceData}, service_canisters::{GovernanceCanisterId, ServiceCanisterId}};
        

pub static SERVICE_CANISTERS_MEMORY_ID: MemoryId = MemoryId::new(0);
pub static MODULE_PROPOSALS_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    pub static MEMORY_MANAGER: MemoryManagerStorage = init_memory_manager();
    pub static SERVICE_CANISTERS: StorageRef<GovernanceCanisterId, ServiceCanisterId> =
        init_btree(&MEMORY_MANAGER, SERVICE_CANISTERS_MEMORY_ID);
    pub static MODULE_PROPOSALS: StorageRef<GovernanceCanisterId, ServiceData> =
        init_btree(&MEMORY_MANAGER, MODULE_PROPOSALS_MEMORY_ID);
}
