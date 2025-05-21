use candid::{CandidType, Nat};
use ic_cdk::api::{canister_self, time};
use serde::{Deserialize, Serialize};
use toolkit_utils::{
    api_error::ApiError, cell::CellStorage, impl_storable_for, result::CanisterResult,
    storage::StorageInsertable,
};

use crate::{
    api::{
        api_clients::ApiClients,
        sns_governance_api::{
            By, ChangeAutoStakeMaturity, ClaimOrRefresh, ClaimOrRefreshResponse, Command, Command1,
            Configure, IncreaseDissolveDelay, ManageNeuron, MemoAndController, Operation,
        },
        sns_ledger_api::{Account, Result_, TransferArg},
    },
    helpers::subaccount_helper::generate_subaccount_by_nonce,
    storage::{
        config_storage::config_store, log_storage::LogStore,
        sns_neuron_reference_storage::SnsNeuronReferenceStore,
    },
};

impl_storable_for!(SnsNeuronReference);

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct SnsNeuronReference {
    pub blockheight: Nat,
    pub subaccount: [u8; 32],
    pub nonce: u64,
    pub neuron_id: Option<Vec<u8>>,
    pub parent_subaccount: Option<[u8; 32]>,
}

impl SnsNeuronReference {
    pub async fn new(amount_e8s: u64) -> CanisterResult<SnsNeuronReference> {
        let sns_governance = ApiClients::sns_governance();

        let (parameters,) = sns_governance
            .get_nervous_system_parameters(())
            .await
            .map_err(|(_, e)| {
                let _ = LogStore::insert(format!(
                    "{}: Error getting nervous system parameters: {}",
                    time(),
                    e
                ));
                ApiError::external_service_error(&format!(
                    "Error getting nervous system parameters: {}",
                    e
                ))
            })?;

        let fee = parameters.transaction_fee_e8s.unwrap_or(0);
        let minimum_stake = parameters.neuron_minimum_stake_e8s.unwrap_or(0);

        if amount_e8s < minimum_stake + fee {
            return Err(ApiError::bad_request(&format!(
                "Amount is too small, minimum is {} e8s",
                minimum_stake + fee
            )));
        }

        let nonce = SnsNeuronReferenceStore::get_latest_key() + 1;
        let subaccount = generate_subaccount_by_nonce(nonce);

        let transfer_arg = TransferArg {
            to: Account {
                owner: sns_governance.0,
                subaccount: Some(subaccount.to_vec()),
            },
            fee: Some(Nat::from(fee)),
            memo: Some(nonce.to_be_bytes().to_vec()),
            from_subaccount: None,
            created_at_time: None,
            amount: Nat::from(amount_e8s),
        };

        let sns_ledger = ApiClients::sns_ledger();
        let result = sns_ledger
            .icrc_1_transfer(transfer_arg)
            .await
            .map_err(|e| {
                let _ = LogStore::insert(format!("{}: Error creating SNS neuron: {:?}", time(), e));
                e
            })
            .map_err(|e| {
                let _ = LogStore::insert(format!("{}: Error creating SNS neuron: {:?}", time(), e));
                ApiError::external_service_error("Error creating SNS neuron")
            })?;

        match result.0 {
            Result_::Ok(result) => {
                let neuron = SnsNeuronReference {
                    blockheight: result,
                    subaccount,
                    nonce,
                    neuron_id: None,
                    parent_subaccount: None,
                };

                Ok(neuron)
            }
            Result_::Err(e) => {
                let _ = LogStore::insert(format!("{}: Error creating SNS neuron: {:?}", time(), e));
                Err(ApiError::external_service_error(
                    "Error creating SNS neuron",
                ))
            }
        }
    }

    pub async fn claim_or_refresh(&mut self) -> CanisterResult<ClaimOrRefreshResponse> {
        let (result,) = ApiClients::sns_governance()
            .manage_neuron(ManageNeuron {
                subaccount: self.subaccount.to_vec(),
                command: Some(Command::ClaimOrRefresh(ClaimOrRefresh {
                    by: Some(By::MemoAndController(MemoAndController {
                        controller: Some(canister_self()),
                        memo: self.nonce,
                    })),
                })),
            })
            .await
            .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

        match result.command {
            Some(Command1::ClaimOrRefresh(response)) => Ok(response),
            Some(Command1::Error(e)) => Err(ApiError::external_service_error(&e.error_message)),
            _ => Err(ApiError::external_service_error("Unknown command")),
        }
    }

