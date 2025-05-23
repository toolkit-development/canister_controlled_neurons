use candid::encode_args;
use canister_controlled_neuron::{
    api::sns_governance_api::{Action, Motion, Proposal},
    types::{
        args::sns_neuron_args::{CreateSnsNeuronArgs, CreateSnsNeuronProposalArgs, SnsNeuronArgs},
        modules::{ModuleResponse, NeuronType},
        sns_neuron_reference::SnsNeuronReferenceResponse,
    },
};
use test_helper::{
    context::Context,
    declarations::sns_governance_api::{
        Action as SnsAction, Command, MintSnsTokens, Motion as SnsMotion, NeuronId,
        Proposal as SnsProposal, ProposalId,
    },
    sender::Sender,
};
use toolkit_utils::{icrc_ledger_types::icrc1::account::Account, result::CanisterResult};

#[test]
fn test_get_config() -> Result<(), String> {
    let context = Context::new(true);
    let sns_context = context.sns.as_ref().unwrap();

    println!("sns_neurons: {:?}", sns_context.sns_neurons.len());

    let motion_proposal = sns_context.sns_command(
        &context.pic,
        sns_context.developer_neuron_id.clone().unwrap(),
        Command::MakeProposal(SnsProposal {
            url: "https://example.com".to_string(),
            title: "Test motion".to_string(),
            summary: "Test description".to_string(),
            action: Some(SnsAction::Motion(SnsMotion {
                motion_text: "Test motion".to_string(),
            })),
        }),
        Sender::Owner,
    );

    println!("motion_proposal: {:?}", motion_proposal);

    let sns_proposal = context
        .sns
        .unwrap()
        .get_sns_proposal(&context.pic, Some(ProposalId { id: 1 }), Sender::Owner)
        .unwrap();

    println!("sns_proposal: {:?}", sns_proposal);

    Ok(())
}

