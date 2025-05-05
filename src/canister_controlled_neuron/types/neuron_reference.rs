use candid::CandidType;
use ic_ledger_types::{
    transfer, AccountIdentifier, Memo, Subaccount, Tokens, TransferArgs, DEFAULT_SUBACCOUNT,
    MAINNET_GOVERNANCE_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use serde::{Deserialize, Serialize};
use toolkit_utils::{
    api_error::ApiError, cell::CellStorage, impl_storable_for, result::CanisterResult,
};

use crate::{
    api::{
        api_clients::ApiClients,
        icp_governance_api::{
            Account, AccountIdentifier as ApiAccountIdentifier, By, ChangeAutoStakeMaturity,
            ClaimOrRefresh, ClaimOrRefreshResponse, Command1, Configure, Disburse,
            DisburseResponse, Follow, IncreaseDissolveDelay, MakeProposalRequest,
            MakeProposalResponse, ManageNeuronCommandRequest, ManageNeuronRequest,
            ManageNeuronResponse, Neuron as GovNeuron, NeuronId, NeuronIdOrSubaccount, Operation,
            ProposalId, RegisterVote, Result2, SetVisibility, Spawn, SpawnResponse,
        },
    },
    helpers::subaccount_helper::generate_subaccount_by_nonce,
    storage::{config_storage::config_store, neuron_reference_storage::NeuronReferenceStore},
};

use super::{modules::Vote, topic::Topic};

impl_storable_for!(NeuronReference);

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
pub struct NeuronReference {
    pub blockheight: u64,
    pub subaccount: [u8; 32],
    pub nonce: u64,
    pub neuron_id: Option<u64>,
    pub parent_subaccount: Option<[u8; 32]>,
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
            parent_subaccount: None,
        };

        Ok(neuron)
    }

    pub async fn claim_or_refresh(&mut self) -> CanisterResult<ClaimOrRefreshResponse> {
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

        match result.command {
            Some(Command1::ClaimOrRefresh(response)) => Ok(response),
            Some(Command1::Error(e)) => Err(ApiError::external_service_error(&e.error_message)),
            _ => Err(ApiError::external_service_error("Unknown command")),
        }
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

    pub async fn configure(&self, operation: Operation) -> CanisterResult<()> {
        let result = self
            .command(ManageNeuronCommandRequest::Configure(Configure {
                operation: Some(operation),
            }))
            .await?;
        match result.command {
            Some(Command1::Configure {}) => Ok(()),
            _ => Err(ApiError::external_service_error("Unexpected response")),
        }
    }

    pub async fn increase_dissolve_delay(&self, dissolve_delay: u64) -> CanisterResult<()> {
        self.configure(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
            additional_dissolve_delay_seconds: dissolve_delay as u32,
        }))
        .await
    }

    pub async fn auto_stake_maturity(&self, auto_stake: bool) -> CanisterResult<()> {
        self.configure(Operation::ChangeAutoStakeMaturity(
            ChangeAutoStakeMaturity {
                requested_setting_for_auto_stake_maturity: auto_stake,
            },
        ))
        .await
    }

    pub async fn set_dissolve_state(&self, start_dissolving: bool) -> CanisterResult<()> {
        if start_dissolving {
            self.configure(Operation::StartDissolving {})
        } else {
            self.configure(Operation::StopDissolving {})
        }
        .await
    }

    pub async fn set_publicity(&self, publicity: SetVisibility) -> CanisterResult<()> {
        self.configure(Operation::SetVisibility(publicity)).await
    }

    pub async fn spawn(&self, nonce: u64) -> CanisterResult<SpawnResponse> {
        // let config = config_store().get()?;
        let result = self
            .command(ManageNeuronCommandRequest::Spawn(Spawn {
                percentage_to_spawn: Some(100),
                new_controller: None, //Some(config.governance_canister_id),
                nonce: Some(nonce),
            }))
            .await?;
        match result.command {
            Some(Command1::Error(e)) => {
                Err(ApiError::external_service_error(e.error_message.as_str()))
            }
            Some(Command1::Spawn(response)) => Ok(response),
            _ => Err(ApiError::external_service_error("Unknown command")),
        }
    }

    pub async fn create_proposal(
        &self,
        proposal: MakeProposalRequest,
    ) -> CanisterResult<MakeProposalResponse> {
        let result = self
            .command(ManageNeuronCommandRequest::MakeProposal(proposal))
            .await?;
        match result.command {
            Some(Command1::MakeProposal(response)) => Ok(response),
            Some(Command1::Error(e)) => {
                Err(ApiError::external_service_error(e.error_message.as_str()))
            }
            _ => Err(ApiError::external_service_error("Unexpected response")),
        }
    }

    pub async fn vote(&self, proposal_id: u64, vote: Vote) -> CanisterResult<bool> {
        let result = self
            .command(ManageNeuronCommandRequest::RegisterVote(RegisterVote {
                proposal: Some(ProposalId { id: proposal_id }),
                vote: match vote {
                    Vote::Approve => 1,
                    Vote::Reject => 2,
                },
            }))
            .await?;
        match result.command {
            Some(Command1::RegisterVote {}) => Ok(true),
            Some(Command1::Error(e)) => {
                Err(ApiError::external_service_error(e.error_message.as_str()))
            }
            _ => Err(ApiError::external_service_error("Unexpected response")),
        }
    }

    pub async fn set_following(
        &self,
        topic: Topic,
        following_neurons: Vec<u64>,
    ) -> CanisterResult<()> {
        let followees = following_neurons
            .iter()
            .map(|id| NeuronId { id: *id })
            .collect();

        let result = self
            .command(ManageNeuronCommandRequest::Follow(Follow {
                topic: topic.into(),
                followees,
            }))
            .await?;

        match result.command {
            Some(Command1::Follow {}) => Ok(()),
            _ => Err(ApiError::external_service_error("Unexpected response")),
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
                        id: None,
                        command: Some(command),
                        neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::NeuronId(NeuronId {
                            id: neuron_id,
                        })),
                    })
                    .await
                    .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

                if let Some(command_result) = result.command {
                    match command_result {
                        Command1::Error(e) => {
                            // Not sure why this throws
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

    pub async fn disburse(&self) -> CanisterResult<DisburseResponse> {
        let config = config_store().get()?;
        let account_identifier =
            AccountIdentifier::new(&config.governance_canister_id, &DEFAULT_SUBACCOUNT);
        let x = self
            .command(ManageNeuronCommandRequest::Disburse(Disburse {
                to_account: Some(ApiAccountIdentifier {
                    hash: account_identifier.as_bytes().to_vec(),
                }),
                amount: None,
            }))
            .await?;

        match x.command {
            Some(Command1::Disburse(response)) => Ok(response),
            _ => Err(ApiError::external_service_error("Unexpected response")),
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
            parent_subaccount: self.parent_subaccount,
            topup_account: Account {
                owner: Some(MAINNET_GOVERNANCE_CANISTER_ID),
                subaccount: Some(self.subaccount.to_vec()),
            },
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
    pub parent_subaccount: Option<[u8; 32]>,
    pub topup_account: Account,
}
