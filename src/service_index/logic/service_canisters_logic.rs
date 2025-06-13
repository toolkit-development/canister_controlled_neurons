#![allow(deprecated)] // This is an experimental feature to generate Rust binding from Candid.
use candid::{Encode, Nat, Principal};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument,
};
use ic_cdk::api::management_canister::provisional::CanisterSettings;
use toolkit_utils::{
    api_error::ApiError,
    result::CanisterResult,
    storage::{StorageInsertableByKey, StorageQueryable},
};

use crate::api::canister_controlled_neuron_api::Config;
use crate::api::icp_root_api::ListSnsCanistersArg;
use crate::types::service_canisters::{LedgerCanisterId, RootCanisterId};
use crate::{
    api::{
        api_clients::ApiClients,
        icp_governance_api::{
            Action, GetProposal, GetProposalResponse, ProposalData, ProposalId, Result1,
        },
        icp_ledger_index_api::{
            Account, GetAccountIdentifierTransactionsResult,
            GetAccountTransactionsArgs, Operation, TransactionWithId,
        },
    },
    storage::{
        module_proposals_storage::ModuleProposalsStore,
        service_canisters_storage::ServiceCanistersStore,
    },
    types::{
        proposal::{PurchaseModulesProposalData, ServiceData},
        service_canisters::{GovernanceCanisterId, ServiceCanisterId, ServiceCanisterModules},
    },
};
pub struct ServiceCanistersLogic;

impl ServiceCanistersLogic {
    pub fn get_service_canisters() -> CanisterResult<Vec<ServiceCanisterId>> {
        let canisters = ServiceCanistersStore::get_all();
        Ok(canisters
            .into_iter()
            .map(|(_, canister)| canister)
            .collect())
    }

    pub fn set_service_canisters(
        governance_canister_id: GovernanceCanisterId,
        service_canister_id: ServiceCanisterId,
    ) {
        ServiceCanistersStore::upsert_by_key(governance_canister_id, service_canister_id);
    }

    // async fn find_proposal(governance_canister_id: GovernanceCanisterId) -> CanisterResult {
    //     let (proposal_response,) = ApiClients::sns_governance(governance_canister_id)
    //         .list_proposals(ListProposals {
    //             include_reward_status: vec![],
    //             before_proposal: None,
    //             limit: 100,
    //             exclude_type: vec![],
    //             include_topics: None,
    //             include_status: vec![],
    //         })
    //         .await
    //         .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;
    //     let proposals = proposal_response.proposals;
    //     for proposal in proposals {
    //         let proposal_action = Some(Some(proposal.proposal).action);
    //         let proposal_action_is_transfer_sns_treasury_funds =
    //             proposal_action == Some(Some(Action::TransferSnsTreasuryFunds));
    //         let proposal_action_is_transfer_sns_treasury_funds_to_governance_canister =
    //             proposal_action_is_transfer_sns_treasury_funds
    //                 && Some(Some(proposal.proposal).to) == Some(Some(governance_canister_id));
    //         if proposal_action_is_transfer_sns_treasury_funds_to_governance_canister {
    //             return Ok(proposal.id);
    //         }
    //     }

    //     Ok(())
    // }

    pub async fn get_proposal(
        governance_canister_id: GovernanceCanisterId,
        proposal_id: u64,
    ) -> CanisterResult<ProposalData> {
        let (proposal_response,): (GetProposalResponse,) =
            ApiClients::sns_governance(governance_canister_id)
                .get_proposal(GetProposal {
                    proposal_id: Some(ProposalId { id: proposal_id }),
                })
                .await
                .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

        let proposal = match proposal_response.result {
            Some(Result1::Proposal(proposal)) => proposal,
            Some(Result1::Error(error)) => {
                return Err(ApiError::external_service_error(
                    error.error_message.as_str(),
                ))
            }
            None => return Err(ApiError::external_service_error("Proposal not found")),
        };

        Ok(proposal)
    }

