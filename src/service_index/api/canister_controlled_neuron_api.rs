// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.

#![allow(dead_code, unused_imports)]
#![allow(deprecated)]// This is an experimental feature to generate Rust binding from Candid.

use candid::{self, CandidType, Deserialize, Principal, Encode, Decode};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub struct Config {
  pub sns_ledger_canister_id: Principal,
  pub governance_canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
pub struct ValidationResponse { pub field: String, pub message: String }

#[derive(CandidType, Deserialize)]
pub enum ApiErrorType {
  Duplicate,
  SerializeError,
  DeserializeError,
  PayloadTooLarge,
  NotFound,
  Deprecated,
  ValidationError(Vec<ValidationResponse>),
  Unsupported,
  Unauthorized,
  ServiceUnavailable,
  Unexpected,
  NotImplemented,
  ExternalServiceError,
  Forbidden,
  BadRequest,
  Conflict,
}

#[derive(CandidType, Deserialize)]
pub struct ApiError {
  pub tag: Option<String>,
  pub source: Option<String>,
  pub info: Option<Vec<String>>,
  pub method_name: Option<String>,
  pub message: String,
  pub timestamp: u64,
  pub error_type: ApiErrorType,
}

#[derive(CandidType, Deserialize)]
pub enum Result_ { Ok(Config), Err(ApiError) }

#[derive(CandidType, Deserialize)]
pub struct NeuronId { pub id: u64 }

#[derive(CandidType, Deserialize)]
pub struct BallotInfo { pub vote: i32, pub proposal_id: Option<NeuronId> }

#[derive(CandidType, Deserialize)]
pub enum DissolveState {
  DissolveDelaySeconds(u64),
  WhenDissolvedTimestampSeconds(u64),
}

#[derive(CandidType, Deserialize)]
pub struct Followees { pub followees: Vec<NeuronId> }

#[derive(CandidType, Deserialize)]
pub struct NeuronStakeTransfer {
  pub to_subaccount: serde_bytes::ByteBuf,
  pub neuron_stake_e8s: u64,
  pub from: Option<Principal>,
  pub memo: u64,
  pub from_subaccount: serde_bytes::ByteBuf,
  pub transfer_timestamp: u64,
  pub block_height: u64,
}

#[derive(CandidType, Deserialize)]
pub struct KnownNeuronData { pub name: String, pub description: Option<String> }

#[derive(CandidType, Deserialize)]
pub struct Neuron {
  pub id: Option<NeuronId>,
  pub staked_maturity_e8s_equivalent: Option<u64>,
  pub controller: Option<Principal>,
  pub recent_ballots: Vec<BallotInfo>,
  pub voting_power_refreshed_timestamp_seconds: Option<u64>,
  pub kyc_verified: bool,
  pub potential_voting_power: Option<u64>,
  pub neuron_type: Option<i32>,
  pub not_for_profit: bool,
  pub maturity_e8s_equivalent: u64,
  pub deciding_voting_power: Option<u64>,
  pub cached_neuron_stake_e8s: u64,
  pub created_timestamp_seconds: u64,
  pub auto_stake_maturity: Option<bool>,
  pub aging_since_timestamp_seconds: u64,
  pub hot_keys: Vec<Principal>,
  pub account: serde_bytes::ByteBuf,
  pub joined_community_fund_timestamp_seconds: Option<u64>,
  pub dissolve_state: Option<DissolveState>,
  pub followees: Vec<(i32,Followees,)>,
  pub neuron_fees_e8s: u64,
  pub visibility: Option<i32>,
  pub transfer: Option<NeuronStakeTransfer>,
  pub known_neuron_data: Option<KnownNeuronData>,
  pub spawn_at_timestamp_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum Result1 { Ok(Neuron), Err(ApiError) }

#[derive(CandidType, Deserialize)]
pub struct Account {
  pub owner: Option<Principal>,
  pub subaccount: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronReferenceResponse {
  pub subaccount: serde_bytes::ByteBuf,
  pub blockheight: u64,
  pub topup_account: Account,
  pub nonce: u64,
  pub storage_reference_id: u64,
  pub parent_subaccount: Option<serde_bytes::ByteBuf>,
  pub neuron_id: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub enum Result2 { Ok(Vec<NeuronReferenceResponse>), Err(ApiError) }

#[derive(CandidType, Deserialize)]
pub struct SupportedStandard { pub url: String, pub name: String }

#[derive(CandidType, Deserialize)]
pub struct Icrc28TrustedOriginsResponse { pub trusted_origins: Vec<String> }

#[derive(CandidType, Deserialize)]
pub struct AutoStakeArgs {
  pub subaccount: serde_bytes::ByteBuf,
  pub auto_stake: bool,
}

#[derive(CandidType, Deserialize)]
pub struct SpawnArgs {
  pub start_dissolving: bool,
  pub parent_subaccount: serde_bytes::ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct AddDissolveDelayArgs {
  pub dissolve_delay_seconds: u64,
  pub subaccount: serde_bytes::ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub enum Vote { Approve, Reject }

#[derive(CandidType, Deserialize)]
pub struct VoteArgs {
  pub vote: Vote,
  pub subaccount: serde_bytes::ByteBuf,
  pub proposal_id: u64,
}

#[derive(CandidType, Deserialize)]
pub struct SetDissolveStateArgs {
  pub start_dissolving: bool,
  pub subaccount: serde_bytes::ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct TopUpNeuronArgs {
  pub subaccount: serde_bytes::ByteBuf,
  pub amount_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct KnownNeuron {
  pub id: Option<NeuronId>,
  pub known_neuron_data: Option<KnownNeuronData>,
}

#[derive(CandidType, Deserialize)]
pub struct Spawn {
  pub percentage_to_spawn: Option<u32>,
  pub new_controller: Option<Principal>,
  pub nonce: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Split { pub amount_e8s: u64 }

#[derive(CandidType, Deserialize)]
pub struct Follow { pub topic: i32, pub followees: Vec<NeuronId> }

#[derive(CandidType, Deserialize)]
pub struct DisburseMaturity {
  pub to_account: Option<Account>,
  pub percentage_to_disburse: u32,
}

#[derive(CandidType, Deserialize)]
pub struct ClaimOrRefreshNeuronFromAccount {
  pub controller: Option<Principal>,
  pub memo: u64,
}

#[derive(CandidType, Deserialize)]
pub enum By {
  NeuronIdOrSubaccount{},
  MemoAndController(ClaimOrRefreshNeuronFromAccount),
  Memo(u64),
}

#[derive(CandidType, Deserialize)]
pub struct ClaimOrRefresh { pub by: Option<By> }

#[derive(CandidType, Deserialize)]
pub struct RemoveHotKey { pub hot_key_to_remove: Option<Principal> }

#[derive(CandidType, Deserialize)]
pub struct AddHotKey { pub new_hot_key: Option<Principal> }

#[derive(CandidType, Deserialize)]
pub struct ChangeAutoStakeMaturity {
  pub requested_setting_for_auto_stake_maturity: bool,
}

#[derive(CandidType, Deserialize)]
pub struct IncreaseDissolveDelay { pub additional_dissolve_delay_seconds: u32 }

#[derive(CandidType, Deserialize)]
pub struct SetVisibility { pub visibility: Option<i32> }

#[derive(CandidType, Deserialize)]
pub struct SetDissolveTimestamp { pub dissolve_timestamp_seconds: u64 }

#[derive(CandidType, Deserialize)]
pub enum Operation {
  RemoveHotKey(RemoveHotKey),
  AddHotKey(AddHotKey),
  ChangeAutoStakeMaturity(ChangeAutoStakeMaturity),
  StopDissolving{},
  StartDissolving{},
  IncreaseDissolveDelay(IncreaseDissolveDelay),
  SetVisibility(SetVisibility),
  JoinCommunityFund{},
  LeaveCommunityFund{},
  SetDissolveTimestamp(SetDissolveTimestamp),
}

#[derive(CandidType, Deserialize)]
pub struct Configure { pub operation: Option<Operation> }

#[derive(CandidType, Deserialize)]
pub struct RegisterVote { pub vote: i32, pub proposal: Option<NeuronId> }

#[derive(CandidType, Deserialize)]
pub struct Merge { pub source_neuron_id: Option<NeuronId> }

#[derive(CandidType, Deserialize)]
pub struct DisburseToNeuron {
  pub dissolve_delay_seconds: u64,
  pub kyc_verified: bool,
  pub amount_e8s: u64,
  pub new_controller: Option<Principal>,
  pub nonce: u64,
}

#[derive(CandidType, Deserialize)]
pub struct StakeMaturity { pub percentage_to_stake: Option<u32> }

#[derive(CandidType, Deserialize)]
pub struct MergeMaturity { pub percentage_to_merge: u32 }

#[derive(CandidType, Deserialize)]
pub struct AccountIdentifier { pub hash: serde_bytes::ByteBuf }

#[derive(CandidType, Deserialize)]
pub struct Amount { pub e8s: u64 }

#[derive(CandidType, Deserialize)]
pub struct Disburse {
  pub to_account: Option<AccountIdentifier>,
  pub amount: Option<Amount>,
}

#[derive(CandidType, Deserialize)]
pub enum ManageNeuronCommandRequest {
  Spawn(Spawn),
  Split(Split),
  Follow(Follow),
  DisburseMaturity(DisburseMaturity),
  RefreshVotingPower{},
  ClaimOrRefresh(ClaimOrRefresh),
  Configure(Configure),
  RegisterVote(RegisterVote),
  Merge(Merge),
  DisburseToNeuron(DisburseToNeuron),
  MakeProposal(Box<MakeProposalRequest>),
  StakeMaturity(StakeMaturity),
  MergeMaturity(MergeMaturity),
  Disburse(Disburse),
}

#[derive(CandidType, Deserialize)]
pub enum NeuronIdOrSubaccount {
  Subaccount(serde_bytes::ByteBuf),
  NeuronId(NeuronId),
}

#[derive(CandidType, Deserialize)]
pub struct ManageNeuronRequest {
  pub id: Option<NeuronId>,
  pub command: Option<ManageNeuronCommandRequest>,
  pub neuron_id_or_subaccount: Option<NeuronIdOrSubaccount>,
}

#[derive(CandidType, Deserialize)]
pub struct Controllers { pub controllers: Vec<Principal> }

#[derive(CandidType, Deserialize)]
pub struct CanisterSettings {
  pub freezing_threshold: Option<u64>,
  pub wasm_memory_threshold: Option<u64>,
  pub controllers: Option<Controllers>,
  pub log_visibility: Option<i32>,
  pub wasm_memory_limit: Option<u64>,
  pub memory_allocation: Option<u64>,
  pub compute_allocation: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct UpdateCanisterSettings {
  pub canister_id: Option<Principal>,
  pub settings: Option<CanisterSettings>,
}

#[derive(CandidType, Deserialize)]
pub struct InstallCodeRequest {
  pub arg: Option<serde_bytes::ByteBuf>,
  pub wasm_module: Option<serde_bytes::ByteBuf>,
  pub skip_stopping_before_installing: Option<bool>,
  pub canister_id: Option<Principal>,
  pub install_mode: Option<i32>,
}

#[derive(CandidType, Deserialize)]
pub struct StopOrStartCanister {
  pub action: Option<i32>,
  pub canister_id: Option<Principal>,
}

#[derive(CandidType, Deserialize)]
pub struct Percentage { pub basis_points: Option<u64> }

#[derive(CandidType, Deserialize)]
pub struct Duration { pub seconds: Option<u64> }

#[derive(CandidType, Deserialize)]
pub struct Tokens { pub e8s: Option<u64> }

#[derive(CandidType, Deserialize)]
pub struct VotingRewardParameters {
  pub reward_rate_transition_duration: Option<Duration>,
  pub initial_reward_rate: Option<Percentage>,
  pub final_reward_rate: Option<Percentage>,
}

#[derive(CandidType, Deserialize)]
pub struct GovernanceParameters {
  pub neuron_maximum_dissolve_delay_bonus: Option<Percentage>,
  pub neuron_maximum_age_for_age_bonus: Option<Duration>,
  pub neuron_maximum_dissolve_delay: Option<Duration>,
  pub neuron_minimum_dissolve_delay_to_vote: Option<Duration>,
  pub neuron_maximum_age_bonus: Option<Percentage>,
  pub neuron_minimum_stake: Option<Tokens>,
  pub proposal_wait_for_quiet_deadline_increase: Option<Duration>,
  pub proposal_initial_voting_period: Option<Duration>,
  pub proposal_rejection_fee: Option<Tokens>,
  pub voting_reward_parameters: Option<VotingRewardParameters>,
}

#[derive(CandidType, Deserialize)]
pub struct Image { pub base64_encoding: Option<String> }

#[derive(CandidType, Deserialize)]
pub struct LedgerParameters {
  pub transaction_fee: Option<Tokens>,
  pub token_symbol: Option<String>,
  pub token_logo: Option<Image>,
  pub token_name: Option<String>,
}

#[derive(CandidType, Deserialize)]
pub struct Canister { pub id: Option<Principal> }

#[derive(CandidType, Deserialize)]
pub struct NeuronBasketConstructionParameters {
  pub dissolve_delay_interval: Option<Duration>,
  pub count: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct GlobalTimeOfDay { pub seconds_after_utc_midnight: Option<u64> }

#[derive(CandidType, Deserialize)]
pub struct Countries { pub iso_codes: Vec<String> }

#[derive(CandidType, Deserialize)]
pub struct SwapParameters {
  pub minimum_participants: Option<u64>,
  pub neurons_fund_participation: Option<bool>,
  pub duration: Option<Duration>,
  pub neuron_basket_construction_parameters: Option<
    NeuronBasketConstructionParameters
  >,
  pub confirmation_text: Option<String>,
  pub maximum_participant_icp: Option<Tokens>,
  pub minimum_icp: Option<Tokens>,
  pub minimum_direct_participation_icp: Option<Tokens>,
  pub minimum_participant_icp: Option<Tokens>,
  pub start_time: Option<GlobalTimeOfDay>,
  pub maximum_direct_participation_icp: Option<Tokens>,
  pub maximum_icp: Option<Tokens>,
  pub neurons_fund_investment_icp: Option<Tokens>,
  pub restricted_countries: Option<Countries>,
}

#[derive(CandidType, Deserialize)]
pub struct SwapDistribution { pub total: Option<Tokens> }

#[derive(CandidType, Deserialize)]
pub struct NeuronDistribution {
  pub controller: Option<Principal>,
  pub dissolve_delay: Option<Duration>,
  pub memo: Option<u64>,
  pub vesting_period: Option<Duration>,
  pub stake: Option<Tokens>,
}

#[derive(CandidType, Deserialize)]
pub struct DeveloperDistribution {
  pub developer_neurons: Vec<NeuronDistribution>,
}

#[derive(CandidType, Deserialize)]
pub struct InitialTokenDistribution {
  pub treasury_distribution: Option<SwapDistribution>,
  pub developer_distribution: Option<DeveloperDistribution>,
  pub swap_distribution: Option<SwapDistribution>,
}

#[derive(CandidType, Deserialize)]
pub struct CreateServiceNervousSystem {
  pub url: Option<String>,
  pub governance_parameters: Option<GovernanceParameters>,
  pub fallback_controller_principal_ids: Vec<Principal>,
  pub logo: Option<Image>,
  pub name: Option<String>,
  pub ledger_parameters: Option<LedgerParameters>,
  pub description: Option<String>,
  pub dapp_canisters: Vec<Canister>,
  pub swap_parameters: Option<SwapParameters>,
  pub initial_token_distribution: Option<InitialTokenDistribution>,
}

#[derive(CandidType, Deserialize)]
pub struct ExecuteNnsFunction {
  pub nns_function: i32,
  pub payload: serde_bytes::ByteBuf,
}

#[derive(CandidType, Deserialize)]
pub struct NodeProvider {
  pub id: Option<Principal>,
  pub reward_account: Option<AccountIdentifier>,
}

#[derive(CandidType, Deserialize)]
pub struct RewardToNeuron { pub dissolve_delay_seconds: u64 }

#[derive(CandidType, Deserialize)]
pub struct RewardToAccount { pub to_account: Option<AccountIdentifier> }

#[derive(CandidType, Deserialize)]
pub enum RewardMode {
  RewardToNeuron(RewardToNeuron),
  RewardToAccount(RewardToAccount),
}

#[derive(CandidType, Deserialize)]
pub struct RewardNodeProvider {
  pub node_provider: Option<NodeProvider>,
  pub reward_mode: Option<RewardMode>,
  pub amount_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct RewardNodeProviders {
  pub use_registry_derived_rewards: Option<bool>,
  pub rewards: Vec<RewardNodeProvider>,
}

#[derive(CandidType, Deserialize)]
pub struct VotingPowerEconomics {
  pub start_reducing_voting_power_after_seconds: Option<u64>,
  pub neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
  pub clear_following_after_seconds: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct Decimal { pub human_readable: Option<String> }

#[derive(CandidType, Deserialize)]
pub struct NeuronsFundMatchedFundingCurveCoefficients {
  pub contribution_threshold_xdr: Option<Decimal>,
  pub one_third_participation_milestone_xdr: Option<Decimal>,
  pub full_participation_milestone_xdr: Option<Decimal>,
}

#[derive(CandidType, Deserialize)]
pub struct NeuronsFundEconomics {
  pub maximum_icp_xdr_rate: Option<Percentage>,
  pub neurons_fund_matched_funding_curve_coefficients: Option<
    NeuronsFundMatchedFundingCurveCoefficients
  >,
  pub max_theoretical_neurons_fund_participation_amount_xdr: Option<Decimal>,
  pub minimum_icp_xdr_rate: Option<Percentage>,
}

#[derive(CandidType, Deserialize)]
pub struct NetworkEconomics {
  pub neuron_minimum_stake_e8s: u64,
  pub voting_power_economics: Option<VotingPowerEconomics>,
  pub max_proposals_to_keep_per_topic: u32,
  pub neuron_management_fee_per_proposal_e8s: u64,
  pub reject_cost_e8s: u64,
  pub transaction_fee_e8s: u64,
  pub neuron_spawn_dissolve_delay_seconds: u64,
  pub minimum_icp_xdr_rate: u64,
  pub maximum_node_provider_rewards_e8s: u64,
  pub neurons_fund_economics: Option<NeuronsFundEconomics>,
}

#[derive(CandidType, Deserialize)]
pub struct Principals { pub principals: Vec<Principal> }

#[derive(CandidType, Deserialize)]
pub enum Change { ToRemove(NodeProvider), ToAdd(NodeProvider) }

#[derive(CandidType, Deserialize)]
pub struct AddOrRemoveNodeProvider { pub change: Option<Change> }

#[derive(CandidType, Deserialize)]
pub struct Motion { pub motion_text: String }

#[derive(CandidType, Deserialize)]
pub enum ProposalActionRequest {
  RegisterKnownNeuron(KnownNeuron),
  ManageNeuron(ManageNeuronRequest),
  UpdateCanisterSettings(UpdateCanisterSettings),
  InstallCode(InstallCodeRequest),
  StopOrStartCanister(StopOrStartCanister),
  CreateServiceNervousSystem(CreateServiceNervousSystem),
  ExecuteNnsFunction(ExecuteNnsFunction),
  RewardNodeProvider(RewardNodeProvider),
  RewardNodeProviders(RewardNodeProviders),
  ManageNetworkEconomics(NetworkEconomics),
  ApproveGenesisKyc(Principals),
  AddOrRemoveNodeProvider(AddOrRemoveNodeProvider),
  Motion(Motion),
}

#[derive(CandidType, Deserialize)]
pub struct MakeProposalRequest {
  pub url: String,
  pub title: Option<String>,
  pub action: Option<ProposalActionRequest>,
  pub summary: String,
}

#[derive(CandidType, Deserialize)]
pub struct CreateProposalArgs {
  pub subaccount: serde_bytes::ByteBuf,
  pub proposal: Box<MakeProposalRequest>,
}

#[derive(CandidType, Deserialize)]
pub struct CreateNeuronArgs {
  pub dissolve_delay_seconds: Option<u64>,
  pub amount_e8s: u64,
  pub auto_stake: Option<bool>,
}

#[derive(CandidType, Deserialize)]
pub enum Topic {
  Kyc,
  ServiceNervousSystemManagement,
  NetworkCanisterManagement,
  ApiBoundaryNodeManagement,
  SubnetRental,
  NeuronManagement,
  NodeProviderRewards,
  SubnetManagement,
  ExchangeRate,
  NodeAdmin,
  IcOsVersionElection,
  ProtocolCanisterManagement,
  NetworkEconomics,
  IcOsVersionDeployment,
  ParticipantManagement,
  Governance,
  SnsAndCommunityFund,
  Unspecified,
}

#[derive(CandidType, Deserialize)]
pub struct FollowingArgs { pub topic: Topic, pub followees: Vec<u64> }

#[derive(CandidType, Deserialize)]
pub struct SetFollowingArgs {
  pub subaccount: serde_bytes::ByteBuf,
  pub following: Vec<FollowingArgs>,
}

#[derive(CandidType, Deserialize)]
pub struct DisburseArgs { pub subaccount: serde_bytes::ByteBuf }

#[derive(CandidType, Deserialize)]
pub enum IcpNeuronArgs {
  AutoStake(AutoStakeArgs),
  Spawn(SpawnArgs),
  AddDissolveDelay(AddDissolveDelayArgs),
  Vote(VoteArgs),
  SetDissolveState(SetDissolveStateArgs),
  TopUp(TopUpNeuronArgs),
  CreateProposal(CreateProposalArgs),
  Create(CreateNeuronArgs),
  SetFollowing(SetFollowingArgs),
  Disburse(DisburseArgs),
}

#[derive(CandidType, Deserialize)]
pub enum NeuronType { Icp(IcpNeuronArgs) }

#[derive(CandidType, Deserialize)]
pub struct MakeProposalResponse {
  pub message: Option<String>,
  pub proposal_id: Option<NeuronId>,
}

#[derive(CandidType, Deserialize)]
pub struct GovernanceError { pub error_message: String, pub error_type: i32 }

#[derive(CandidType, Deserialize)]
pub struct SpawnResponse { pub created_neuron_id: Option<NeuronId> }

#[derive(CandidType, Deserialize)]
pub struct DisburseMaturityResponse { pub amount_disbursed_e8s: Option<u64> }

#[derive(CandidType, Deserialize)]
pub struct ClaimOrRefreshResponse { pub refreshed_neuron_id: Option<NeuronId> }

#[derive(CandidType, Deserialize)]
pub struct NeuronInfo {
  pub dissolve_delay_seconds: u64,
  pub recent_ballots: Vec<BallotInfo>,
  pub voting_power_refreshed_timestamp_seconds: Option<u64>,
  pub potential_voting_power: Option<u64>,
  pub neuron_type: Option<i32>,
  pub deciding_voting_power: Option<u64>,
  pub created_timestamp_seconds: u64,
  pub state: i32,
  pub stake_e8s: u64,
  pub joined_community_fund_timestamp_seconds: Option<u64>,
  pub retrieved_at_timestamp_seconds: u64,
  pub visibility: Option<i32>,
  pub known_neuron_data: Option<KnownNeuronData>,
  pub voting_power: u64,
  pub age_seconds: u64,
}

#[derive(CandidType, Deserialize)]
pub struct MergeResponse {
  pub target_neuron: Option<Neuron>,
  pub source_neuron: Option<Neuron>,
  pub target_neuron_info: Option<NeuronInfo>,
  pub source_neuron_info: Option<NeuronInfo>,
}

#[derive(CandidType, Deserialize)]
pub struct StakeMaturityResponse {
  pub maturity_e8s: u64,
  pub staked_maturity_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct MergeMaturityResponse {
  pub merged_maturity_e8s: u64,
  pub new_stake_e8s: u64,
}

#[derive(CandidType, Deserialize)]
pub struct DisburseResponse { pub transfer_block_height: u64 }

#[derive(CandidType, Deserialize)]
pub enum Command1 {
  Error(GovernanceError),
  Spawn(SpawnResponse),
  Split(SpawnResponse),
  Follow{},
  DisburseMaturity(DisburseMaturityResponse),
  RefreshVotingPower{},
  ClaimOrRefresh(ClaimOrRefreshResponse),
  Configure{},
  RegisterVote{},
  Merge(MergeResponse),
  DisburseToNeuron(SpawnResponse),
  MakeProposal(MakeProposalResponse),
  StakeMaturity(StakeMaturityResponse),
  MergeMaturity(MergeMaturityResponse),
  Disburse(DisburseResponse),
}

#[derive(CandidType, Deserialize)]
pub struct ManageNeuronResponse { pub command: Option<Command1> }

#[derive(CandidType, Deserialize)]
pub enum ModuleResponse {
  Boolean(bool),
  MakeProposalResponse(MakeProposalResponse),
  BlockHeight(u64),
  Neuron(NeuronReferenceResponse),
  ManageNeuronResponse(ManageNeuronResponse),
}

#[derive(CandidType, Deserialize)]
pub enum Result3 { Ok(ModuleResponse), Err(ApiError) }

#[derive(CandidType, Deserialize)]
pub enum Result4 { Ok(String), Err(String) }

pub struct CanisterControlledNeuronApi(pub Principal);
impl CanisterControlledNeuronApi {
  pub async fn get_candid_interface_tmp_hack(&self) -> Result<(String,)> {
    ic_cdk::call(self.0, "__get_candid_interface_tmp_hack", ()).await
  }
  pub async fn get_config(&self) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "get_config", ()).await
  }
  pub async fn get_full_neuron(&self, arg0: serde_bytes::ByteBuf) -> Result<
    (Result1,)
  > { ic_cdk::call(self.0, "get_full_neuron", (arg0,)).await }
  pub async fn get_logs(&self) -> Result<(Vec<String>,)> {
    ic_cdk::call(self.0, "get_logs", ()).await
  }
  pub async fn get_neuron_references(&self) -> Result<(Result2,)> {
    ic_cdk::call(self.0, "get_neuron_references", ()).await
  }
  pub async fn icrc_10_supported_standards(&self) -> Result<
    (Vec<SupportedStandard>,)
  > { ic_cdk::call(self.0, "icrc10_supported_standards", ()).await }
  pub async fn icrc_28_trusted_origins(&self) -> Result<
    (Icrc28TrustedOriginsResponse,)
  > { ic_cdk::call(self.0, "icrc28_trusted_origins", ()).await }
  pub async fn icts_description(&self) -> Result<(String,)> {
    ic_cdk::call(self.0, "icts_description", ()).await
  }
  pub async fn icts_name(&self) -> Result<(String,)> {
    ic_cdk::call(self.0, "icts_name", ()).await
  }
  pub async fn icts_version(&self) -> Result<(String,)> {
    ic_cdk::call(self.0, "icts_version", ()).await
  }
  pub async fn tk_service_manage_neuron(&self, arg0: NeuronType) -> Result<
    (Result3,)
  > { ic_cdk::call(self.0, "tk_service_manage_neuron", (arg0,)).await }
  pub async fn tk_service_validate_manage_neuron(
    &self,
    arg0: NeuronType,
  ) -> Result<(Result4,)> {
    ic_cdk::call(self.0, "tk_service_validate_manage_neuron", (arg0,)).await
  }
}