#[test]
fn test_create_neuron_blanco() -> Result<(), String> {
    let context = Context::new(true);
    let sns = context.sns.as_ref().unwrap();

    // mint tokens for the neuron controller canister
    let mint_sns_tokens = sns.sns_command(
        &context.pic,
        sns.developer_neuron_id.clone().unwrap(),
        Command::MakeProposal(SnsProposal {
            url: "https://example.com".to_string(),
            title: "Test mint".to_string(),
            summary: "Test mint".to_string(),
            action: Some(SnsAction::MintSnsTokens(MintSnsTokens {
                to_principal: Some(context.neuron_controller_canister),
                to_subaccount: None,
                memo: Some(1),
                amount_e8s: Some(10_000_000_000),
            })),
        }),
        Sender::Owner,
    )?;

    println!("mint_sns_tokens: {:?}", mint_sns_tokens);

    // vote for the proposal
    let vote_result = sns.vote_with_neurons(
        &context.pic,
        Some(ProposalId { id: 1 }),
        sns.sns_neurons
            .iter()
            .map(|n| n.id.clone().unwrap())
            .collect(),
        1,
    )?;
    println!("vote_result: {:?}", vote_result);

    let proposal_id =
        sns.get_sns_proposal(&context.pic, Some(ProposalId { id: 1 }), Sender::Owner)?;
    println!("proposal_id: {:?}", proposal_id);

    // get the balance of the neuron controller canister
    let balance = sns.get_balance(
        &context.pic,
        Account {
            owner: context.neuron_controller_canister,
            subaccount: None,
        },
    )?;

    println!("balance: {:?}", balance);
    assert!(balance == 10_000_000_000u64);

    let args: NeuronType = NeuronType::Sns(SnsNeuronArgs::Create(CreateSnsNeuronArgs {
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

    let neuron_references = context.query::<CanisterResult<Vec<SnsNeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_sns_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());

    Ok(())
}

#[test]
fn test_create_neuron_with_dissolve_delay() -> Result<(), String> {
    let context = Context::new(true);
    let sns = context.sns.as_ref().unwrap();

    // mint tokens for the neuron controller canister
    let mint_sns_tokens = sns.sns_command(
        &context.pic,
        sns.developer_neuron_id.clone().unwrap(),
        Command::MakeProposal(SnsProposal {
            url: "https://example.com".to_string(),
            title: "Test mint".to_string(),
            summary: "Test mint".to_string(),
            action: Some(SnsAction::MintSnsTokens(MintSnsTokens {
                to_principal: Some(context.neuron_controller_canister),
                to_subaccount: None,
                memo: Some(1),
                amount_e8s: Some(10_000_000_000),
            })),
        }),
        Sender::Owner,
    )?;

    println!("mint_sns_tokens: {:?}", mint_sns_tokens);

    // vote for the proposal
    let vote_result = sns.vote_with_neurons(
        &context.pic,
        Some(ProposalId { id: 1 }),
        sns.sns_neurons
            .iter()
            .map(|n| n.id.clone().unwrap())
            .collect(),
        1,
    )?;
    println!("vote_result: {:?}", vote_result);

    let proposal_id =
        sns.get_sns_proposal(&context.pic, Some(ProposalId { id: 1 }), Sender::Owner)?;
    println!("proposal_id: {:?}", proposal_id);

    // get the balance of the neuron controller canister
    let balance = sns.get_balance(
        &context.pic,
        Account {
            owner: context.neuron_controller_canister,
            subaccount: None,
        },
    )?;

    println!("balance: {:?}", balance);
    assert!(balance == 10_000_000_000u64);

    let args: NeuronType = NeuronType::Sns(SnsNeuronArgs::Create(CreateSnsNeuronArgs {
        amount_e8s: 1_000_000_000,
        auto_stake: None,
        dissolve_delay_seconds: Some(255_000_000),
    }));

    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
    )?;

    println!("result: {:?}", create_neuron);
    assert!(create_neuron.is_ok());

    let neuron_references = context.query::<CanisterResult<Vec<SnsNeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_sns_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_id = neuron_references.unwrap()[0].neuron_id.clone().unwrap();

    let neuron_info = context
        .sns
        .as_ref()
        .unwrap()
        .get_sns_neuron(&context.pic, NeuronId { id: neuron_id })?;

    println!("neuron_info: {:?}", neuron_info);

    Ok(())
}

#[test]
fn test_create_proposal_with_neuron() -> Result<(), String> {
    let context = Context::new(true);
    let sns = context.sns.as_ref().unwrap();

    // mint tokens for the neuron controller canister
    let mint_sns_tokens = sns.sns_command(
        &context.pic,
        sns.developer_neuron_id.clone().unwrap(),
        Command::MakeProposal(SnsProposal {
            url: "https://example.com".to_string(),
            title: "Test mint".to_string(),
            summary: "Test mint".to_string(),
            action: Some(SnsAction::MintSnsTokens(MintSnsTokens {
                to_principal: Some(context.neuron_controller_canister),
                to_subaccount: None,
                memo: Some(1),
                amount_e8s: Some(10_000_000_000),
            })),
        }),
        Sender::Owner,
    )?;

    println!("mint_sns_tokens: {:?}", mint_sns_tokens);

    // vote for the proposal
    let vote_result = sns.vote_with_neurons(
        &context.pic,
        Some(ProposalId { id: 1 }),
        sns.sns_neurons
            .iter()
            .map(|n| n.id.clone().unwrap())
            .collect(),
        1,
    )?;
    println!("vote_result: {:?}", vote_result);

    let proposal_id =
        sns.get_sns_proposal(&context.pic, Some(ProposalId { id: 1 }), Sender::Owner)?;
    println!("proposal_id: {:?}", proposal_id);

    // get the balance of the neuron controller canister
    let balance = sns.get_balance(
        &context.pic,
        Account {
            owner: context.neuron_controller_canister,
            subaccount: None,
        },
    )?;

    println!("balance: {:?}", balance);
    assert!(balance == 10_000_000_000u64);

    let args: NeuronType = NeuronType::Sns(SnsNeuronArgs::Create(CreateSnsNeuronArgs {
        amount_e8s: 1_000_000_000,
        auto_stake: None,
        dissolve_delay_seconds: Some(255_000_000),
    }));

    let create_neuron = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
    )?;

    println!("result: {:?}", create_neuron);
    assert!(create_neuron.is_ok());

    let neuron_references = context.query::<CanisterResult<Vec<SnsNeuronReferenceResponse>>>(
        Sender::Other(context.config.governance_canister_id),
        "get_sns_neuron_references",
        None,
    )?;

    println!("neuron_reference: {:?}", &neuron_references);
    assert!(neuron_references.is_ok());
    let neuron_id = neuron_references.unwrap()[0].neuron_id.clone().unwrap();

    let neuron_info = context.sns.as_ref().unwrap().get_sns_neuron(
        &context.pic,
        NeuronId {
            id: neuron_id.clone(),
        },
    )?;

    println!("neuron_info: {:?}", neuron_info);

    let args: NeuronType =
        NeuronType::Sns(SnsNeuronArgs::CreateProposal(CreateSnsNeuronProposalArgs {
            neuron_id,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal".to_string(),
                summary: "Test proposal".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal".to_string(),
                })),
            },
        }));

    let create_proposal = context.update::<CanisterResult<ModuleResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "tk_service_manage_neuron",
        Some(encode_args((args,)).unwrap()),
    )?;

    println!("create_proposal: {:?}", create_proposal);

    let proposal_id = match create_proposal {
        Ok(ModuleResponse::GetProposalResponse(response)) => response.proposal_id,
        _ => return Err("Failed to create proposal".to_string()),
    };

    println!("proposal_id: {:?}", proposal_id);

    let proposal = context.sns.as_ref().unwrap().get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_id.unwrap().id,
        }),
        Sender::Owner,
    )?;

    println!("proposal: {:?}", proposal);

    Ok(())
}
