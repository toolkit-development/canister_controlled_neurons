use crate::{
    api::{
        api_clients::ApiClients,
        governance_api::{ListNeurons, ListNeuronsResponse, NeuronInfo},
    },
    storage::neurons_storage::NeuronReferenceStore,
    traits::storage::{StorageInsertable, StorageQueryable, StorageUpdateable},
    types::{
        api_error::ApiError,
        neuron::{NeuronReference, NeuronReferenceResponse},
        result::CanisterResult,
    },
};

pub struct NeuronLogic;

impl NeuronLogic {
    pub fn get_neurons() -> CanisterResult<Vec<NeuronReferenceResponse>> {
        let neurons = NeuronReferenceStore::get_all();
        Ok(neurons
            .into_iter()
            .map(|(id, neuron)| neuron.to_response(id))
            .collect())
    }

    /// after this call `claim_or_refresh_neuron` should be called
    pub async fn create_neuron(amount_e8s: u64) -> CanisterResult<NeuronReferenceResponse> {
        let neuron = NeuronReference::new(amount_e8s).await?;
        NeuronReferenceStore::insert(neuron.clone()).map(|(id, neuron)| neuron.to_response(id))
    }

    pub async fn claim_or_refresh_neuron(id: u64) -> CanisterResult<NeuronReferenceResponse> {
        let (_, mut neuron) = NeuronReferenceStore::get(id)?;

        let neuron = neuron.claim_or_refresh().await?;
        NeuronReferenceStore::update(id, neuron).map(|(id, neuron)| neuron.to_response(id))
    }

    // after this call `claim_or_refresh_neuron` should be called
    pub async fn top_up_neuron_by_subaccount(id: u64, amount_e8s: u64) -> CanisterResult<u64> {
        let (_, neuron) = NeuronReferenceStore::get(id)?;

        let blockheight = neuron.top_up(amount_e8s).await?;
        Ok(blockheight)
    }

    pub async fn get_neuron_info(id: u64) -> CanisterResult<NeuronInfo> {
        let (_, neuron) = NeuronReferenceStore::get(id)?;
        neuron.get_info().await
    }

    pub async fn list_controlled_neurons() -> CanisterResult<ListNeuronsResponse> {
        let (result,) = ApiClients::governance()
            .list_neurons(ListNeurons {
                page_size: Some(1000),
                include_public_neurons_in_full_neurons: None,
                neuron_ids: vec![],
                page_number: None,
                include_empty_neurons_readable_by_caller: None,
                neuron_subaccounts: None,
                include_neurons_readable_by_caller: true,
            })
            .await
            .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

        Ok(result)
    }
}
