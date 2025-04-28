use crate::{
    traits::storage::{
        StaticStorageRef, Storage, StorageInsertable, StorageQueryable, StorageUpdateable,
    },
    types::neuron::NeuronReference,
};

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
        Self::storage().with(|data| data.borrow().last_key_value().map(|(k, _)| k).unwrap_or(1))
    }
}
