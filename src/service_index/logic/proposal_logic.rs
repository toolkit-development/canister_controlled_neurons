
use toolkit_utils::storage::{StorageInsertableByKey, StorageQueryable};
use toolkit_utils::{
    api_error::ApiError,
    result::CanisterResult,
};

use crate::api::{
    api_clients::ApiClients,
    icp_governance_api::{
        Action, GetProposal, GetProposalResponse, ProposalData, ProposalId, Result1,
    },
};
use crate::misc::ic_utils::get_canisters;
use crate::types::{
    proposal::{PurchaseModulesProposalData, ServiceData},
    service_canisters::{GovernanceCanisterId, RootCanisterId, ServiceCanisterModules},
};
use crate::storage::{
    module_proposals_storage::ModuleProposalsStore,
};

pub struct ProposalLogic;

impl ProposalLogic {

    pub fn get_proposals(governance_canister_id: GovernanceCanisterId) -> CanisterResult<Vec<PurchaseModulesProposalData>> {
        let (_, service_data) = ModuleProposalsStore::get(governance_canister_id)?;
        Ok(service_data.proposals)
    }

    pub async fn propose_purchase_modules(
        root_canister_id: RootCanisterId,
        proposal_id: u64,
        modules: ServiceCanisterModules,
    ) -> CanisterResult<()> {
        let (governance_canister_id, ledger_canister_id) =
            get_canisters(root_canister_id).await?;
        
        let proposal = Self::get_proposal(governance_canister_id, proposal_id).await?;
        let (amount, memo) = Self::verify_proposal(proposal, modules.clone())?;

        let get_result = ModuleProposalsStore::get(governance_canister_id);
        match get_result {
            Ok((_, mut service_data)) => {
                if service_data
                    .proposals
                    .iter()
                    .any(|p| p.proposal_id == proposal_id)
                {
                    return Err(ApiError::external_service_error("Proposal already exists"));
                }

                service_data.proposals.push(PurchaseModulesProposalData {
                    proposal_id,
                    amount,
                    modules,
                    memo,
                });

                ModuleProposalsStore::upsert_by_key(governance_canister_id, service_data);
            }
            Err(_) => {
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
    
    pub fn get_modules_cost(modules: ServiceCanisterModules) -> u64 {
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

    pub fn verify_proposal(
        proposal: ProposalData,
        modules: ServiceCanisterModules,
    ) -> CanisterResult<(u64, u64)> {
        let proposal = proposal
            .proposal
            .ok_or_else(|| ApiError::external_service_error("No proposal found"))?;

        let transfer = match proposal.action {
            Some(Action::TransferSnsTreasuryFunds(t)) => t,
            _ => return Err(ApiError::external_service_error("Incorrect action type")),
        };

        if transfer.from_treasury != 1 {
            return Err(ApiError::external_service_error(
                "Transfer must be from sns-treasury",
            ));
        }

        let modules_cost = Self::get_modules_cost(modules);
        if transfer.amount_e8s < modules_cost {
            return Err(ApiError::external_service_error(
                format!(
                    "Transfer amount {} is less than required cost {}",
                    transfer.amount_e8s, modules_cost
                )
                .as_str(),
            ));
        }

        if let Some(to_principal) = transfer.to_principal {
            let this_canister = ic_cdk::api::canister_self();
            if to_principal != this_canister {
                return Err(ApiError::external_service_error(
                    "Transfer must be to this canister",
                ));
            }
        }

        transfer
            .memo
            .map(|memo| Ok((transfer.amount_e8s, memo)))
            .unwrap_or_else(|| Err(ApiError::external_service_error("Memo is required")))
    }

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
} 