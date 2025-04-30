use candid::encode_args;
use canister_controlled_neuron::{
    api::icp_governance_api::Neuron,
    types::{
        config::Config,
        modules::{
            CreateNeuronArgs, IcpNeuronArgs, Module, ModuleResponse, NeuronType,
            TreasuryManagementModuleType,
        },
        neuron_reference::NeuronReferenceResponse,
    },
};
use test_helper::{context::Context, sender::Sender};
use toolkit_utils::{icrc_ledger_types::icrc1::account::Account, result::CanisterResult};

#[test]
fn test_get_config() -> Result<(), String> {
    let context = Context::new();
    // fetch the config
    let config_result =
        context.query::<CanisterResult<Config>>(Sender::Owner, "get_config", None)?;

    println!("config_result: {:?}", config_result);
    assert!(config_result.is_ok());

    Ok(())
}

#[test]
fn test_create_neuron_blanco() -> Result<(), String> {
    let context = Context::new();

    context.transfer_icp(
        10_000_000_000,
        Account {
            owner: context.owner_account.owner,
            subaccount: None,
        },
        Account {
            owner: context.neuron_controller_canister,
            subaccount: None,
        },
    );

    let balance = context.get_icp_balance(context.neuron_controller_canister);
    println!("balance: {:?}", balance);
    assert!(balance.is_ok());
    assert!(balance.unwrap() == 10_000_000_000u64);

    let module_args: Module = Module::TreasuryManagement(TreasuryManagementModuleType::Neuron(
        NeuronType::Icp(IcpNeuronArgs::Create(CreateNeuronArgs {
            amount_e8s: 1_000_000_000,
            auto_stake: None,
            dissolve_delay_seconds: None,
        })),
    ));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "set_module",
        Some(encode_args((module_args,)).unwrap()),
    )?;

    println!("result: {:?}", create_neuron);
    assert!(create_neuron.is_ok());

    let logs = context.query::<Vec<String>>(
        Sender::Other(context.config.governance_canister_id),
        "get_logs",
        None,
    )?;
    println!("logs: {:?}", logs);
    assert!(logs.is_empty());

    let neuron_references = context.query::<CanisterResult<Vec<NeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();

    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = &neuron_references_unwrapped[0].subaccount.clone();

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(encode_args((subaccount,)).unwrap()),
    )?;
    println!("neuron_info: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    Ok(())
}

#[test]
fn test_create_neuron_with_dissolve_delay() -> Result<(), String> {
    let context = Context::new();

    context.transfer_icp(
        10_000_000_000,
        Account {
            owner: context.owner_account.owner,
            subaccount: None,
        },
        Account {
            owner: context.neuron_controller_canister,
            subaccount: None,
        },
    );

    let balance = context.get_icp_balance(context.neuron_controller_canister);
    println!("balance: {:?}", balance);
    assert!(balance.is_ok());
    assert!(balance.unwrap() == 10_000_000_000u64);

    let module_args: Module = Module::TreasuryManagement(TreasuryManagementModuleType::Neuron(
        NeuronType::Icp(IcpNeuronArgs::Create(CreateNeuronArgs {
            amount_e8s: 1_000_000_000,
            auto_stake: None,
            dissolve_delay_seconds: Some(255_000_000), // more then 8 years in seconds
        })),
    ));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "set_module",
        Some(encode_args((module_args,)).unwrap()),
    )?;

    println!("result: {:?}", create_neuron);
    assert!(create_neuron.is_ok());

    let logs = context.query::<Vec<String>>(
        Sender::Other(context.config.governance_canister_id),
        "get_logs",
        None,
    )?;
    println!("logs: {:?}", logs);
    assert!(logs.is_empty());

    let neuron_references = context.query::<CanisterResult<Vec<NeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();

    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = &neuron_references_unwrapped[0].subaccount.clone();

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(encode_args((subaccount,)).unwrap()),
    )?;
    println!("neuron_info: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    Ok(())
}

#[test]
fn test_create_neuron_with_auto_stake() -> Result<(), String> {
    let context = Context::new();

    context.transfer_icp(
        10_000_000_000,
        Account {
            owner: context.owner_account.owner,
            subaccount: None,
        },
        Account {
            owner: context.neuron_controller_canister,
            subaccount: None,
        },
    );

    let balance = context.get_icp_balance(context.neuron_controller_canister);
    println!("balance: {:?}", balance);
    assert!(balance.is_ok());
    assert!(balance.unwrap() == 10_000_000_000u64);

    let module_args: Module = Module::TreasuryManagement(TreasuryManagementModuleType::Neuron(
        NeuronType::Icp(IcpNeuronArgs::Create(CreateNeuronArgs {
            amount_e8s: 1_000_000_000,
            auto_stake: Some(true),
            dissolve_delay_seconds: None,
        })),
    ));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "set_module",
        Some(encode_args((module_args,)).unwrap()),
    )?;

    println!("result: {:?}", create_neuron);
    assert!(create_neuron.is_ok());

    let logs = context.query::<Vec<String>>(
        Sender::Other(context.config.governance_canister_id),
        "get_logs",
        None,
    )?;
    println!("logs: {:?}", logs);
    assert!(logs.is_empty());

    let neuron_references = context.query::<CanisterResult<Vec<NeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();

    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = &neuron_references_unwrapped[0].subaccount.clone();

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(encode_args((subaccount,)).unwrap()),
    )?;
    println!("neuron_info: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    Ok(())
}
