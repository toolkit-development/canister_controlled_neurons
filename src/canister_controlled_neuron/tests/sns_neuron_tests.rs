use std::time::Duration;

use candid::encode_args;
use canister_controlled_neuron::{
    api::sns_governance_api::{Action, MintSnsTokens, Motion, Proposal},
    types::{
        args::sns_neuron_args::{CreateSnsNeuronArgs, CreateSnsNeuronProposalArgs, SnsNeuronArgs},
        modules::{ModuleResponse, NeuronType},
        sns_chain_proposals::{PostSnsChainProposal, SnsChainProposalsResponse},
        sns_neuron_reference::SnsNeuronReferenceResponse,
    },
};
use test_helper::{
    context::Context,
    declarations::sns_governance_api::{
        Action as SnsAction, Command, MintSnsTokens as MintSnsTokensTest, Motion as SnsMotion,
        NeuronId as SnsNeuronId, Proposal as SnsProposal, ProposalId, Result1,
    },
    sender::Sender,
};
use toolkit_utils::{icrc_ledger_types::icrc1::account::Account, result::CanisterResult};

#[test]
fn test_get_config() -> Result<(), String> {
    let context = Context::new(true);
    let sns_context = context.sns.as_ref().unwrap();

    println!("sns_neurons: {:?}", sns_context.developer_sns_neurons.len());

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
            action: Some(SnsAction::MintSnsTokens(MintSnsTokensTest {
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
        sns.developer_sns_neurons
            .iter()
            .map(|n| n.id.clone().unwrap())
            .collect(),
        1,
        false,
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
            action: Some(SnsAction::MintSnsTokens(MintSnsTokensTest {
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
        sns.developer_sns_neurons
            .iter()
            .map(|n| n.id.clone().unwrap())
            .collect(),
        1,
        false,
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
        .get_sns_neuron(&context.pic, SnsNeuronId { id: neuron_id })?;

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
            action: Some(SnsAction::MintSnsTokens(MintSnsTokensTest {
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
        sns.developer_sns_neurons
            .iter()
            .map(|n| n.id.clone().unwrap())
            .collect(),
        1,
        false,
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
        SnsNeuronId {
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

#[test]
fn test_create_chain_proposals_with_majority() -> Result<(), String> {
    let context = Context::new(true);
    let sns = context.sns.as_ref().unwrap();
    let service_canister_neuron_id = sns.prepare_neurons(&context)?.id;

    // these proposals will execute directly do to a majority of voting power
    let proposals = vec![
        PostSnsChainProposal {
            index: 0,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 1".to_string(),
                summary: "Test proposal 1".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 1".to_string(),
                })),
            },
        },
        PostSnsChainProposal {
            index: 1,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 2".to_string(),
                summary: "Test proposal 2".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 2".to_string(),
                })),
            },
        },
        PostSnsChainProposal {
            index: 2,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 3".to_string(),
                summary: "Test proposal 3".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 3".to_string(),
                })),
            },
        },
    ];

    let create_chain_proposals = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "create_chain_proposals",
        Some(encode_args((service_canister_neuron_id, proposals, true)).unwrap()),
    )?;

    let binding = create_chain_proposals.clone().unwrap();
    let proposal_1 = binding.proposals.iter().find(|p| p.index == 0).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_1.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    println!("--------------------------------");
    println!("--------------------------------");
    println!("proposal_1: {:?}", proposal_1.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");
    assert!(create_chain_proposals.is_ok());
    assert!(proposal_1.proposal_id.is_some());

    let proposal_1_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_1.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_1_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let second_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    let binding = second_proposal.clone().unwrap();
    let proposal_2 = binding.proposals.iter().find(|p| p.index == 1).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_2.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    println!("--------------------------------");
    println!("--------------------------------");

    println!("proposal_2: {:?}", proposal_2.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");
    assert!(second_proposal.is_ok());
    assert!(proposal_2.proposal_id.is_some());

    let proposal_2_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_2.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_2_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let third_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    let binding = third_proposal.clone().unwrap();
    let proposal_3 = binding.proposals.iter().find(|p| p.index == 2).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_3.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    println!("--------------------------------");
    println!("--------------------------------");

    println!("proposal_3: {:?}", proposal_3.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");

    assert!(third_proposal.is_ok());
    assert!(proposal_3.proposal_id.is_some());

    let proposal_3_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_3.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_3_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let non_existing_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    assert!(non_existing_proposal.is_err());
    println!("--------------------------------");
    println!("non_existing_proposal: {:?}", non_existing_proposal);
    println!("--------------------------------");

    Ok(())
}

#[test]
fn test_create_chain_proposals_with_manual_start_chain() -> Result<(), String> {
    let context = Context::new(true);
    let sns = context.sns.as_ref().unwrap();
    let service_canister_neuron_id = sns.prepare_neurons(&context)?.id;

    // these proposals will execute directly do to a majority of voting power
    let proposals = vec![
        PostSnsChainProposal {
            index: 0,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 1".to_string(),
                summary: "Test proposal 1".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 1".to_string(),
                })),
            },
        },
        PostSnsChainProposal {
            index: 1,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 2".to_string(),
                summary: "Test proposal 2".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 2".to_string(),
                })),
            },
        },
        PostSnsChainProposal {
            index: 2,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 3".to_string(),
                summary: "Test proposal 3".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 3".to_string(),
                })),
            },
        },
    ];

    let create_chain_proposals = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "create_chain_proposals",
        Some(encode_args((service_canister_neuron_id, proposals, false)).unwrap()),
    )?;

    assert!(create_chain_proposals.is_ok());

    let start_chain = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "start_chain",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    assert!(start_chain.is_ok());

    let binding = start_chain.clone().unwrap();
    let proposal_1 = binding.proposals.iter().find(|p| p.index == 0).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_1.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    println!("--------------------------------");
    println!("--------------------------------");
    println!("proposal_1: {:?}", proposal_1.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");

    assert!(create_chain_proposals.is_ok());
    assert!(proposal_1.proposal_id.is_some());

    let proposal_1_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_1.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_1_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let second_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    let binding = second_proposal.clone().unwrap();
    let proposal_2 = binding.proposals.iter().find(|p| p.index == 1).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_2.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    println!("--------------------------------");
    println!("--------------------------------");

    println!("proposal_2: {:?}", proposal_2.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");

    assert!(second_proposal.is_ok());
    assert!(proposal_2.proposal_id.is_some());

    let proposal_2_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_2.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_2_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let third_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    let binding = third_proposal.clone().unwrap();
    let proposal_3 = binding.proposals.iter().find(|p| p.index == 2).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_3.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    println!("--------------------------------");
    println!("--------------------------------");

    println!("proposal_3: {:?}", proposal_3.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");

    assert!(third_proposal.is_ok());
    assert!(proposal_3.proposal_id.is_some());

    let proposal_3_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_3.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_3_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let non_existing_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    assert!(non_existing_proposal.is_err());
    println!("--------------------------------");
    println!("--------------------------------");
    println!("non_existing_proposal: {:?}", non_existing_proposal);
    println!("--------------------------------");
    println!("--------------------------------");

    Ok(())
}

#[test]
fn test_create_chain_proposals_with_treasury_request() -> Result<(), String> {
    let context = Context::new(true);
    let sns = context.sns.as_ref().unwrap();
    let service_canister_neuron_id = sns.prepare_neurons(&context)?.id;

    // these proposals will execute directly do to a majority of voting power
    let proposals = vec![
        PostSnsChainProposal {
            index: 0,
            // critical proposal (not auto accepted)
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 1".to_string(),
                summary: "Test proposal 1".to_string(),
                action: Some(Action::MintSnsTokens(MintSnsTokens {
                    to_principal: Some(context.neuron_controller_canister),
                    to_subaccount: None,
                    memo: Some(1u64),
                    amount_e8s: Some(10_000_000_000),
                })),
            },
        },
        PostSnsChainProposal {
            index: 1,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 2".to_string(),
                summary: "Test proposal 2".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 2".to_string(),
                })),
            },
        },
        PostSnsChainProposal {
            index: 2,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 3".to_string(),
                summary: "Test proposal 3".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 3".to_string(),
                })),
            },
        },
    ];

    let create_chain_proposals = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "create_chain_proposals",
        Some(encode_args((service_canister_neuron_id, proposals, true)).unwrap()),
    )?;

    let binding = create_chain_proposals.clone().unwrap();
    let proposal_1 = binding.proposals.iter().find(|p| p.index == 0).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        5,
        Some(ProposalId {
            id: proposal_1.proposal_id.unwrap(),
        }),
        1,
        true,
    )?;

    for _ in 0..100 {
        context.pic.tick();
    }
    context.pic.advance_time(Duration::from_secs(60 * 60 * 2));
    for _ in 0..100 {
        context.pic.tick();
    }

    let confirm_proposal_1 = context.query::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "get_sns_chain_proposals",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    println!("confirm_proposal_1: {:?}", confirm_proposal_1);

    // for _ in 0..100 {
    //     context.pic.tick();
    // }

    println!("--------------------------------");
    println!("--------------------------------");
    println!("proposal_1: {:?}", proposal_1.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!(
        "current_index: {:?}",
        create_chain_proposals.clone().unwrap().current_index
    );
    println!("--------------------------------");
    println!("--------------------------------");
    assert!(create_chain_proposals.is_ok());
    assert!(proposal_1.proposal_id.is_some());

    let proposal_1_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_1.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_1_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let second_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    let binding = second_proposal.clone().unwrap();
    let proposal_2 = binding.proposals.iter().find(|p| p.index == 1).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_2.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    // for _ in 0..10 {
    //     context.pic.tick();
    // }

    println!("--------------------------------");
    println!("--------------------------------");
    println!(
        "current_index: {:?}",
        second_proposal.clone().unwrap().current_index
    );
    println!("proposal_2: {:?}", proposal_2.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");

    assert!(second_proposal.is_ok());
    assert!(proposal_2.proposal_id.is_some());

    let proposal_2_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_2.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_2_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let third_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    let binding = third_proposal.clone().unwrap();
    let proposal_3 = binding.proposals.iter().find(|p| p.index == 2).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_3.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    // for _ in 0..10 {
    //     context.pic.tick();
    // }

    println!("--------------------------------");
    println!("--------------------------------");
    println!(
        "current_index: {:?}",
        third_proposal.clone().unwrap().current_index
    );
    println!("proposal_3: {:?}", proposal_3.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");

    assert!(third_proposal.is_ok());
    assert!(proposal_3.proposal_id.is_some());

    let proposal_3_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_3.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_3_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let non_existing_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "execute_next_proposal",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    assert!(non_existing_proposal.is_err());
    println!("--------------------------------");
    println!("--------------------------------");
    println!("non_existing_proposal: {:?}", non_existing_proposal);
    println!("--------------------------------");
    println!("--------------------------------");

    Ok(())
}

#[test]
fn test_create_chain_proposals_with_treasury_request_and_timer() -> Result<(), String> {
    let context = Context::new(true);
    let sns = context.sns.as_ref().unwrap();
    let service_canister_neuron_id = sns.prepare_neurons(&context)?.id;

    // these proposals will execute directly do to a majority of voting power
    let proposals = vec![
        PostSnsChainProposal {
            index: 0,
            // critical proposal (not auto accepted)
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 1".to_string(),
                summary: "Test proposal 1".to_string(),
                action: Some(Action::MintSnsTokens(MintSnsTokens {
                    to_principal: Some(context.neuron_controller_canister),
                    to_subaccount: None,
                    memo: Some(1u64),
                    amount_e8s: Some(10_000),
                })),
            },
        },
        PostSnsChainProposal {
            index: 1,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 2".to_string(),
                summary: "Test proposal 2".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 2".to_string(),
                })),
            },
        },
        PostSnsChainProposal {
            index: 2,
            proposal: Proposal {
                url: "https://example.com".to_string(),
                title: "Test proposal 3".to_string(),
                summary: "Test proposal 3".to_string(),
                action: Some(Action::Motion(Motion {
                    motion_text: "Test proposal 3".to_string(),
                })),
            },
        },
    ];

    let create_chain_proposals = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "create_chain_proposals",
        Some(encode_args((service_canister_neuron_id, proposals, true)).unwrap()),
    )?;

    let binding = create_chain_proposals.clone().unwrap();
    let proposal_1 = binding.proposals.iter().find(|p| p.index == 0).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        5,
        Some(ProposalId {
            id: proposal_1.proposal_id.unwrap(),
        }),
        1,
        true,
    )?;

    for _ in 0..100 {
        context.pic.tick();
    }
    context.pic.advance_time(Duration::from_secs(60 * 60 * 2));
    for _ in 0..100 {
        context.pic.tick();
    }

    let confirm_proposal_1 = context.query::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "get_sns_chain_proposals",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    println!("confirm_proposal_1: {:?}", confirm_proposal_1);

    println!("--------------------------------");
    println!("--------------------------------");
    println!("proposal_1: {:?}", proposal_1.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!(
        "current_index: {:?}",
        create_chain_proposals.clone().unwrap().current_index
    );
    println!("--------------------------------");
    println!("--------------------------------");
    assert!(create_chain_proposals.is_ok());
    assert!(proposal_1.proposal_id.is_some());

    let proposal_1_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_1.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_1_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    for _ in 0..100 {
        context.pic.tick();
    }
    context.pic.advance_time(Duration::from_secs(60 * 60 * 2));
    for _ in 0..100 {
        context.pic.tick();
    }

    let second_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "get_sns_chain_proposals",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    let binding = second_proposal.clone().unwrap();
    let proposal_2 = binding.proposals.iter().find(|p| p.index == 1).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_2.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    for _ in 0..100 {
        context.pic.tick();
    }
    context.pic.advance_time(Duration::from_secs(60 * 60 * 2));
    for _ in 0..100 {
        context.pic.tick();
    }

    println!("--------------------------------");
    println!("--------------------------------");
    println!(
        "current_index: {:?}",
        second_proposal.clone().unwrap().current_index
    );
    println!("proposal_2: {:?}", proposal_2.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");

    assert!(second_proposal.is_ok());
    assert!(proposal_2.proposal_id.is_some());

    let proposal_2_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_2.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_2_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    for _ in 0..100 {
        context.pic.tick();
    }
    context.pic.advance_time(Duration::from_secs(60 * 60 * 2));
    for _ in 0..100 {
        context.pic.tick();
    }

    let third_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "get_sns_chain_proposals",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    let binding = third_proposal.clone().unwrap();
    let proposal_3 = binding.proposals.iter().find(|p| p.index == 2).unwrap();

    let vote_result = sns.vote_with_participants_count(
        &context,
        3,
        Some(ProposalId {
            id: proposal_3.proposal_id.unwrap(),
        }),
        1,
        false,
    )?;

    for _ in 0..100 {
        context.pic.tick();
    }
    context.pic.advance_time(Duration::from_secs(60 * 60 * 2));
    for _ in 0..100 {
        context.pic.tick();
    }

    println!("--------------------------------");
    println!("--------------------------------");
    println!(
        "current_index: {:?}",
        third_proposal.clone().unwrap().current_index
    );
    println!("proposal_3: {:?}", proposal_3.proposal_id);
    println!(
        "amount of votes: (also includes failed votes) {:?}",
        vote_result.len()
    );
    println!("--------------------------------");
    println!("--------------------------------");

    assert!(third_proposal.is_ok());
    assert!(proposal_3.proposal_id.is_some());

    let proposal_3_check = sns.get_sns_proposal(
        &context.pic,
        Some(ProposalId {
            id: proposal_3.proposal_id.unwrap(),
        }),
        Sender::Other(context.config.governance_canister_id),
    )?;

    assert!(matches!(
        proposal_3_check.result,
        Some(Result1::Proposal(ref proposal)) if proposal.executed_timestamp_seconds > 0
    ));

    let non_existing_proposal = context.update::<CanisterResult<SnsChainProposalsResponse>>(
        Sender::Other(context.config.governance_canister_id),
        "get_sns_chain_proposals",
        Some(encode_args((&create_chain_proposals.clone().unwrap().id,)).unwrap()),
    )?;

    assert!(non_existing_proposal.is_ok());
    println!("--------------------------------");
    println!("--------------------------------");
    println!("non_existing_proposal: {:?}", non_existing_proposal);
    println!("--------------------------------");
    println!("--------------------------------");

    Ok(())
}
