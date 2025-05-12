use toolkit_utils::{
    api_error::ApiError,
    result::CanisterResult,
    storage::{Storage, StorageInsertable, StorageQueryable, StorageUpdateable},
    StaticStorageRef,
};

use crate::types::{icp_neuron_reference::IcpNeuronReference, modules::IcpNeuronIdentifier};

use super::storages::ICP_NEURON_REFERENCES;

pub struct IcpNeuronReferenceStore;

impl Storage<u64, IcpNeuronReference> for IcpNeuronReferenceStore {
    const NAME: &'static str = "neuron_reference";

    fn storage() -> StaticStorageRef<u64, IcpNeuronReference> {
        &ICP_NEURON_REFERENCES
    }
}

impl StorageQueryable<u64, IcpNeuronReference> for IcpNeuronReferenceStore {}
impl StorageUpdateable<u64, IcpNeuronReference> for IcpNeuronReferenceStore {}
impl StorageInsertable<IcpNeuronReference> for IcpNeuronReferenceStore {}

impl IcpNeuronReferenceStore {
    pub fn get_latest_key() -> u64 {
        Self::storage().with(|data| data.borrow().last_key_value().map(|(k, _)| k).unwrap_or(0))
    }

    pub fn get_by_identifier(
        identifier: &IcpNeuronIdentifier,
    ) -> CanisterResult<(u64, IcpNeuronReference)> {
        match identifier {
            IcpNeuronIdentifier::NeuronId(neuron_id) => Self::storage().with(|data| {
                data.borrow()
                    .iter()
                    .find(|(_, neuron)| neuron.neuron_id == Some(*neuron_id))
                    .map(|(id, neuron)| (id, neuron.clone()))
                    .ok_or_else(|| ApiError::not_found("Neuron not found"))
            }),
            IcpNeuronIdentifier::Subaccount(subaccount) => Self::storage().with(|data| {
                data.borrow()
                    .iter()
                    .find(|(_, neuron)| neuron.subaccount == *subaccount)
                    .map(|(id, neuron)| (id, neuron.clone()))
                    .ok_or_else(|| ApiError::not_found("Neuron not found"))
            }),
        }
    }
}
