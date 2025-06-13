use toolkit_utils::{
    storage::{Storage, StorageInsertableByKey, StorageQueryable, StorageUpdateable},
    StaticStorageRef,
};

use crate::types::service_canisters::{GovernanceCanisterId, ServiceCanisterId};

use super::storages::SERVICE_CANISTERS;

pub struct ServiceCanistersStore;

impl Storage<GovernanceCanisterId, ServiceCanisterId> for ServiceCanistersStore {
    const NAME: &'static str = "service_canisters";

    fn storage() -> StaticStorageRef<GovernanceCanisterId, ServiceCanisterId> {
        &SERVICE_CANISTERS
    }
}

impl StorageQueryable<GovernanceCanisterId, ServiceCanisterId> for ServiceCanistersStore {}
impl StorageUpdateable<GovernanceCanisterId, ServiceCanisterId> for ServiceCanistersStore {}
impl StorageInsertableByKey<GovernanceCanisterId, ServiceCanisterId> for ServiceCanistersStore {}
