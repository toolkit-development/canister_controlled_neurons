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
    storage::{icp_neuron_reference_storage::IcpNeuronReferenceStore, log_storage::LogStore},
    types::{
        args::icp_neuron_args::{IcpNeuronArgs, IcpNeuronIdentifier, IcpNeuronVote},
        icp_neuron_reference::{IcpNeuronReference, IcpNeuronReferenceResponse},
        modules::ModuleResponse,
        topic::Topic,
    },
};

pub struct ICPNeuronLogic;

impl ICPNeuronLogic {
    pub fn remove_neuron(id: u64) -> CanisterResult<()> {
        IcpNeuronReferenceStore::remove(id);
        Ok(())
    }

    pub fn get_neurons() -> CanisterResult<Vec<IcpNeuronReferenceResponse>> {
        let neurons = IcpNeuronReferenceStore::get_all();
        Ok(neurons
            .into_iter()
            .map(|(id, neuron)| neuron.to_response(id))
            .collect())
    }

    pub async fn create_neuron(
        amount_e8s: u64,
        auto_stake: Option<bool>,
        dissolve_delay: Option<u64>,
    ) -> CanisterResult<IcpNeuronReferenceResponse> {
        let neuron = IcpNeuronReference::new(amount_e8s).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error creating neuron: {}", time(), e));
            e
        })?;
        let (id, mut neuron) = IcpNeuronReferenceStore::insert(neuron.clone())?;

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

        let response = IcpNeuronReferenceStore::update(id, neuron.clone())?;

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

    pub async fn top_up_neuron(
        identifier: &IcpNeuronIdentifier,
        amount_e8s: u64,
    ) -> CanisterResult<bool> {
        let (_, mut neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;

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
        identifier: &IcpNeuronIdentifier,
        command: ManageNeuronCommandRequest,
    ) -> CanisterResult<ManageNeuronResponse> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        let neuron = neuron.command(command).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error commanding neuron: {}", time(), e));
            e
        })?;
        Ok(neuron)
    }

    pub async fn add_dissolve_delay(
        identifier: &IcpNeuronIdentifier,
        dissolve_delay: u64,
    ) -> CanisterResult<bool> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        neuron.increase_dissolve_delay(dissolve_delay).await?;
        let _ = LogStore::insert(format!("Dissolve delay set to {} seconds", dissolve_delay));
        Ok(true)
    }

    pub async fn set_dissolve_state(
        identifier: &IcpNeuronIdentifier,
        start_dissolving: bool,
    ) -> CanisterResult<bool> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        neuron.set_dissolve_state(start_dissolving).await?;
        Ok(true)
    }

    pub async fn auto_stake_maturity(
        identifier: &IcpNeuronIdentifier,
        auto_stake: bool,
    ) -> CanisterResult<bool> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        neuron.auto_stake_maturity(auto_stake).await?;
        let _ = LogStore::insert(format!("Auto stake maturity set to {}", auto_stake));
        Ok(true)
    }

    pub async fn spawn_neuron(
        identifier: &IcpNeuronIdentifier,
        start_dissolving: bool,
    ) -> CanisterResult<bool> {
        let (_, parent_neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        let new_nonce = IcpNeuronReferenceStore::get_latest_key() + 1;
        let new_neuron_id = parent_neuron
            .spawn(new_nonce)
            .await?
            .created_neuron_id
            .map(|id| id.id);

        let subaccount = generate_subaccount_by_nonce(new_nonce);
        let spawned_neuron = IcpNeuronReference {
            blockheight: 0,
            subaccount,
            nonce: new_nonce,
            neuron_id: new_neuron_id,
            parent_subaccount: Some(parent_neuron.subaccount),
        };

        let _ = IcpNeuronReferenceStore::insert(spawned_neuron.clone())?;

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
        identifier: &IcpNeuronIdentifier,
        proposal: MakeProposalRequest,
    ) -> CanisterResult<MakeProposalResponse> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        let result = neuron.create_proposal(proposal).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error creating proposal: {}", time(), e));
            e
        })?;
        Ok(result)
    }

    pub async fn vote(
        identifier: &IcpNeuronIdentifier,
        proposal_id: u64,
        vote: IcpNeuronVote,
    ) -> CanisterResult<bool> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        let result = neuron.vote(proposal_id, vote).await.map_err(|e| {
            let _ = LogStore::insert(format!("{}: Error voting: {}", time(), e));
            e
        })?;

        Ok(result)
    }

    pub async fn disburse(identifier: &IcpNeuronIdentifier) -> CanisterResult<DisburseResponse> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        neuron.disburse().await
    }

    pub async fn set_visibility(
        identifier: &IcpNeuronIdentifier,
        visibility: i32,
    ) -> CanisterResult<()> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
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
        identifier: &IcpNeuronIdentifier,
        topic: Topic,
        following_neurons: Vec<u64>,
    ) -> CanisterResult<()> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
        neuron.set_following(topic, following_neurons).await
    }

    pub async fn get_full_neuron(identifier: &IcpNeuronIdentifier) -> CanisterResult<GovNeuron> {
        let (_, neuron) = IcpNeuronReferenceStore::get_by_identifier(identifier)?;
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

    pub async fn handle_icp_neuron_args(args: IcpNeuronArgs) -> CanisterResult<ModuleResponse> {
        match args {
            IcpNeuronArgs::Create(args) => {
                let result = ICPNeuronLogic::create_neuron(
                    args.amount_e8s,
                    args.auto_stake,
                    args.dissolve_delay_seconds,
                )
                .await?;
                Ok(ModuleResponse::IcpNeuron(Box::new(result)))
            }
            IcpNeuronArgs::TopUp(args) => {
                let result =
                    ICPNeuronLogic::top_up_neuron(&args.identifier, args.amount_e8s).await?;
                Ok(ModuleResponse::Boolean(result))
            }
            IcpNeuronArgs::AddDissolveDelay(args) => {
                let result = ICPNeuronLogic::add_dissolve_delay(
                    &args.identifier,
                    args.dissolve_delay_seconds,
                )
                .await?;
                Ok(ModuleResponse::Boolean(result))
            }
            IcpNeuronArgs::SetDissolveState(args) => {
                let result =
                    ICPNeuronLogic::set_dissolve_state(&args.identifier, args.start_dissolving)
                        .await?;
                Ok(ModuleResponse::Boolean(result))
            }
            IcpNeuronArgs::AutoStake(args) => {
                let result =
                    ICPNeuronLogic::auto_stake_maturity(&args.identifier, args.auto_stake).await?;
                Ok(ModuleResponse::Boolean(result))
            }
            IcpNeuronArgs::Spawn(args) => {
                let result =
                    ICPNeuronLogic::spawn_neuron(&args.identifier, args.start_dissolving).await?;
                Ok(ModuleResponse::Boolean(result))
            }
            IcpNeuronArgs::CreateProposal(args) => {
                let result =
                    ICPNeuronLogic::create_proposal(&args.identifier, args.proposal).await?;
                Ok(ModuleResponse::MakeProposalResponse(Box::new(result)))
            }
            IcpNeuronArgs::Vote(args) => {
                let result =
                    ICPNeuronLogic::vote(&args.identifier, args.proposal_id, args.vote).await?;
                Ok(ModuleResponse::Boolean(result))
            }
            IcpNeuronArgs::Disburse(args) => {
                let _ = ICPNeuronLogic::disburse(&args.identifier).await?;
                Ok(ModuleResponse::Boolean(true))
            }
            IcpNeuronArgs::SetFollowing(set_following_args) => {
                for arg in set_following_args.following {
                    ICPNeuronLogic::set_following(
                        &set_following_args.identifier,
                        arg.topic,
                        arg.followees,
                    )
                    .await?;
                }
                Ok(ModuleResponse::Boolean(true))
            }
        }
    }

    pub async fn validate_icp_neuron_args(args: IcpNeuronArgs) -> CanisterResult<String> {
        match args {
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
                IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
                ICPNeuronLogic::get_full_neuron(&args.identifier).await?;
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
                IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
                ICPNeuronLogic::get_full_neuron(&args.identifier).await?;
                Ok(serde_json::to_string(&args).unwrap())
            }
            IcpNeuronArgs::SetDissolveState(args) => {
                IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
                ICPNeuronLogic::get_full_neuron(&args.identifier).await?;
                Ok(serde_json::to_string(&args).unwrap())
            }
            IcpNeuronArgs::AutoStake(args) => {
                IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
                ICPNeuronLogic::get_full_neuron(&args.identifier).await?;
                Ok(serde_json::to_string(&args).unwrap())
            }
            IcpNeuronArgs::Spawn(args) => {
                IcpNeuronReferenceStore::get_by_identifier(&args.identifier)?;
                let neuron = ICPNeuronLogic::get_full_neuron(&args.identifier).await?;
                if neuron.maturity_e8s_equivalent < 100000000 {
                    return Err(ApiError::bad_request(
                        "neuron must have at least 1 ICP in maturity to spawn",
                    ));
                }
                Ok(serde_json::to_string(&args).unwrap())
            }
            IcpNeuronArgs::CreateProposal(create_proposal_args) => {
                IcpNeuronReferenceStore::get_by_identifier(&create_proposal_args.identifier)?;
                ICPNeuronLogic::get_full_neuron(&create_proposal_args.identifier).await?;
                Ok(serde_json::to_string(&create_proposal_args).unwrap())
            }
            IcpNeuronArgs::Vote(vote_args) => {
                IcpNeuronReferenceStore::get_by_identifier(&vote_args.identifier)?;
                ICPNeuronLogic::get_full_neuron(&vote_args.identifier).await?;
                Ok(serde_json::to_string(&vote_args).unwrap())
            }
            IcpNeuronArgs::Disburse(disburse_args) => {
                IcpNeuronReferenceStore::get_by_identifier(&disburse_args.identifier)?;
                ICPNeuronLogic::get_full_neuron(&disburse_args.identifier).await?;
                Ok(serde_json::to_string(&disburse_args).unwrap())
            }
            IcpNeuronArgs::SetFollowing(set_following_args) => {
                IcpNeuronReferenceStore::get_by_identifier(&set_following_args.identifier)?;
                ICPNeuronLogic::get_full_neuron(&set_following_args.identifier).await?;
                Ok(serde_json::to_string(&set_following_args).unwrap())
            }
        }
    }
}
