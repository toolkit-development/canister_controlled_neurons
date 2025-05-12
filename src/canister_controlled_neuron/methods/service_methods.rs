use ic_cdk::update;
use toolkit_utils::result::CanisterResult;

use crate::{
    logic::generic_logic::GenericLogic,
    misc::guards::is_governance_canister,
    types::modules::{ModuleResponse, NeuronType},
};

#[update]
pub async fn tk_service_manage_neuron(args: NeuronType) -> CanisterResult<ModuleResponse> {
    is_governance_canister()?;
    GenericLogic::tk_service_manage_neuron(args).await
}

#[update]
pub async fn tk_service_validate_manage_neuron(args: NeuronType) -> Result<String, String> {
    is_governance_canister().map_err(|e| e.to_string())?;
    GenericLogic::tk_service_validate_manage_neuron(args)
        .await
        .map_err(|e| e.to_string())
}
