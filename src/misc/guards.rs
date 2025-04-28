use ic_cdk::api::msg_caller;

use crate::{
    logic::config_logic::ConfigLogic,
    types::{api_error::ApiError, result::CanisterResult},
};

pub fn is_authorized() -> CanisterResult<()> {
    if !ConfigLogic::get_config()?.owners.contains(&msg_caller()) {
        return Err(ApiError::forbidden("Caller is not owner"));
    }
    Ok(())
}
