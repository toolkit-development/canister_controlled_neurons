use toolkit_utils::{
    api_error::ApiError,
    result::CanisterResult,
    storage::{Storage, StorageInsertable, StorageQueryable, StorageUpdateable},
    StaticStorageRef,
};

use crate::types::neuron_reference::NeuronReference;

use super::storages::NEURON_REFERENCES;

pub struct NeuronReferenceStore;

impl Storage<u64, NeuronReference> for NeuronReferenceStore {
    const NAME: &'static str = "neuron_reference";

    fn storage() -> StaticStorageRef<u64, NeuronReference> {
        &NEURON_REFERENCES
    }
}

impl StorageQueryable<u64, NeuronReference> for NeuronReferenceStore {}
impl StorageUpdateable<u64, NeuronReference> for NeuronReferenceStore {}
impl StorageInsertable<NeuronReference> for NeuronReferenceStore {}

impl NeuronReferenceStore {
    pub fn get_latest_key() -> u64 {
        Self::storage().with(|data| data.borrow().last_key_value().map(|(k, _)| k).unwrap_or(0))
    }

    pub fn get_by_subaccount(subaccount: [u8; 32]) -> CanisterResult<(u64, NeuronReference)> {
        Self::storage().with(|data| {
            let (id, neuron) = data
                .borrow()
                .iter()
                .find(|(_, neuron)| neuron.subaccount == subaccount)
                .ok_or(ApiError::not_found("Neuron not found"))?;

            Ok((id, neuron.clone()))
        })
    }
}