    fn get_modules_cost(modules: ServiceCanisterModules) -> u64 {
        let mut cost = 0;
        if modules.pay_roll {
            cost += 100_000_000;
        }
        if modules.bookkeeping {
            cost += 100_000_000;
        }
        if modules.notification {
            cost += 100_000_000;
        }
        if modules.treasury_management {
            cost += 100_000_000;
        }
        if modules.governance {
            cost += 100_000_000;
        }
        if cost == 0 && modules.canister {
            cost += 100_000_000;
        }
        cost
    }

    fn verify_proposal(
        proposal: ProposalData,
        modules: ServiceCanisterModules,
    ) -> CanisterResult<(u64, u64)> {
        // Check if the proposal has an action
        let proposal = match proposal.proposal {
            Some(p) => p,
            None => return Err(ApiError::external_service_error("No proposal found")),
        };

        // Check if the action is TransferSnsTreasuryFunds
        let transfer = match proposal.action {
            Some(Action::TransferSnsTreasuryFunds(t)) => t,
            _ => return Err(ApiError::external_service_error("Correct action not found")),
        };

        // Check if the transfer is from the ICP treasury
        if transfer.from_treasury != 1 {
            return Err(ApiError::external_service_error(
                "Transfer from treasury is not sns-treasury",
            ));
        }

        // Check if the transfer amount is greater than the amount required for modules.
        let modules_cost = Self::get_modules_cost(modules);
        if transfer.amount_e8s < modules_cost {
            return Err(ApiError::external_service_error(
                "Transfer amount is too low",
            ));
        }

        // Check if the transfer is to this canister
        if let Some(to_principal) = transfer.to_principal {
            let this_canister = ic_cdk::api::canister_self();
            if to_principal != this_canister {
                return Err(ApiError::external_service_error(
                    "Transfer to principal is not this canister",
                ));
            }
        }

        // Check if memo exists
        match transfer.memo {
            Some(memo) => Ok((transfer.amount_e8s, memo)),
            None => Err(ApiError::external_service_error("Memo is required")),
        }
    }

    async fn get_canisters(root_canister_id: RootCanisterId) -> CanisterResult<(GovernanceCanisterId, LedgerCanisterId)> {
        let (canisters, ) = ApiClients::sns_root(root_canister_id).list_sns_canisters(ListSnsCanistersArg {
        }).await.map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

        let governance_canister_id = canisters.governance.unwrap();
        let ledger_canister_id: Principal = canisters.ledger.unwrap();
        Ok((governance_canister_id, ledger_canister_id))
    }

    pub async fn propose_purchase_modules(
        root_canister_id: RootCanisterId,
        proposal_id: u64,
        modules: ServiceCanisterModules,
    ) -> CanisterResult<()> {
        let (governance_canister_id, ledger_canister_id) = Self::get_canisters(root_canister_id).await?;
        // get the proposal
        let proposal = Self::get_proposal(governance_canister_id, proposal_id).await?;

        // verify the proposal id is valid
        let (amount, memo) = Self::verify_proposal(proposal, modules.clone())?;

        // get the service data
        let get_result = ModuleProposalsStore::get(governance_canister_id);
        match get_result {
            Ok((_, mut service_data)) => {
                // check if the proposal id is already in the proposals
                if service_data
                    .proposals
                    .iter()
                    .any(|p| p.proposal_id == proposal_id)
                {
                    return Err(ApiError::external_service_error("Proposal already exists"));
                }

                // add the proposal id to the proposals
                service_data.proposals.push(PurchaseModulesProposalData {
                    proposal_id,
                    amount,
                    modules,
                    memo,
                });

                // update the service data
                ModuleProposalsStore::upsert_by_key(governance_canister_id, service_data);
            }
            Err(_) => {
                // create the service data
                ModuleProposalsStore::upsert_by_key(
                    governance_canister_id,
                    ServiceData {
                        root_canister_id,
                        ledger_canister_id,
                        proposals: vec![PurchaseModulesProposalData {
                            proposal_id,
                            modules,
                            amount,
                            memo,
                        }],
                    },
                );
            }
        }
        Ok(())
    }

