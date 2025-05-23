use std::time::Duration;

use candid::encode_args;
use canister_controlled_neuron::{
    api::icp_governance_api::{MakeProposalRequest, Motion, Neuron, ProposalActionRequest},
    types::{
        args::icp_neuron_args::{
            CreateIcpNeuronArgs, CreateIcpNeuronProposalArgs, DisburseIcpNeuronArgs, IcpNeuronArgs,
            IcpNeuronIdentifier, SpawnIcpNeuronArgs,
        },
        config::Config,
        icp_neuron_reference::IcpNeuronReferenceResponse,
        modules::{ModuleResponse, NeuronType},
    },
};
use test_helper::{context::Context, sender::Sender};
use toolkit_utils::{icrc_ledger_types::icrc1::account::Account, result::CanisterResult};

#[test]
fn test_get_config() -> Result<(), String> {
    let context = Context::default();
    // fetch the config
    let config_result =
        context.query::<CanisterResult<Config>>(Sender::Owner, "get_config", None)?;

    println!("config_result: {:?}", config_result);
    assert!(config_result.is_ok());

    Ok(())
}

#[test]
fn test_create_neuron_blanco() -> Result<(), String> {
    let context = Context::default();

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

    let args: NeuronType = NeuronType::Icp(IcpNeuronArgs::Create(CreateIcpNeuronArgs {
        amount_e8s: 1_000_000_000,
        auto_stake: None,
        dissolve_delay_seconds: None,
    }));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
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

    let neuron_references = context.query::<CanisterResult<Vec<IcpNeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();

    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = IcpNeuronIdentifier::Subaccount(neuron_references_unwrapped[0].subaccount);

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
    let context = Context::default();

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

    let args: NeuronType = NeuronType::Icp(IcpNeuronArgs::Create(CreateIcpNeuronArgs {
        amount_e8s: 1_000_000_000,
        auto_stake: None,
        dissolve_delay_seconds: Some(255_000_000), // more then 8 years in seconds
    }));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
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

    let neuron_references = context.query::<CanisterResult<Vec<IcpNeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();

    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = IcpNeuronIdentifier::Subaccount(neuron_references_unwrapped[0].subaccount);

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
    let context = Context::default();

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

    let args: NeuronType = NeuronType::Icp(IcpNeuronArgs::Create(CreateIcpNeuronArgs {
        amount_e8s: 1_000_000_000,
        auto_stake: Some(true),
        dissolve_delay_seconds: None,
    }));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
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

    let neuron_references = context.query::<CanisterResult<Vec<IcpNeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();

    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = IcpNeuronIdentifier::Subaccount(neuron_references_unwrapped[0].subaccount);

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
    let context = Context::default();
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

    let args: NeuronType = NeuronType::Icp(IcpNeuronArgs::Create(CreateIcpNeuronArgs {
        amount_e8s: 10_000_000_000u64,
        auto_stake: Some(false),
        dissolve_delay_seconds: Some(255_000_000),
    }));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
    )?;
    assert!(create_neuron.is_ok());

    let neuron_references = context.query::<CanisterResult<Vec<IcpNeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();
    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = IcpNeuronIdentifier::Subaccount(neuron_references_unwrapped[0].subaccount);

    let args: NeuronType =
        NeuronType::Icp(IcpNeuronArgs::CreateProposal(CreateIcpNeuronProposalArgs {
            identifier: subaccount.clone(),
            proposal: MakeProposalRequest {
                title: Some("Test proposal".to_string()),
                summary: "Simulate governance vote".to_string(),
                action: Some(ProposalActionRequest::Motion(Motion {
                    motion_text: "Approve something".to_string(),
                })),
                url: "".to_string(),
            },
        }));
    let _ = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
    )?;

    let proposal = context.get_icp_proposal(2, Sender::Owner);
    assert!(proposal.is_ok());

    context.pic.advance_time(Duration::from_secs(400000));
    context.pic.tick();

    let proposal = context.get_icp_proposal(2, Sender::Owner);
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
fn test_spawn_neuron_manual_disburse() -> Result<(), String> {
    let context = Context::default();

    let config_result =
        context.query::<CanisterResult<Config>>(Sender::Owner, "get_config", None)?;
    assert!(config_result.is_ok());
    let config = config_result.unwrap();

    let governance_canister_balance = context.get_icp_balance(config.governance_canister_id);
    println!(
        "governance_canister_balance: {:?}",
        governance_canister_balance
    );
    assert!(governance_canister_balance.is_ok());
    assert!(governance_canister_balance.unwrap() == 0u64);

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

    let args: NeuronType = NeuronType::Icp(IcpNeuronArgs::Create(CreateIcpNeuronArgs {
        amount_e8s: 10_000_000_000u64,
        auto_stake: Some(false),
        dissolve_delay_seconds: Some(255_000_000),
    }));
    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
    )?;
    assert!(create_neuron.is_ok());

    let balance = context.get_icp_balance(context.neuron_controller_canister);
    println!("balance: {:?}", balance);
    assert!(balance.is_ok());
    assert!(balance.unwrap() == 0u64);

    let neuron_references = context.query::<CanisterResult<Vec<IcpNeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_neuron_references",
        None,
    )?;
    assert!(neuron_references.is_ok());
    let neuron_references_unwrapped = neuron_references.unwrap();
    assert!(!neuron_references_unwrapped.is_empty());
    let subaccount = IcpNeuronIdentifier::Subaccount(neuron_references_unwrapped[0].subaccount);

    let args: NeuronType =
        NeuronType::Icp(IcpNeuronArgs::CreateProposal(CreateIcpNeuronProposalArgs {
            identifier: subaccount.clone(),
            proposal: MakeProposalRequest {
                title: Some("Test proposal".to_string()),
                summary: "Simulate governance vote".to_string(),
                action: Some(ProposalActionRequest::Motion(Motion {
                    motion_text: "Approve something".to_string(),
                })),
                url: "".to_string(),
            },
        }));
    let _ = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
    )?;

    let proposal = context.get_icp_proposal(2, Sender::Owner);
    assert!(proposal.is_ok());

    context.pic.advance_time(Duration::from_secs(400000));
    context.pic.tick();

    let proposal = context.get_icp_proposal(2, Sender::Owner);
    assert!(proposal.is_ok());

    context.pic.advance_time(Duration::from_secs(500000));
    context.pic.tick();
    context.pic.advance_time(Duration::from_secs(500000));
    context.pic.tick();

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(encode_args((subaccount.clone(),)).unwrap()),
    )?;
    println!("neuron_info: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    let neuron_info_unwrapped = neuron_info.unwrap();
    assert!(neuron_info_unwrapped.maturity_e8s_equivalent > 0);

    let args: NeuronType = NeuronType::Icp(IcpNeuronArgs::Spawn(SpawnIcpNeuronArgs {
        identifier: subaccount.clone(),
        start_dissolving: true,
    }));
    let x = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
    )?;
    println!("x: {:?}", x);

    let neuron_references = context.query::<CanisterResult<Vec<IcpNeuronReferenceResponse>>>(
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
        Some(
            encode_args((IcpNeuronIdentifier::Subaccount(
                neuron_references_unwrapped[1].subaccount,
            ),))
            .unwrap(),
        ),
    )?;

    println!("neuron_info: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    assert!(neuron_info.unwrap().maturity_e8s_equivalent > 0);

    context.pic.advance_time(Duration::from_secs(605000));
    context.pic.tick();

    let neuron_info = context.update::<CanisterResult<Neuron>>(
        Sender::Other(context.config.governance_canister_id),
        "get_full_neuron",
        Some(
            encode_args((IcpNeuronIdentifier::Subaccount(
                neuron_references_unwrapped[1].subaccount,
            ),))
            .unwrap(),
        ),
    )?;

    println!("no maturity neuron: {:?}", neuron_info);
    assert!(neuron_info.is_ok());
    let neuron_info_unwrapped = neuron_info.unwrap();
    assert!(neuron_info_unwrapped.maturity_e8s_equivalent == 0);

    let disburse_args: NeuronType =
        NeuronType::Icp(IcpNeuronArgs::Disburse(DisburseIcpNeuronArgs {
            identifier: IcpNeuronIdentifier::Subaccount(neuron_references_unwrapped[1].subaccount),
        }));

    let disburse_result = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((disburse_args,)).unwrap()),
    )?;
    println!("disburse_result: {:?}", disburse_result);
    assert!(disburse_result.is_ok());

    context.pic.tick();
    context.pic.advance_time(Duration::from_secs(605000));

    let config_result =
        context.query::<CanisterResult<Config>>(Sender::Owner, "get_config", None)?;
    assert!(config_result.is_ok());
    let config = config_result.unwrap();

    let governance_canister_balance = context.get_icp_balance(config.governance_canister_id);
    println!(
        "governance_canister_balance after: {:?}",
        governance_canister_balance
    );
    println!(
        "neuron_info_unwrapped.cached_neuron_stake_e8s: {:?}",
        neuron_info_unwrapped.cached_neuron_stake_e8s
    );
    assert!(governance_canister_balance.is_ok());
    assert!(
        governance_canister_balance.unwrap()
            == (neuron_info_unwrapped.cached_neuron_stake_e8s - 10_000) // fee
    );
    Ok(())
}
