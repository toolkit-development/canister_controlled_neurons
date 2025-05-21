use candid::Principal;
use ic_cdk::{msg_caller, update};
use toolkit_utils::{api_error::ApiError, result::CanisterResult};

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
pub async fn test_tk_service_manage_neuron(args: NeuronType) -> CanisterResult<ModuleResponse> {
    if msg_caller()
        != Principal::from_text("xsqgv-477xy-akqcs-v3tue-2skfn-s3s3c-6v5b4-7jpxs-pbxmi-outv4-oqe")
            .unwrap()
    {
        return Err(ApiError::bad_request("Unauthorized"));
    }
    GenericLogic::tk_service_manage_neuron(args).await
}

#[update]
pub async fn tk_service_validate_manage_neuron(args: NeuronType) -> Result<String, String> {
    is_governance_canister().map_err(|e| e.to_string())?;
    GenericLogic::tk_service_validate_manage_neuron(args)
        .await
        .map_err(|e| e.to_string())
}