    pub async fn top_up(&self, amount_e8s: u64) -> CanisterResult<Nat> {
        let config = config_store().get()?;

        let transfer_arg = TransferArg {
            to: Account {
                owner: config.governance_canister_id,
                subaccount: Some(self.subaccount.to_vec()),
            },
            fee: None,
            memo: None,
            from_subaccount: None,
            created_at_time: None,
            amount: Nat::from(amount_e8s),
        };

        let sns_ledger = ApiClients::sns_ledger();
        let result = sns_ledger
            .icrc_1_transfer(transfer_arg)
            .await
            .map_err(|e| {
                let _ = LogStore::insert(format!("{}: Error top up SNS neuron: {:?}", time(), e));
                e
            })
            .map_err(|e| {
                let _ = LogStore::insert(format!("{}: Error top up SNS neuron: {:?}", time(), e));
                ApiError::external_service_error("Error top up SNS neuron")
            })?;

        match result.0 {
            Result_::Ok(result) => Ok(result),
            Result_::Err(e) => {
                let _ = LogStore::insert(format!("{}: Error top up SNS neuron: {:?}", time(), e));
                Err(ApiError::external_service_error("Error top up SNS neuron"))
            }
        }
    }

    pub async fn increase_dissolve_delay(&self, dissolve_delay: u64) -> CanisterResult<()> {
        self.configure(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
            additional_dissolve_delay_seconds: dissolve_delay as u32,
        }))
        .await
    }

    pub async fn set_dissolve_state(&self, start_dissolving: bool) -> CanisterResult<()> {
        if start_dissolving {
            self.configure(Operation::StartDissolving {})
        } else {
            self.configure(Operation::StopDissolving {})
        }
        .await?;
        Ok(())
    }

    pub async fn auto_stake_maturity(&self, auto_stake: bool) -> CanisterResult<Command1> {
        let result = self
            .command(Command::Configure(Configure {
                operation: Some(Operation::ChangeAutoStakeMaturity(
                    ChangeAutoStakeMaturity {
                        requested_setting_for_auto_stake_maturity: auto_stake,
                    },
                )),
            }))
            .await?;
        Ok(result)
    }

    pub async fn command(&self, command: Command) -> CanisterResult<Command1> {
        match &self.neuron_id {
            Some(neuron_id) => {
                let (result,) = ApiClients::sns_governance()
                    .manage_neuron(ManageNeuron {
                        command: Some(command),
                        subaccount: neuron_id.to_vec(),
                    })
                    .await
                    .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

                if let Some(command_result) = result.command {
                    match command_result {
                        Command1::Error(e) => {
                            Err(ApiError::external_service_error(e.error_message.as_str()))
                        }
                        _ => Ok(command_result),
                    }
                } else {
                    Err(ApiError::external_service_error("Unknown command"))
                }
            }
            None => Err(ApiError::bad_request("Neuron not claimed yet")),
        }
    }

    pub async fn configure(&self, operation: Operation) -> CanisterResult<()> {
        let result = self
            .command(Command::Configure(Configure {
                operation: Some(operation),
            }))
            .await?;
        match result {
            Command1::Configure {} => Ok(()),
            _ => Err(ApiError::external_service_error("Unexpected response")),
        }
    }

    pub fn to_response(self, storage_reference_id: u64) -> SnsNeuronReferenceResponse {
        let sns_governance = ApiClients::sns_governance();
        SnsNeuronReferenceResponse {
            storage_reference_id,
            blockheight: self.blockheight,
            subaccount: self.subaccount,
            nonce: self.nonce,
            neuron_id: self.neuron_id,
            parent_subaccount: self.parent_subaccount,
            topup_account: Account {
                owner: sns_governance.0,
                subaccount: Some(self.subaccount.to_vec()),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct SnsNeuronReferenceResponse {
    pub storage_reference_id: u64,
    pub blockheight: Nat,
    pub subaccount: [u8; 32],
    pub nonce: u64,
    pub neuron_id: Option<Vec<u8>>,
    pub parent_subaccount: Option<[u8; 32]>,
    pub topup_account: Account,
}
