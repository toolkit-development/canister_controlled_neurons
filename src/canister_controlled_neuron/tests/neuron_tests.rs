use canister_controlled_neuron::types::config::Config;
use test_helper::{context::Context, sender::Sender};
use toolkit_utils::result::CanisterResult;

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
