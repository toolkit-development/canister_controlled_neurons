use ic_cdk::api::msg_caller;
use toolkit_utils::{api_error::ApiError, result::CanisterResult};

use crate::logic::config_logic::ConfigLogic;

pub fn is_governance_canister() -> CanisterResult<()> {
    if ConfigLogic::get_config()?.governance_canister_id != msg_caller() {
        return Err(ApiError::forbidden("Caller is not owner"));
    }
    Ok(())
}
