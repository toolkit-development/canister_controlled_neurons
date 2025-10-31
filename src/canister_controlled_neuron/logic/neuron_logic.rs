use ic_cdk::api::{canister_self, time};
use toolkit_utils::{
    api_error::ApiError,
    result::CanisterResult,
    storage::{StorageInsertable, StorageQueryable, StorageUpdateable},
    transactions::get_icp_balance,
};

use crate::{
    api::{
        api_clients::ApiClients,
        icp_governance_api::{
            DisburseResponse, ListNeurons, ListNeuronsResponse, MakeProposalRequest,
            MakeProposalResponse, ManageNeuronCommandRequest, ManageNeuronResponse,
            Neuron as GovNeuron, SetVisibility,
        },
    },
    helpers::subaccount_helper::generate_subaccount_by_nonce,
    storage::{log_storage::LogStore, neuron_reference_storage::NeuronReferenceStore},
    types::{
        modules::{IcpNeuronArgs, ModuleResponse, NeuronType, Vote},
        neuron_reference::{NeuronReference, NeuronReferenceResponse},
        topic::Topic,
    },
};

pub struct NeuronLogic;

impl NeuronLogic {
    pub fn remove_neuron(id: u64) -> CanisterResult<()> {
        NeuronReferenceStore::remove(id);
        Ok(())
    }

    pub fn get_neurons() -> CanisterResult<Vec<NeuronReferenceResponse>> {
        let neurons = NeuronReferenceStore::get_all();
        Ok(neurons
            .into_iter()
            .map(|(id, neuron)| neuron.to_response(id))
            .collect())
    }