    fn find_transaction(
        transactions: Vec<TransactionWithId>,
        purchase_data: PurchaseModulesProposalData,
    ) -> CanisterResult<TransactionWithId> {
        for transaction in transactions {
            if transaction.transaction.memo != purchase_data.memo {
                continue;
            }
            let (to, amount) = match &transaction.transaction.operation {
                Operation::Transfer {
                    to,
                    fee: _,
                    from: _,
                    amount,
                    spender: _,
                } => (to, amount),
                _ => continue,
            };
            let this_canister = ic_cdk::api::canister_self();

            if to == &this_canister.to_text() && amount.e8s == purchase_data.amount {
                return Ok(transaction);
            }
        }
        Err(ApiError::external_service_error("Transaction not found"))
    }

    async fn check_payment(
        governance_canister_id: GovernanceCanisterId,
        purchase_data: PurchaseModulesProposalData,
    ) -> CanisterResult<()> {
        let page_size = 1000u64;
        let mut start = None;

        loop {
            let (transactions_response,) = ApiClients::icp_ledger_index()
                .get_account_transactions(GetAccountTransactionsArgs {
                    max_results: Nat::from(page_size),
                    start,
                    account: Account {
                        owner: governance_canister_id,
                        subaccount: None,
                    },
                })
                .await
                .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

            let transactions = match transactions_response {
                GetAccountIdentifierTransactionsResult::Ok(transactions) => {
                    transactions.transactions
                }
                GetAccountIdentifierTransactionsResult::Err(e) => {
                    return Err(ApiError::external_service_error(e.message.as_str()))
                }
            };

            if transactions.is_empty() {
                return Err(ApiError::external_service_error("Transaction not found"));
            }

            let next_start = transactions.last().map(|tx| Nat::from(tx.id));
            let tx_len = transactions.len() as u64;
            match Self::find_transaction(transactions, purchase_data.clone()) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    if tx_len <= page_size {
                        return Err(ApiError::external_service_error("Transaction not found"));
                    }
                    // Update start for next page
                    if let Some(start_id) = next_start {
                        start = Some(start_id);
                    } else {
                        return Err(ApiError::external_service_error("Transaction not found"));
                    }
                }
            }
        }
    }

    async fn deploy_service_canister(
        sns_ledger_canister_id: Principal,
        governance_canister_id: GovernanceCanisterId,
    ) -> CanisterResult<ServiceCanisterId> {
        // Step 1: Create the canister
        let create_args = CreateCanisterArgument {
            settings: Some(CanisterSettings {
                controllers: Some(vec![governance_canister_id]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                log_visibility: None,
                wasm_memory_limit: None,
            }),
        };
        let canister_record = create_canister(create_args, 10_000_000_000_000)
            .await
            .unwrap();
        let canister_id = canister_record.0.canister_id;
        let config = Config {
            sns_ledger_canister_id,
            governance_canister_id,
        };
        
        // Encode as Candid
        let arg = Encode!(&config).expect("Candid encoding failed");
        let install_args = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module: include_bytes!("../../../wasm/canister_controlled_neuron.wasm.gz").to_vec(),
            arg,
        };
        install_code(install_args).await.unwrap();

        Ok(canister_id)
    }

    pub async fn activate_modules(
        governance_canister_id: GovernanceCanisterId,
    ) -> CanisterResult<()> {
        // get the proposal
        let (_, service_data) = ModuleProposalsStore::get(governance_canister_id)?;
        for proposal in service_data.proposals {
            // check the payment
            Self::check_payment(governance_canister_id, proposal.clone()).await?;

            // check if there is a service canister id
            let service_canister_id;
            let service_canister_result = ServiceCanistersStore::get(governance_canister_id);
            if service_canister_result.is_ok() {
                service_canister_id = Some(service_canister_result.unwrap().1);
            } else {
                let sns_ledger_canister_id = service_data.ledger_canister_id;
                service_canister_id = Some(Self::deploy_service_canister(sns_ledger_canister_id, governance_canister_id).await?);
                ServiceCanistersLogic::set_service_canisters(governance_canister_id, service_canister_id.unwrap());
            }

            // print the service canister id
            ic_cdk::println!("Service canister id: {}", service_canister_id.unwrap());

            // TODO: activate modules
            
        }
        Ok(())
    }
}
