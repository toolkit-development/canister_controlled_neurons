use candid::Principal;
use toolkit_utils::{api_error::ApiError, cell::CellStorage, result::CanisterResult};

use crate::{
    methods::neuron_methods::get_canister_icp_balance,
    storage::{config_storage::config_store, neuron_reference_storage::NeuronReferenceStore},
    types::{
        config::Config,
        modules::{
            IcpNeuronArgs, Module, ModuleResponse, NeuronType, TreasuryManagementModuleType,
        },
    },
};

use super::neuron_logic::NeuronLogic;

pub struct ConfigLogic;

impl ConfigLogic {
    pub fn init(
        governance_canister_id: Principal,
        sns_ledger_canister_id: Principal,
    ) -> CanisterResult<Config> {
        config_store().set(Config::new(governance_canister_id, sns_ledger_canister_id))
    }

    pub fn get_config() -> CanisterResult<Config> {
        config_store().get()
    }

    pub async fn set_module(module: Module) -> CanisterResult<ModuleResponse> {
        match module {
            Module::TreasuryManagement(module) => match module {
                TreasuryManagementModuleType::Neuron(module) => match module {
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
                            let result = NeuronLogic::top_up_neuron_by_subaccount(
                                args.subaccount,
                                args.amount_e8s,
                            )
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
                            let result = NeuronLogic::set_dissolve_state(
                                args.subaccount,
                                args.start_dissolving,
                            )
                            .await?;
                            Ok(ModuleResponse::Boolean(result))
                        }
                        IcpNeuronArgs::AutoStake(args) => {
                            let result =
                                NeuronLogic::auto_stake_maturity(args.subaccount, args.auto_stake)
                                    .await?;
                            Ok(ModuleResponse::Boolean(result))
                        }
                    },
                },
            },
        }
    }
    pub async fn validate_set_module(module: Module) -> CanisterResult<String> {
        match module {
            Module::TreasuryManagement(module) => match module {
                TreasuryManagementModuleType::Neuron(neuron) => match neuron {
                    NeuronType::Icp(args) => match args {
                        IcpNeuronArgs::Create(args) => {
                            let balance = get_canister_icp_balance().await?;
                            if balance < args.amount_e8s {
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
                            let balance = get_canister_icp_balance().await?;
                            if balance < args.amount_e8s {
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
                    },
                },
            },
        }
    }
}