    pub async fn create_neuron(
        amount_e8s: u64,
        auto_stake: Option<bool>,
        dissolve_delay: Option<u64>,
    ) -> CanisterResult<NeuronReferenceResponse> {
        let neuron = NeuronReference::new(amount_e8s).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error creating neuron: {}", time(), e));
            e
        })?;
        let (id, mut neuron) = NeuronReferenceStore::insert(neuron.clone())?;

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

        let response = NeuronReferenceStore::update(id, neuron.clone())?;

        // Make neuron public by default
        neuron
            .set_publicity(SetVisibility {
                visibility: Some(2),
            })
            .await
            .map_err(|e| {
                let _ = LogStore::insert(format!("{}: Error setting visibility: {}", time(), e));
                e
            })?;

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

    pub async fn top_up_neuron_by_subaccount(
        subaccount: [u8; 32],
        amount_e8s: u64,
    ) -> CanisterResult<bool> {
        let (_, mut neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;

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

    pub async fn command_neuron(
        subaccount: [u8; 32],
        command: ManageNeuronCommandRequest,
    ) -> CanisterResult<ManageNeuronResponse> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        let neuron = neuron.command(command).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error commanding neuron: {}", time(), e));
            e
        })?;
        Ok(neuron)
    }

    pub async fn add_dissolve_delay(
        subaccount: [u8; 32],
        dissolve_delay: u64,
    ) -> CanisterResult<bool> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.increase_dissolve_delay(dissolve_delay).await?;
        let _ = LogStore::insert(format!("Dissolve delay set to {} seconds", dissolve_delay));
        Ok(true)
    }

    pub async fn set_dissolve_state(
        subaccount: [u8; 32],
        start_dissolving: bool,
    ) -> CanisterResult<bool> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.set_dissolve_state(start_dissolving).await?;
        Ok(true)
    }

    pub async fn auto_stake_maturity(
        subaccount: [u8; 32],
        auto_stake: bool,
    ) -> CanisterResult<bool> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.auto_stake_maturity(auto_stake).await?;
        let _ = LogStore::insert(format!("Auto stake maturity set to {}", auto_stake));
        Ok(true)
    }

    pub async fn spawn_neuron(
        parent_subaccount: [u8; 32],
        start_dissolving: bool,
    ) -> CanisterResult<bool> {
        let (_, parent_neuron) = NeuronReferenceStore::get_by_subaccount(parent_subaccount)?;
        let new_nonce = NeuronReferenceStore::get_latest_key() + 1;
        let new_neuron_id = parent_neuron
            .spawn(new_nonce)
            .await?
            .created_neuron_id
            .map(|id| id.id);

        let subaccount = generate_subaccount_by_nonce(new_nonce);
        let spawned_neuron = NeuronReference {
            blockheight: 0,
            subaccount,
            nonce: new_nonce,
            neuron_id: new_neuron_id,
            parent_subaccount: Some(parent_subaccount),
        };

        let _ = NeuronReferenceStore::insert(spawned_neuron.clone())?;

        spawned_neuron
            .set_publicity(SetVisibility {
                visibility: Some(2),
            })
            .await
            .map_err(|e| {
                let _ = LogStore::insert(format!(
                    "{}: Error setting visibility for spawned neuron: {}",
                    time(),
                    e
                ));
                e
            })?;

        let _ = LogStore::insert(format!(
            "{}: Spawned neuron with neuron_id: {:?}",
            time(),
            new_neuron_id
        ));

        if start_dissolving {
            spawned_neuron.set_dissolve_state(true).await?;
            let _ = LogStore::insert(format!(
                "{}: Started dissolving neuron with neuron_id: {:?}",
                time(),
                new_neuron_id
            ));
        }

        Ok(true)
    }

    pub async fn create_proposal(
        subaccount: [u8; 32],
        proposal: MakeProposalRequest,
    ) -> CanisterResult<MakeProposalResponse> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        let result = neuron.create_proposal(proposal).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error creating proposal: {}", time(), e));
            e
        })?;
        Ok(result)
    }

    pub async fn vote(subaccount: [u8; 32], proposal_id: u64, vote: Vote) -> CanisterResult<bool> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        let result = neuron.vote(proposal_id, vote).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error voting: {}", time(), e));
            e
        })?;

        Ok(result)
    }

    pub async fn disburse(subaccount: [u8; 32]) -> CanisterResult<DisburseResponse> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.disburse().await
    }

    pub async fn set_visibility(subaccount: [u8; 32], visibility: i32) -> CanisterResult<()> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron
            .set_publicity(SetVisibility {
                visibility: Some(visibility),
            })
            .await
            .map_err(|e| {
                let _ = LogStore::insert(format!("{}: Error setting visibility: {}", time(), e));
                e
            })?;
        Ok(())
    }

    pub async fn set_following(
        subaccount: [u8; 32],
        topic: Topic,
        following_neurons: Vec<u64>,
    ) -> CanisterResult<()> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.set_following(topic, following_neurons).await
    }

    pub async fn get_full_neuron(subaccount: [u8; 32]) -> CanisterResult<GovNeuron> {
        let (_, neuron) = NeuronReferenceStore::get_by_subaccount(subaccount)?;
        neuron.get_full_neuron().await
    }

    pub async fn list_controlled_neurons() -> CanisterResult<ListNeuronsResponse> {
        let (result,) = ApiClients::icp_governance()
            .list_neurons(ListNeurons {
                page_size: Some(1000),
                include_public_neurons_in_full_neurons: None,
                neuron_ids: vec![],
                page_number: None,
                include_empty_neurons_readable_by_caller: None,
                neuron_subaccounts: None,
                include_neurons_readable_by_caller: true,
            })
            .await
            .map_err(|(_, e)| ApiError::external_service_error(e.as_str()))?;

        Ok(result)
    }

    pub async fn tk_service_manage_neuron(module: NeuronType) -> CanisterResult<ModuleResponse> {
        match module {
            NeuronType::Icp(module) => match module {
                IcpNeuronArgs::Create(args) => {
                    let result = NeuronLogic::create_neuron(
                        args.amount_e8s,
                        args.auto_stake,
                        args.dissolve_delay_seconds,
                    )
                    .await?;
                    Ok(ModuleResponse::Neuron(Box::new(result)))
                }
                IcpNeuronArgs::TopUp(args) => {
                    let result =
                        NeuronLogic::top_up_neuron_by_subaccount(args.subaccount, args.amount_e8s)
                            .await?;
                    Ok(ModuleResponse::Boolean(result))
                }
                IcpNeuronArgs::AddDissolveDelay(args) => {
                    let result = NeuronLogic::add_dissolve_delay(
                        args.subaccount,
                        args.dissolve_delay_seconds,
                    )
                    .await?;
                    Ok(ModuleResponse::Boolean(result))
                }
                IcpNeuronArgs::SetDissolveState(args) => {
                    let result =
                        NeuronLogic::set_dissolve_state(args.subaccount, args.start_dissolving)
                            .await?;
                    Ok(ModuleResponse::Boolean(result))
                }
                IcpNeuronArgs::AutoStake(args) => {
                    let result =
                        NeuronLogic::auto_stake_maturity(args.subaccount, args.auto_stake).await?;
                    Ok(ModuleResponse::Boolean(result))
                }
                IcpNeuronArgs::Spawn(args) => {
                    let result =
                        NeuronLogic::spawn_neuron(args.parent_subaccount, args.start_dissolving)
                            .await?;
                    Ok(ModuleResponse::Boolean(result))
                }
                IcpNeuronArgs::CreateProposal(args) => {
                    let result =
                        NeuronLogic::create_proposal(args.subaccount, args.proposal).await?;
                    Ok(ModuleResponse::MakeProposalResponse(Box::new(result)))
                }
                IcpNeuronArgs::Vote(args) => {
                    let result =
                        NeuronLogic::vote(args.subaccount, args.proposal_id, args.vote).await?;
                    Ok(ModuleResponse::Boolean(result))
                }
                IcpNeuronArgs::Disburse(args) => {
                    let _ = NeuronLogic::disburse(args.subaccount).await?;
                    Ok(ModuleResponse::Boolean(true))
                }
                IcpNeuronArgs::SetFollowing(set_following_args) => {
                    for arg in set_following_args.following {
                        NeuronLogic::set_following(
                            set_following_args.subaccount,
                            arg.topic,
                            arg.followees,
                        )
                        .await?;
                    }
                    Ok(ModuleResponse::Boolean(true))
                }
                IcpNeuronArgs::Command(args) => {
                    let result = NeuronLogic::command_neuron(args.subaccount, args.command).await?;
                    Ok(ModuleResponse::ManageNeuronResponse(Box::new(result)))
                }
            },
        }
    }

    pub async fn tk_service_validate_manage_neuron(args: NeuronType) -> CanisterResult<String> {
        match args {
            NeuronType::Icp(args) => match args {
                IcpNeuronArgs::Create(args) => {
                    let balance = get_icp_balance(canister_self()).await?;
                    if balance.e8s() < args.amount_e8s {
                        return Err(ApiError::bad_request("Insufficient balance"));
                    }

                    if args.amount_e8s < 100_010_000 {
                        return Err(ApiError::bad_request(
                            "Amount must be greater than 1 ICP + fee",
                        ));
                    }
                    Ok(serde_json::to_string(&args).unwrap())
                }
                IcpNeuronArgs::TopUp(args) => {
                    NeuronReferenceStore::get_by_subaccount(args.subaccount)?;
                    NeuronLogic::get_full_neuron(args.subaccount).await?;
                    let balance = get_icp_balance(canister_self()).await?;
                    if balance.e8s() < args.amount_e8s {
                        return Err(ApiError::bad_request("Insufficient balance"));
                    }

                    if args.amount_e8s < 100_010_000 {
                        return Err(ApiError::bad_request(
                            "Amount must be greater than 1 ICP + fee",
                        ));
                    }
                    Ok(serde_json::to_string(&args).unwrap())
                }
                IcpNeuronArgs::AddDissolveDelay(args) => {
                    NeuronReferenceStore::get_by_subaccount(args.subaccount)?;
                    NeuronLogic::get_full_neuron(args.subaccount).await?;
                    Ok(serde_json::to_string(&args).unwrap())
                }
                IcpNeuronArgs::SetDissolveState(args) => {
                    NeuronReferenceStore::get_by_subaccount(args.subaccount)?;
                    NeuronLogic::get_full_neuron(args.subaccount).await?;
                    Ok(serde_json::to_string(&args).unwrap())
                }
                IcpNeuronArgs::AutoStake(args) => {
                    NeuronReferenceStore::get_by_subaccount(args.subaccount)?;
                    NeuronLogic::get_full_neuron(args.subaccount).await?;
                    Ok(serde_json::to_string(&args).unwrap())
                }
                IcpNeuronArgs::Spawn(args) => {
                    NeuronReferenceStore::get_by_subaccount(args.parent_subaccount)?;
                    let neuron = NeuronLogic::get_full_neuron(args.parent_subaccount).await?;
                    if neuron.maturity_e8s_equivalent < 100000000 {
                        return Err(ApiError::bad_request(
                            "neuron must have at least 1 ICP in maturity to spawn",
                        ));
                    }
                    Ok(serde_json::to_string(&args).unwrap())
                }
                IcpNeuronArgs::CreateProposal(create_proposal_args) => {
                    NeuronReferenceStore::get_by_subaccount(create_proposal_args.subaccount)?;
                    NeuronLogic::get_full_neuron(create_proposal_args.subaccount).await?;
                    Ok(serde_json::to_string(&create_proposal_args).unwrap())
                }
                IcpNeuronArgs::Vote(vote_args) => {
                    NeuronReferenceStore::get_by_subaccount(vote_args.subaccount)?;
                    NeuronLogic::get_full_neuron(vote_args.subaccount).await?;
                    Ok(serde_json::to_string(&vote_args).unwrap())
                }
                IcpNeuronArgs::Disburse(disburse_args) => {
                    NeuronReferenceStore::get_by_subaccount(disburse_args.subaccount)?;
                    NeuronLogic::get_full_neuron(disburse_args.subaccount).await?;
                    Ok(serde_json::to_string(&disburse_args).unwrap())
                }
                IcpNeuronArgs::SetFollowing(set_following_args) => {
                    NeuronReferenceStore::get_by_subaccount(set_following_args.subaccount)?;
                    NeuronLogic::get_full_neuron(set_following_args.subaccount).await?;
                    Ok(serde_json::to_string(&set_following_args).unwrap())
                }
                IcpNeuronArgs::Command(args) => Ok(serde_json::to_string(&args).unwrap()),
            },
        }
    }
}
