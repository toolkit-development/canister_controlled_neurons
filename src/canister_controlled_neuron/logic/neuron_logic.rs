use ic_cdk::api::time;
use toolkit_utils::{
    api_error::ApiError,
    result::CanisterResult,
    storage::{StorageInsertable, StorageQueryable, StorageUpdateable},
};

use crate::{
    api::{
        api_clients::ApiClients,
        icp_governance_api::{
            ListNeurons, ListNeuronsResponse, ManageNeuronCommandRequest, ManageNeuronResponse,
            Neuron as GovNeuron,
        },
    },
    storage::{log_storage::LogStore, neuron_reference_storage::NeuronReferenceStore},
    types::neuron_reference::{NeuronReference, NeuronReferenceResponse},
};

pub struct NeuronLogic;

impl NeuronLogic {
    pub fn remove_neuron(id: u64) -> CanisterResult<()> {
        NeuronReferenceStore::remove(id);
        Ok(())
    }

    pub fn get_neurons() -> CanisterResult<Vec<NeuronReferenceResponse>> {
        let neurons = NeuronReferenceStore::get_all();
        Ok(neurons
            .into_iter()
            .map(|(id, neuron)| neuron.to_response(id))
            .collect())
    }

    pub async fn create_neuron(
        amount_e8s: u64,
        auto_stake: Option<bool>,
        dissolve_delay: Option<u64>,
    ) -> CanisterResult<NeuronReferenceResponse> {
        let neuron = NeuronReference::new(amount_e8s).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error creating neuron: {}", time(), e));
            e
        })?;
        let (id, mut neuron) = NeuronReferenceStore::insert(neuron.clone())?;

        let claimed_neuron = neuron.claim_or_refresh().await.map_err(|e| {
            let _ = LogStore::insert(format!(
                "{}: Error claiming or refreshing neuron: {}",
                time(),
                e
            ));
            e
        })?;
        let response = NeuronReferenceStore::update(id, claimed_neuron)?;

        if let Some(dissolve_delay) = dissolve_delay {
            neuron
                .increase_dissolve_delay(dissolve_delay)
                .await
                .map_err(|e| {
                    let _ = LogStore::insert(format!(
                        "{}: Error setting dissolve delay: {}",
                        time(),
                        e
                    ));
                    e
                })?;
        }

        if let Some(auto_stake) = auto_stake {
            neuron.auto_stake_maturity(auto_stake).await.map_err(|e| {
                let _ = LogStore::insert(format!(
                    "{}: Error setting auto stake maturity: {}",
                    time(),
                    e
                ));
                e
            })?;
        }

        Ok(response.1.to_response(response.0))
    }

    pub async fn top_up_neuron_by_subaccount(
        subaccount: [u8; 32],
        amount_e8s: u64,
    ) -> CanisterResult<bool> {
        let (_, mut neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;

        let _ = neuron.top_up(amount_e8s).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error topping up neuron: {}", time(), e));
            e
        })?;

        neuron.claim_or_refresh().await.map_err(|e| {
            let _ = LogStore::insert(format!(
                "{}: Error claiming or refreshing neuron: {}",
                time(),
                e
            ));
            e
        })?;

        Ok(true)
    }

    pub async fn command_neuron(
        subaccount: [u8; 32],
        command: ManageNeuronCommandRequest,
    ) -> CanisterResult<ManageNeuronResponse> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        let neuron = neuron.command(command).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error commanding neuron: {}", time(), e));
            e
        })?;
        Ok(neuron)
    }

    pub async fn add_dissolve_delay(
        subaccount: [u8; 32],
        dissolve_delay: u64,
    ) -> CanisterResult<bool> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.increase_dissolve_delay(dissolve_delay).await?;
        let _ = LogStore::insert(format!("Dissolve delay set to {} seconds", dissolve_delay));
        Ok(true)
    }

    pub async fn set_dissolve_state(
        subaccount: [u8; 32],
        start_dissolving: bool,
    ) -> CanisterResult<bool> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.set_dissolve_state(start_dissolving).await?;
        Ok(true)
    }

    pub async fn auto_stake_maturity(
        subaccount: [u8; 32],
        auto_stake: bool,
    ) -> CanisterResult<bool> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.auto_stake_maturity(auto_stake).await?;
        let _ = LogStore::insert(format!("Auto stake maturity set to {}", auto_stake));
        Ok(true)
    }

    pub async fn get_full_neuron(subaccount: [u8; 32]) -> CanisterResult<GovNeuron> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.get_full_neuron().await
    }

    pub async fn list_controlled_neurons() -> CanisterResult<ListNeuronsResponse> {
        let (result,) = ApiClients::icp_governance()
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
