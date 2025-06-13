use toolkit_utils::{
    storage::{Storage, StorageInsertableByKey, StorageQueryable, StorageUpdateable},
    StaticStorageRef,
};

use crate::types::{proposal::{ServiceData}, service_canisters::GovernanceCanisterId};

use super::storages::MODULE_PROPOSALS;

pub struct ModuleProposalsStore;

impl Storage<GovernanceCanisterId, ServiceData> for ModuleProposalsStore {
    const NAME: &'static str = "module_proposals";

    fn storage() -> StaticStorageRef<GovernanceCanisterId, ServiceData> {
        &MODULE_PROPOSALS
    }
}

impl StorageQueryable<GovernanceCanisterId, ServiceData> for ModuleProposalsStore {}
impl StorageUpdateable<GovernanceCanisterId, ServiceData> for ModuleProposalsStore {}
impl StorageInsertableByKey<GovernanceCanisterId, ServiceData> for ModuleProposalsStore {}
