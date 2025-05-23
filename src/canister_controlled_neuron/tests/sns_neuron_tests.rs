use test_helper::{
    context::Context,
    declarations::sns_governance_api::{Action, Command, Motion, Proposal, ProposalId},
    sender::Sender,
};

#[test]
fn test_get_config() -> Result<(), String> {
    let context = Context::new(true);
    let sns_context = context.sns.as_ref().unwrap();

    println!("sns_neurons: {:?}", sns_context.sns_neurons.len());

    let motion_proposal = sns_context.sns_command(
        &context.pic,
        sns_context.sns_neurons[0].id.clone().unwrap(),
        Command::MakeProposal(Proposal {
            url: "https://example.com".to_string(),
            title: "Test motion".to_string(),
            summary: "Test description".to_string(),
            action: Some(Action::Motion(Motion {
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
