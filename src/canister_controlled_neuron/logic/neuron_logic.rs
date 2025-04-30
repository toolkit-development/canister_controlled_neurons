use std::time::Duration;

use ic_cdk::{api::time, futures::spawn};
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
    storage::neuron_reference_storage::NeuronReferenceStore,
    timers::storages::{Timers, COUNTER},
    traits::timer_traits::TimerActions,
    types::{
        modules::DisburseMaturityType,
        neuron_reference::{
            NeuronReference, NeuronReferenceResponse, PostCustomTargetmaturityDisbursement,
        },
    },
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
        maturity_disbursement_target: Option<PostCustomTargetmaturityDisbursement>,
    ) -> CanisterResult<NeuronReferenceResponse> {
        let neuron = NeuronReference::new(amount_e8s, maturity_disbursement_target.clone()).await?;
        let (id, mut neuron) = NeuronReferenceStore::insert(neuron.clone())?;

        let claimed_neuron = neuron.claim_or_refresh().await?;
        let response = NeuronReferenceStore::update(id, claimed_neuron)?;

        if let Some(maturity_disbursement) = response.1.maturity_disbursements.clone() {
            let neuron_clone = response.1.clone();
            Timers::create_recurring(
                &neuron.subaccount,
                Duration::from_secs(maturity_disbursement.interval_seconds),
                move || {
                    let neuron = neuron_clone.clone(); // clone inside the closure
                    spawn(async move { neuron.disburse_maturity().await });
                },
            );
        }

        Ok(response.1.to_response(response.0))
    }

    // after this call `claim_or_refresh_neuron` should be called
    pub async fn top_up_neuron_by_subaccount(
        subaccount: [u8; 32],
        amount_e8s: u64,
    ) -> CanisterResult<u64> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;

        let blockheight = neuron.top_up(amount_e8s).await?;
        Ok(blockheight)
    }

    pub async fn command_neuron(
        subaccount: [u8; 32],
        command: ManageNeuronCommandRequest,
    ) -> CanisterResult<ManageNeuronResponse> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        let neuron = neuron.command(command).await?;
        Ok(neuron)
    }

    pub async fn set_maturity_disbursements(
        subaccount: [u8; 32],
        maturity_disbursements: DisburseMaturityType,
    ) -> CanisterResult<NeuronReferenceResponse> {
        let (id, mut neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        let updated_neuron = neuron.set_maturity_disbursements(maturity_disbursements)?;

        match updated_neuron.maturity_disbursements.clone() {
            Some(maturity_disbursements) => {
                let neuron_clone = updated_neuron.clone();
                Timers::create_recurring(
                    &subaccount,
                    Duration::from_secs(maturity_disbursements.interval_seconds),
                    move || {
                        let mut neuron = neuron_clone.clone();
                        neuron.last_disbursements_time_nanos = Some(time());
                        let _ = NeuronReferenceStore::update(id, neuron.clone());
                        COUNTER.with_borrow_mut(|counter| {
                            *counter += 1;
                        });
                        // Not sure is this is going to work, but it's the only way I can think of to update the neuron
                        // spawn(async move {
                        //     neuron.disburse_maturity().await;
                        // });
                    },
                );
            }
            None => {
                Timers::clear(&subaccount);
            }
        }
        let (id, neuron) = NeuronReferenceStore::update(id, updated_neuron)?;
        Ok(neuron.to_response(id))
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
