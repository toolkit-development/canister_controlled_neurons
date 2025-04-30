use candid::Principal;
use toolkit_utils::{cell::CellStorage, result::CanisterResult};

use crate::{
    storage::config_storage::config_store,
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
                                args.maturity_disbursement_target,
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
                            Ok(ModuleResponse::BlockHeight(result))
                        }
                        IcpNeuronArgs::Command(args) => {
                            let result =
                                NeuronLogic::command_neuron(args.subaccount, args.command).await?;
                            Ok(ModuleResponse::ManageNeuronResponse(Box::new(result)))
                        }
                        IcpNeuronArgs::DisburseMaturity(args) => {
                            let result =
                                NeuronLogic::set_maturity_disbursements(args.subaccount, args.args)
                                    .await?;
                            Ok(ModuleResponse::Neuron(Box::new(result)))
                        }
                    },
                },
            },
        }
    }
}
