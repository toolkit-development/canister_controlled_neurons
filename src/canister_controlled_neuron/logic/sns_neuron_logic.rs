use ic_cdk::api::time;
use toolkit_utils::{
    result::CanisterResult,
    storage::{StorageInsertable, StorageQueryable, StorageUpdateable},
};

use crate::{
    storage::{
        icp_neuron_reference_storage::IcpNeuronReferenceStore, log_storage::LogStore,
        sns_neuron_reference_storage::SnsNeuronReferenceStore,
    },
    types::{
        args::sns_neuron_args::SnsNeuronArgs,
        modules::ModuleResponse,
        sns_neuron_reference::{SnsNeuronReference, SnsNeuronReferenceResponse},
    },
};

pub struct SNSNeuronLogic;

impl SNSNeuronLogic {
    pub fn remove_neuron(id: u64) -> CanisterResult<()> {
        IcpNeuronReferenceStore::remove(id);
        Ok(())
    }

    pub fn get_neurons() -> CanisterResult<Vec<SnsNeuronReferenceResponse>> {
        let neurons = SnsNeuronReferenceStore::get_all();
        Ok(neurons
            .into_iter()
            .map(|(id, neuron)| neuron.to_response(id))
            .collect())
    }

    pub async fn create_neuron(
        amount_e8s: u64,
        auto_stake: Option<bool>,
        dissolve_delay: Option<u64>,
    ) -> CanisterResult<SnsNeuronReferenceResponse> {
        let neuron = SnsNeuronReference::new(amount_e8s).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error creating neuron: {}", time(), e));
            e
        })?;
        let (id, mut neuron) = SnsNeuronReferenceStore::insert(neuron.clone())?;

        let claimed_neuron = neuron.claim_or_refresh().await.map_err(|e| {
            let _ = LogStore::insert(format!(
                "{}: Error claiming or refreshing neuron: {}",
                time(),
                e
            ));
            e
        })?;

        if let Some(refreshed_neuron_id) = claimed_neuron.refreshed_neuron_id {
            neuron.neuron_id = Some(refreshed_neuron_id.id);
        }

        let response = SnsNeuronReferenceStore::update(id, neuron.clone())?;

        if let Some(dissolve_delay) = dissolve_delay {
            neuron
                .increase_dissolve_delay(dissolve_delay)
                .await
                .map_err(|e| {
                    let _ = LogStore::insert(format!(
                        "{}: Error setting dissolve delay: {}",
                        time(),
                        e
                    ));
                    e
                })?;
        }

        if let Some(auto_stake) = auto_stake {
            neuron.auto_stake_maturity(auto_stake).await.map_err(|e| {
                let _ = LogStore::insert(format!(
                    "{}: Error setting auto stake maturity: {}",
                    time(),
                    e
                ));
                e
            })?;
        }

        Ok(response.1.to_response(response.0))
    }

    pub async fn top_up_neuron(neuron_id: Vec<u8>, amount_e8s: u64) -> CanisterResult<bool> {
        let (_, mut neuron) = SnsNeuronReferenceStore::get_by_id(neuron_id)?;

        let _ = neuron.top_up(amount_e8s).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error topping up neuron: {}", time(), e));
            e
        })?;

        neuron.claim_or_refresh().await.map_err(|e| {
            let _ = LogStore::insert(format!(
                "{}: Error claiming or refreshing neuron: {}",
                time(),
                e
            ));
            e
        })?;

        Ok(true)
    }

    pub async fn add_dissolve_delay(
        neuron_id: Vec<u8>,
        dissolve_delay: u64,
    ) -> CanisterResult<bool> {
        let (_, neuron) = SnsNeuronReferenceStore::get_by_id(neuron_id)?;
        neuron.increase_dissolve_delay(dissolve_delay).await?;
        let _ = LogStore::insert(format!("Dissolve delay set to {} seconds", dissolve_delay));
        Ok(true)
    }

    pub async fn set_dissolve_state(
        neuron_id: Vec<u8>,
        start_dissolving: bool,
    ) -> CanisterResult<bool> {
        let (_, neuron) = SnsNeuronReferenceStore::get_by_id(neuron_id)?;
        neuron.set_dissolve_state(start_dissolving).await?;
        Ok(true)
    }

    pub async fn auto_stake_maturity(neuron_id: Vec<u8>, auto_stake: bool) -> CanisterResult<bool> {
        let (_, neuron) = SnsNeuronReferenceStore::get_by_id(neuron_id)?;
        neuron.auto_stake_maturity(auto_stake).await?;
        let _ = LogStore::insert(format!("Auto stake maturity set to {}", auto_stake));
        Ok(true)
    }

    // pub async fn spawn_neuron(
    //     identifier: &IcpNeuronIdentifier,
    //     start_dissolving: bool,
    // ) -> CanisterResult<bool> {
    //     let (_, parent_neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
    //     let new_nonce = IcpNeuronReferenceStore::get_latest_key() + 1;
    //     let new_neuron_id = parent_neuron
    //         .spawn(new_nonce)
    //         .await?
    //         .created_neuron_id
    //         .map(|id| id.id);

    //     let subaccount = generate_subaccount_by_nonce(new_nonce);
    //     let spawned_neuron = IcpNeuronReference {
    //         blockheight: 0,
    //         subaccount,
    //         nonce: new_nonce,
    //         neuron_id: new_neuron_id,
    //         parent_subaccount: Some(parent_neuron.subaccount),
    //     };

    //     let _ = IcpNeuronReferenceStore::insert(spawned_neuron.clone())?;

    //     spawned_neuron
    //         .set_publicity(SetVisibility {
    //             visibility: Some(2),
    //         })
    //         .await
    //         .map_err(|e| {
    //             let _ = LogStore::insert(format!(
    //                 "{}: Error setting visibility for spawned neuron: {}",
    //                 time(),
    //                 e
    //             ));
    //             e
    //         })?;

    //     let _ = LogStore::insert(format!(
    //         "{}: Spawned neuron with neuron_id: {:?}",
    //         time(),
    //         new_neuron_id
    //     ));

    //     if start_dissolving {
    //         spawned_neuron.set_dissolve_state(true).await?;
    //         let _ = LogStore::insert(format!(
    //             "{}: Started dissolving neuron with neuron_id: {:?}",
    //             time(),
    //             new_neuron_id
    //         ));
    //     }

    //     Ok(true)
    // }

    // pub async fn create_proposal(
    //     identifier: &IcpNeuronIdentifier,
    //     proposal: MakeProposalRequest,
    // ) -> CanisterResult<MakeProposalResponse> {
    //     let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
    //     let result = neuron.create_proposal(proposal).await.map_err(|e| {
    //         let _ = LogStore::insert(format!("{}: Error creating proposal: {}", time(), e));
    //         e
    //     })?;
    //     Ok(result)
    // }

    // pub async fn vote(
    //     identifier: &IcpNeuronIdentifier,
    //     proposal_id: u64,
    //     vote: Vote,
    // ) -> CanisterResult<bool> {
    //     let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
    //     let result = neuron.vote(proposal_id, vote).await.map_err(|e| {
    //         let _ = LogStore::insert(format!("{}: Error voting: {}", time(), e));
    //         e
    //     })?;

    //     Ok(result)
    // }

    // pub async fn disburse(identifier: &IcpNeuronIdentifier) -> CanisterResult<DisburseResponse> {
    //     let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
    //     neuron.disburse().await
    // }

    // pub async fn set_visibility(
    //     identifier: &IcpNeuronIdentifier,
    //     visibility: i32,
    // ) -> CanisterResult<()> {
    //     let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
    //     neuron
    //         .set_publicity(SetVisibility {
    //             visibility: Some(visibility),
    //         })
    //         .await
    //         .map_err(|e| {
    //             let _ = LogStore::insert(format!("{}: Error setting visibility: {}", time(), e));
    //             e
    //         })?;
    //     Ok(())
    // }

    // pub async fn set_following(
    //     identifier: &IcpNeuronIdentifier,
    //     topic: Topic,
    //     following_neurons: Vec<u64>,
    // ) -> CanisterResult<()> {
    //     let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
    //     neuron.set_following(topic, following_neurons).await
    // }

    // pub async fn get_full_neuron(identifier: &IcpNeuronIdentifier) -> CanisterResult<GovNeuron> {
    //     let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
    //     neuron.get_full_neuron().await
    // }

    // pub async fn list_controlled_neurons() -> CanisterResult<ListNeuronsResponse> {
    //     let (result,) = ApiClients::icp_governance()
    //         .list_neurons(ListNeurons {
    //             page_size: Some(1000),
    //             include_public_neurons_in_full_neurons: None,
    //             neuron_ids: vec![],
    //             page_number: None,
    //             include_empty_neurons_readable_by_caller: None,
    //             neuron_subaccounts: None,
    //             include_neurons_readable_by_caller: true,
    //         })
    //         .await
    //         .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

    //     Ok(result)
    // }

    pub async fn handle_sns_neuron_args(args: SnsNeuronArgs) -> CanisterResult<ModuleResponse> {
        match args {
            SnsNeuronArgs::Create(args) => {
                let result = SNSNeuronLogic::create_neuron(
                    args.amount_e8s,
                    args.auto_stake,
                    args.dissolve_delay,
                )
                .await?;
                Ok(ModuleResponse::SnsNeuron(Box::new(result)))
            }
            SnsNeuronArgs::TopUp(top_up_neuron_args) => {
                let result = SNSNeuronLogic::top_up_neuron(
                    top_up_neuron_args.neuron_id,
                    top_up_neuron_args.amount_e8s,
                )
                .await?;
                Ok(ModuleResponse::Boolean(result))
            }
            SnsNeuronArgs::AddDissolveDelay(add_dissolve_delay_args) => {
                let result = SNSNeuronLogic::add_dissolve_delay(
                    add_dissolve_delay_args.neuron_id,
                    add_dissolve_delay_args.dissolve_delay_seconds,
                )
                .await?;
                Ok(ModuleResponse::Boolean(result))
            }
            SnsNeuronArgs::SetDissolveState(set_dissolve_state_args) => {
                let result = SNSNeuronLogic::set_dissolve_state(
                    set_dissolve_state_args.neuron_id,
                    set_dissolve_state_args.start_dissolving,
                )
                .await?;
                Ok(ModuleResponse::Boolean(result))
            }
            SnsNeuronArgs::AutoStake(auto_stake_args) => {
                let result = SNSNeuronLogic::auto_stake_maturity(
                    auto_stake_args.neuron_id,
                    auto_stake_args.auto_stake,
                )
                .await?;
                Ok(ModuleResponse::Boolean(result))
            }
            SnsNeuronArgs::Spawn(_) => Ok(ModuleResponse::Boolean(true)),
            SnsNeuronArgs::CreateProposal(_) => Ok(ModuleResponse::Boolean(true)),
            SnsNeuronArgs::Vote(_) => Ok(ModuleResponse::Boolean(true)),
            SnsNeuronArgs::Disburse(_) => Ok(ModuleResponse::Boolean(true)),
            SnsNeuronArgs::SetFollowing(_) => Ok(ModuleResponse::Boolean(true)),
        }
    }

    // pub async fn validate_icp_neuron_args(args: IcpNeuronArgs) -> CanisterResult<String> {
    //     match args {
    //         IcpNeuronArgs::Create(args) => {
    //             let balance = get_icp_balance(canister_self()).await?;
    //             if balance.e8s() < args.amount_e8s {
    //                 return Err(ApiError::bad_request("Insufficient balance"));
    //             }

    //             if args.amount_e8s < 100_010_000 {
    //                 return Err(ApiError::bad_request(
    //                     "Amount must be greater than 1 ICP + fee",
    //                 ));
    //             }
    //             Ok(serde_json::to_string(&args).unwrap())
    //         }
    //         IcpNeuronArgs::TopUp(args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
    //             SNSNeuronLogic::get_full_neuron(&args.identifier).await?;
    //             let balance = get_icp_balance(canister_self()).await?;
    //             if balance.e8s() < args.amount_e8s {
    //                 return Err(ApiError::bad_request("Insufficient balance"));
    //             }

    //             if args.amount_e8s < 100_010_000 {
    //                 return Err(ApiError::bad_request(
    //                     "Amount must be greater than 1 ICP + fee",
    //                 ));
    //             }
    //             Ok(serde_json::to_string(&args).unwrap())
    //         }
    //         IcpNeuronArgs::AddDissolveDelay(args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
    //             SNSNeuronLogic::get_full_neuron(&args.identifier).await?;
    //             Ok(serde_json::to_string(&args).unwrap())
    //         }
    //         IcpNeuronArgs::SetDissolveState(args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
    //             SNSNeuronLogic::get_full_neuron(&args.identifier).await?;
    //             Ok(serde_json::to_string(&args).unwrap())
    //         }
    //         IcpNeuronArgs::AutoStake(args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
    //             SNSNeuronLogic::get_full_neuron(&args.identifier).await?;
    //             Ok(serde_json::to_string(&args).unwrap())
    //         }
    //         IcpNeuronArgs::Spawn(args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
    //             let neuron = SNSNeuronLogic::get_full_neuron(&args.identifier).await?;
    //             if neuron.maturity_e8s_equivalent < 100000000 {
    //                 return Err(ApiError::bad_request(
    //                     "neuron must have at least 1 ICP in maturity to spawn",
    //                 ));
    //             }
    //             Ok(serde_json::to_string(&args).unwrap())
    //         }
    //         IcpNeuronArgs::CreateProposal(create_proposal_args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&create_proposal_args.identifier)?;
    //             SNSNeuronLogic::get_full_neuron(&create_proposal_args.identifier).await?;
    //             Ok(serde_json::to_string(&create_proposal_args).unwrap())
    //         }
    //         IcpNeuronArgs::Vote(vote_args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&vote_args.identifier)?;
    //             SNSNeuronLogic::get_full_neuron(&vote_args.identifier).await?;
    //             Ok(serde_json::to_string(&vote_args).unwrap())
    //         }
    //         IcpNeuronArgs::Disburse(disburse_args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&disburse_args.identifier)?;
    //             SNSNeuronLogic::get_full_neuron(&disburse_args.identifier).await?;
    //             Ok(serde_json::to_string(&disburse_args).unwrap())
    //         }
    //         IcpNeuronArgs::SetFollowing(set_following_args) => {
    //             IcpNeuronReferenceStore::get_by_identifier(&set_following_args.identifier)?;
    //             SNSNeuronLogic::get_full_neuron(&set_following_args.identifier).await?;
    //             Ok(serde_json::to_string(&set_following_args).unwrap())
    //         }
    //     }
    // }
}
