use test_helper::context::Context;

#[test]
fn test_get_config() -> Result<(), String> {
    let context = Context::new();
    Ok(())
}

#[test]
fn deploy_sns_test() -> Result<(), String> {
    let context = Context::new();

    let balance = context.get_icp_balance(context.owner_account.owner);
    println!("balance: {:?}", balance);
    assert!(balance.is_ok());

    // setup an ICP neuron

    // context.pic.update_call(
    //     Principal::from_text(context.owner_account.owner).unwrap(),
    //     "create_service_nervous_system",
    //     Some(context.config.encode_to_vec()),
    // )?;
    Ok(())
}
