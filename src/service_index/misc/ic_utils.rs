use candid::{Nat, Principal};
use toolkit_utils::{
    api_error::ApiError,
    result::CanisterResult,
};

use crate::api::{
    api_clients::ApiClients,
    icp_ledger_index_api::{
        Account, GetAccountIdentifierTransactionsResult, GetAccountTransactionsArgs, Operation,
        TransactionWithId,
    },
    icp_root_api::ListSnsCanistersArg,
};
use crate::types::{
    proposal::PurchaseModulesProposalData,
    service_canisters::{GovernanceCanisterId, LedgerCanisterId, RootCanisterId},
};

pub async fn get_canisters(
    root_canister_id: RootCanisterId,
) -> CanisterResult<(GovernanceCanisterId, LedgerCanisterId)> {
    let (canisters,) = ApiClients::sns_root(root_canister_id)
        .list_sns_canisters(ListSnsCanistersArg {})
        .await
        .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

    let governance_canister_id = canisters.governance.unwrap();
    let ledger_canister_id: Principal = canisters.ledger.unwrap();
    Ok((governance_canister_id, ledger_canister_id))
}

pub fn find_transaction(
    transactions: Vec<TransactionWithId>,
    purchase_data: &PurchaseModulesProposalData,
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

pub async fn check_payment(
    governance_canister_id: GovernanceCanisterId,
    purchase_data: &PurchaseModulesProposalData,
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
        match find_transaction(transactions, purchase_data) {
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