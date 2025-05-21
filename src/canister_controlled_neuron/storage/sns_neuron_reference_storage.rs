use toolkit_utils::{
    api_error::ApiError,
    result::CanisterResult,
    storage::{Storage, StorageInsertable, StorageQueryable, StorageUpdateable},
    StaticStorageRef,
};

use crate::types::sns_neuron_reference::SnsNeuronReference;

use super::storages::SNS_NEURON_REFERENCES;

pub struct SnsNeuronReferenceStore;

impl Storage<u64, SnsNeuronReference> for SnsNeuronReferenceStore {
    const NAME: &'static str = "sns_neuron_reference";

    fn storage() -> StaticStorageRef<u64, SnsNeuronReference> {
        &SNS_NEURON_REFERENCES
    }
}

impl StorageQueryable<u64, SnsNeuronReference> for SnsNeuronReferenceStore {}
impl StorageUpdateable<u64, SnsNeuronReference> for SnsNeuronReferenceStore {}
impl StorageInsertable<SnsNeuronReference> for SnsNeuronReferenceStore {}

impl SnsNeuronReferenceStore {
    pub fn get_latest_key() -> u64 {
        Self::storage().with(|data| data.borrow().last_key_value().map(|(k, _)| k).unwrap_or(0))
    }

    pub fn get_by_id(neuron_id: Vec<u8>) -> CanisterResult<(u64, SnsNeuronReference)> {
        Self::storage().with(|data| {
            data.borrow()
                .iter()
                .find(|(_, neuron)| neuron.neuron_id == Some(neuron_id.clone()))
                .map(|(id, neuron)| (id, neuron.clone()))
                .ok_or_else(|| ApiError::not_found("Neuron not found"))
        })
    }
}
