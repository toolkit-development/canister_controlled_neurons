use std::time::Duration;

use candid::encode_args;
use canister_controlled_neuron::{
    api::icp_governance_api::{MakeProposalRequest, Motion, Neuron, ProposalActionRequest},
    types::{
        config::Config,
        modules::{
            CreateNeuronArgs, CreateProposalArgs, IcpNeuronArgs, Module, ModuleResponse,
            NeuronType, SpawnArgs, TreasuryManagementModuleType,
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

#[test]
fn test_create_proposal() -> Result<(), String> {
    let context = Context::new();
    context.transfer_icp(
        10_000_010_000,
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
    assert!(balance.is_ok());
    assert!(balance.unwrap() == 10_000_010_000u64);

    let module_args: Module = Module::TreasuryManagement(TreasuryManagementModuleType::Neuron(
        NeuronType::Icp(IcpNeuronArgs::Create(CreateNeuronArgs {
            amount_e8s: 10_000_000_000u64,
            auto_stake: Some(false),
            dissolve_delay_seconds: Some(255_000_000),
        })),
    ));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "set_module",
        Some(encode_args((module_args,)).unwrap()),
    )?;
    assert!(create_neuron.is_ok());

    let neuron_references = context.query::<CanisterResult<Vec<NeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();
    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = &neuron_references_unwrapped[0].subaccount.clone();

    let module_args: Module = Module::TreasuryManagement(TreasuryManagementModuleType::Neuron(
        NeuronType::Icp(IcpNeuronArgs::CreateProposal(CreateProposalArgs {
            subaccount: *subaccount,
            proposal: MakeProposalRequest {
                title: Some("Test proposal".to_string()),
                summary: "Simulate governance vote".to_string(),
                action: Some(ProposalActionRequest::Motion(Motion {
                    motion_text: "Approve something".to_string(),
                })),
                url: "".to_string(),
            },
        })),
    ));
    let _ = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "set_module",
        Some(encode_args((module_args,)).unwrap()),
    )?;

    let proposal = context.get_proposal(2, Sender::Owner);
    assert!(proposal.is_ok());

    context.pic.advance_time(Duration::from_secs(400000));
    context.pic.tick();

    let proposal = context.get_proposal(2, Sender::Owner);
    assert!(proposal.is_ok());

    context.pic.advance_time(Duration::from_secs(500000));
    context.pic.tick();
    context.pic.advance_time(Duration::from_secs(500000));
    context.pic.tick();

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(encode_args((subaccount,)).unwrap()),
    )?;
    println!("neuron_info: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    let neuron_info_unwrapped = neuron_info.unwrap();
    assert!(neuron_info_unwrapped.maturity_e8s_equivalent > 0);
    Ok(())
}

#[test]
fn test_spawn_neuron() -> Result<(), String> {
    let context = Context::new();
    context.transfer_icp(
        10_000_010_000,
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
    assert!(balance.is_ok());
    assert!(balance.unwrap() == 10_000_010_000u64);

    let module_args: Module = Module::TreasuryManagement(TreasuryManagementModuleType::Neuron(
        NeuronType::Icp(IcpNeuronArgs::Create(CreateNeuronArgs {
            amount_e8s: 10_000_000_000u64,
            auto_stake: Some(false),
            dissolve_delay_seconds: Some(255_000_000),
        })),
    ));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "set_module",
        Some(encode_args((module_args,)).unwrap()),
    )?;
    assert!(create_neuron.is_ok());

    let neuron_references = context.query::<CanisterResult<Vec<NeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();
    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = &neuron_references_unwrapped[0].subaccount.clone();

    let module_args: Module = Module::TreasuryManagement(TreasuryManagementModuleType::Neuron(
        NeuronType::Icp(IcpNeuronArgs::CreateProposal(CreateProposalArgs {
            subaccount: *subaccount,
            proposal: MakeProposalRequest {
                title: Some("Test proposal".to_string()),
                summary: "Simulate governance vote".to_string(),
                action: Some(ProposalActionRequest::Motion(Motion {
                    motion_text: "Approve something".to_string(),
                })),
                url: "".to_string(),
            },
        })),
    ));
    let _ = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "set_module",
        Some(encode_args((module_args,)).unwrap()),
    )?;

    let proposal = context.get_proposal(2, Sender::Owner);
    assert!(proposal.is_ok());

    context.pic.advance_time(Duration::from_secs(400000));
    context.pic.tick();

    let proposal = context.get_proposal(2, Sender::Owner);
    assert!(proposal.is_ok());

    context.pic.advance_time(Duration::from_secs(500000));
    context.pic.tick();
    context.pic.advance_time(Duration::from_secs(500000));
    context.pic.tick();

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(encode_args((subaccount,)).unwrap()),
    )?;
    println!("neuron_info: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    let neuron_info_unwrapped = neuron_info.unwrap();
    assert!(neuron_info_unwrapped.maturity_e8s_equivalent > 0);

    let module_args: Module = Module::TreasuryManagement(TreasuryManagementModuleType::Neuron(
        NeuronType::Icp(IcpNeuronArgs::Spawn(SpawnArgs {
            parent_subaccount: *subaccount,
            start_dissolving: true,
        })),
    ));
    let _ = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "set_module",
        Some(encode_args((module_args,)).unwrap()),
    )?;

    let neuron_references = context.query::<CanisterResult<Vec<NeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;
    println!("neuron_references: {:?}", neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();
    assert!(neuron_references_unwrapped.len() == 2);

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(encode_args((&neuron_references_unwrapped[1].subaccount,)).unwrap()),
    )?;

    println!("neuron_info: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    assert!(neuron_info.unwrap().maturity_e8s_equivalent > 0);

    context.pic.advance_time(Duration::from_secs(605000));
    context.pic.tick();

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(encode_args((&neuron_references_unwrapped[1].subaccount,)).unwrap()),
    )?;

    println!("no maturity neuron: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    assert!(neuron_info.unwrap().maturity_e8s_equivalent == 0);
    Ok(())
}
