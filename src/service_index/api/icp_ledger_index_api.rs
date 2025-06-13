// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(clippy::large_enum_variant)]
#![allow(deprecated)]
// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Deserialize, Principal, Encode, Decode};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub struct InitArg { pub ledger_id: Principal }

#[derive(CandidType, Deserialize)]
pub struct GetAccountIdentifierTransactionsArgs {
  pub max_results: u64,
  pub start: Option<u64>,
  pub account_identifier: String,
}

#[derive(CandidType, Deserialize)]
pub struct Tokens { pub e8s: u64 }

#[derive(CandidType, Deserialize)]
pub struct TimeStamp { pub timestamp_nanos: u64 }

#[derive(CandidType, Deserialize)]
pub enum Operation {
  Approve{
    fee: Tokens,
    from: String,
    allowance: Tokens,
    expected_allowance: Option<Tokens>,
    expires_at: Option<TimeStamp>,
    spender: String,
  },
  Burn{ from: String, amount: Tokens, spender: Option<String> },
  Mint{ to: String, amount: Tokens },
  Transfer{
    to: String,
    fee: Tokens,
    from: String,
    amount: Tokens,
    spender: Option<String>,
  },
}

#[derive(CandidType, Deserialize)]
pub struct Transaction {
  pub memo: u64,
  pub icrc1_memo: Option<serde_bytes::ByteBuf>,
  pub operation: Operation,
  pub timestamp: Option<TimeStamp>,
  pub created_at_time: Option<TimeStamp>,
}

#[derive(CandidType, Deserialize)]
pub struct TransactionWithId { pub id: u64, pub transaction: Transaction }

#[derive(CandidType, Deserialize)]
pub struct GetAccountIdentifierTransactionsResponse {
  pub balance: u64,
  pub transactions: Vec<TransactionWithId>,
  pub oldest_tx_id: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct GetAccountIdentifierTransactionsError { pub message: String }

#[derive(CandidType, Deserialize)]
pub enum GetAccountIdentifierTransactionsResult {
  Ok(GetAccountIdentifierTransactionsResponse),
  Err(GetAccountIdentifierTransactionsError),
}

#[derive(CandidType, Deserialize)]
pub struct Account {
  pub owner: Principal,
  pub subaccount: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct GetAccountTransactionsArgs {
  pub max_results: candid::Nat,
  pub start: Option<candid::Nat>,
  pub account: Account,
}

#[derive(CandidType, Deserialize)]
pub struct GetBlocksRequest { pub start: candid::Nat, pub length: candid::Nat }

#[derive(CandidType, Deserialize)]
pub struct GetBlocksResponse {
  pub blocks: Vec<serde_bytes::ByteBuf>,
  pub chain_length: u64,
}

#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
  pub url: String,
  pub method: String,
  pub body: serde_bytes::ByteBuf,
  pub headers: Vec<(String,String,)>,
}

#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
  pub body: serde_bytes::ByteBuf,
  pub headers: Vec<(String,String,)>,
  pub status_code: u16,
}

#[derive(CandidType, Deserialize)]
pub struct Status { pub num_blocks_synced: u64 }

pub struct IcpLedgerIndexApi(pub Principal);
impl IcpLedgerIndexApi {
  pub async fn get_account_identifier_balance(&self, arg0: String) -> Result<
    (u64,)
  > { ic_cdk::call(self.0, "get_account_identifier_balance", (arg0,)).await }
  pub async fn get_account_identifier_transactions(
    &self,
    arg0: GetAccountIdentifierTransactionsArgs,
  ) -> Result<(GetAccountIdentifierTransactionsResult,)> {
    ic_cdk::call(self.0, "get_account_identifier_transactions", (arg0,)).await
  }
  pub async fn get_account_transactions(
    &self,
    arg0: GetAccountTransactionsArgs,
  ) -> Result<(GetAccountIdentifierTransactionsResult,)> {
    ic_cdk::call(self.0, "get_account_transactions", (arg0,)).await
  }
  pub async fn get_blocks(&self, arg0: GetBlocksRequest) -> Result<
    (GetBlocksResponse,)
  > { ic_cdk::call(self.0, "get_blocks", (arg0,)).await }
  pub async fn http_request(&self, arg0: HttpRequest) -> Result<
    (HttpResponse,)
  > { ic_cdk::call(self.0, "http_request", (arg0,)).await }
  pub async fn icrc_1_balance_of(&self, arg0: Account) -> Result<(u64,)> {
    ic_cdk::call(self.0, "icrc1_balance_of", (arg0,)).await
  }
  pub async fn ledger_id(&self) -> Result<(Principal,)> {
    ic_cdk::call(self.0, "ledger_id", ()).await
  }
  pub async fn status(&self) -> Result<(Status,)> {
    ic_cdk::call(self.0, "status", ()).await
  }
}
