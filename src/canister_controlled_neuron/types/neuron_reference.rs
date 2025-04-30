use candid::CandidType;
use ic_ledger_types::{
    transfer, AccountIdentifier, Memo, Subaccount, Tokens, TransferArgs,
    MAINNET_GOVERNANCE_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use serde::{Deserialize, Serialize};
use toolkit_utils::{
    api_error::ApiError, cell::CellStorage, impl_storable_for, misc::generic::Time,
    result::CanisterResult,
};

use crate::{
    api::{
        api_clients::ApiClients,
        icp_governance_api::{
            Account, By, ClaimOrRefresh, ClaimOrRefreshResponse, Command1, DisburseMaturity,
            ManageNeuronCommandRequest, ManageNeuronRequest, ManageNeuronResponse,
            Neuron as GovNeuron, NeuronId, Result2,
        },
    },
    helpers::subaccount_helper::generate_subaccount_by_nonce,
    storage::{config_storage::config_store, neuron_reference_storage::NeuronReferenceStore},
};

use super::modules::DisburseMaturityType;

impl_storable_for!(NeuronReference);

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct NeuronReference {
    pub blockheight: u64,
    pub subaccount: [u8; 32],
    pub nonce: u64,
    pub neuron_id: Option<u64>,
    pub maturity_disbursements: Option<MaturityDisbursement>,
    pub last_disbursements_time_nanos: Option<Time>,
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct MaturityDisbursement {
    pub interval_seconds: u64,
    pub targets: Vec<DisburseMaturity>,
}

impl From<PostCustomTargetmaturityDisbursement> for MaturityDisbursement {
    fn from(post_maturity_disbursement: PostCustomTargetmaturityDisbursement) -> Self {
        MaturityDisbursement {
            interval_seconds: post_maturity_disbursement.interval_seconds,
            targets: vec![post_maturity_disbursement.target],
        }
    }
}
#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct PostCustomTargetmaturityDisbursement {
    pub interval_seconds: u64,
    pub target: DisburseMaturity,
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct PostTreasuryTargetmaturityDisbursement {
    pub interval_seconds: u64,
}

impl NeuronReference {
    pub async fn new(
        amount_e8s: u64,
        maturity_disbursement_target: Option<PostCustomTargetmaturityDisbursement>,
    ) -> CanisterResult<NeuronReference> {
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
            maturity_disbursements: maturity_disbursement_target.map(MaturityDisbursement::from),
            last_disbursements_time_nanos: None,
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

    pub fn set_maturity_disbursements(
        &mut self,
        maturity_disbursements: DisburseMaturityType,
    ) -> CanisterResult<NeuronReference> {
        match maturity_disbursements {
            DisburseMaturityType::Stop => {
                self.maturity_disbursements = None;
                Ok(self.clone())
            }
            DisburseMaturityType::StartCustomTarget(args) => {
                self.maturity_disbursements = Some(MaturityDisbursement::from(args));
                Ok(self.clone())
            }
            DisburseMaturityType::StartTreasuryTarget(args) => {
                let config = config_store().get()?;

                self.maturity_disbursements = Some(MaturityDisbursement {
                    interval_seconds: args.interval_seconds,
                    targets: vec![DisburseMaturity {
                        to_account: Some(Account {
                            owner: Some(config.governance_canister_id),
                            subaccount: None,
                        }),
                        percentage_to_disburse: 100,
                    }],
                });
                Ok(self.clone())
            }
        }
    }

    pub async fn disburse_maturity(&self) {
        if let Some(maturity_disbursements) = self.maturity_disbursements.clone() {
            for target in maturity_disbursements.targets.into_iter() {
                let _ = self
                    .command(ManageNeuronCommandRequest::DisburseMaturity(target))
                    .await;
            }
        }
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
            maturity_disbursements: self.maturity_disbursements.clone(),
            last_disbursements_time_nanos: self.last_disbursements_time_nanos,
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
    pub maturity_disbursements: Option<MaturityDisbursement>,
    pub last_disbursements_time_nanos: Option<Time>,
}
