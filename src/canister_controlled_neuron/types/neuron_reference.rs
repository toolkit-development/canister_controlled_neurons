use candid::CandidType;
use ic_ledger_types::{
    transfer, AccountIdentifier, Memo, Subaccount, Tokens, TransferArgs,
    MAINNET_GOVERNANCE_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use serde::{Deserialize, Serialize};
use toolkit_utils::{api_error::ApiError, impl_storable_for, result::CanisterResult};

use crate::{
    api::{
        api_clients::ApiClients,
        icp_governance_api::{
            By, ChangeAutoStakeMaturity, ClaimOrRefresh, ClaimOrRefreshResponse, Command1,
            Configure, IncreaseDissolveDelay, ManageNeuronCommandRequest, ManageNeuronRequest,
            ManageNeuronResponse, Neuron as GovNeuron, NeuronId, Operation, Result2,
        },
    },
    helpers::subaccount_helper::generate_subaccount_by_nonce,
    storage::neuron_reference_storage::NeuronReferenceStore,
};

impl_storable_for!(NeuronReference);

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct NeuronReference {
    pub blockheight: u64,
    pub subaccount: [u8; 32],
    pub nonce: u64,
    pub neuron_id: Option<u64>,
}

impl NeuronReference {
    pub async fn new(amount_e8s: u64) -> CanisterResult<NeuronReference> {
        let fee = 10_000;

        if amount_e8s < 100_000_000 + fee {
            return Err(ApiError::bad_request(&format!(
                "Amount is too small, minimum is {} e8s",
                100_000_000 + fee
            )));
        }

        let nonce = NeuronReferenceStore::get_latest_key() + 1;
        let subaccount = generate_subaccount_by_nonce(nonce);
        let account_identifier =
            AccountIdentifier::new(&MAINNET_GOVERNANCE_CANISTER_ID, &Subaccount(subaccount));

        let transfer_args = TransferArgs {
            memo: Memo(nonce),
            amount: Tokens::from_e8s(amount_e8s),
            fee: Tokens::from_e8s(10000),
            from_subaccount: None,
            to: account_identifier,
            created_at_time: None,
        };

        let blockheight = transfer(MAINNET_LEDGER_CANISTER_ID, &transfer_args)
            .await
            .map_err(|e| ApiError::external_service_error(e.to_string().as_str()))?
            .map_err(|e| ApiError::external_service_error(e.to_string().as_str()))?;

        let neuron = NeuronReference {
            blockheight,
            subaccount,
            nonce,
            neuron_id: None,
        };

        Ok(neuron)
    }

    pub async fn claim_or_refresh(&mut self) -> CanisterResult<NeuronReference> {
        let (result,) = ApiClients::icp_governance()
            .manage_neuron(ManageNeuronRequest {
                id: None,
                command: Some(ManageNeuronCommandRequest::ClaimOrRefresh(ClaimOrRefresh {
                    by: Some(By::Memo(self.nonce)),
                })),
                neuron_id_or_subaccount: None,
            })
            .await
            .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

        if let Some(command) = result.command {
            match command {
                Command1::ClaimOrRefresh(ClaimOrRefreshResponse {
                    refreshed_neuron_id,
                }) => self.neuron_id = Some(refreshed_neuron_id.unwrap().id),
                Command1::Error(e) => {
                    return Err(ApiError::external_service_error(e.error_message.as_str()))
                }
                _ => return Err(ApiError::external_service_error("Unknown command")),
            }
        }

        Ok(self.clone())
    }

    pub async fn top_up(&self, amount_e8s: u64) -> CanisterResult<u64> {
        let account_identifier = AccountIdentifier::new(
            &MAINNET_GOVERNANCE_CANISTER_ID,
            &Subaccount(self.subaccount),
        );

        let transfer_args = TransferArgs {
            memo: Memo(0),
            amount: Tokens::from_e8s(amount_e8s),
            fee: Tokens::from_e8s(10000),
            from_subaccount: None,
            to: account_identifier,
            created_at_time: None,
        };

        transfer(MAINNET_LEDGER_CANISTER_ID, &transfer_args)
            .await
            .map_err(|e| ApiError::external_service_error(e.to_string().as_str()))?
            .map_err(|e| ApiError::external_service_error(e.to_string().as_str()))
    }

    pub async fn configure(&self, operation: Operation) -> CanisterResult<ManageNeuronResponse> {
        self.command(ManageNeuronCommandRequest::Configure(Configure {
            operation: Some(operation),
        }))
        .await
    }

    pub async fn increase_dissolve_delay(
        &self,
        dissolve_delay: u64,
    ) -> CanisterResult<ManageNeuronResponse> {
        self.configure(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
            additional_dissolve_delay_seconds: dissolve_delay as u32,
        }))
        .await
    }

    pub async fn auto_stake_maturity(
        &self,
        auto_stake: bool,
    ) -> CanisterResult<ManageNeuronResponse> {
        self.configure(Operation::ChangeAutoStakeMaturity(
            ChangeAutoStakeMaturity {
                requested_setting_for_auto_stake_maturity: auto_stake,
            },
        ))
        .await
    }

    pub async fn set_dissolve_state(
        &self,
        start_dissolving: bool,
    ) -> CanisterResult<ManageNeuronResponse> {
        if start_dissolving {
            self.configure(Operation::StartDissolving {})
        } else {
            self.configure(Operation::StopDissolving {})
        }
        .await
    }

    pub async fn command(
        &self,
        command: ManageNeuronCommandRequest,
    ) -> CanisterResult<ManageNeuronResponse> {
        match self.neuron_id {
            Some(neuron_id) => {
                let (result,) = ApiClients::icp_governance()
                    .manage_neuron(ManageNeuronRequest {
                        id: Some(NeuronId { id: neuron_id }),
                        command: Some(command),
                        neuron_id_or_subaccount: None,
                    })
                    .await
                    .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

                if let Some(command_result) = result.command {
                    match command_result {
                        Command1::Error(e) => {
                            Err(ApiError::external_service_error(e.error_message.as_str()))
                        }
                        _ => Ok(ManageNeuronResponse {
                            command: Some(command_result),
                        }),
                    }
                } else {
                    Err(ApiError::external_service_error("Unknown command"))
                }
            }
            None => Err(ApiError::bad_request("Neuron not claimed yet")),
        }
    }

    pub async fn get_full_neuron(&self) -> CanisterResult<GovNeuron> {
        if let Some(neuron_id) = self.neuron_id {
            let (result,) = ApiClients::icp_governance()
                .get_full_neuron(neuron_id)
                .await
                .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

            match result {
                Result2::Ok(neuron_info) => Ok(neuron_info),
                Result2::Err(e) => Err(ApiError::external_service_error(e.error_message.as_str())),
            }
        } else {
            Err(ApiError::bad_request("Neuron not claimed yet"))
        }
    }

    pub fn to_response(&self, storage_reference_id: u64) -> NeuronReferenceResponse {
        NeuronReferenceResponse {
            storage_reference_id,
            blockheight: self.blockheight,
            subaccount: self.subaccount,
            nonce: self.nonce,
            neuron_id: self.neuron_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct NeuronReferenceResponse {
    pub storage_reference_id: u64,
    pub blockheight: u64,
    pub subaccount: [u8; 32],
    pub nonce: u64,
    pub neuron_id: Option<u64>,
}
