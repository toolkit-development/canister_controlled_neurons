use candid::Principal;
use ic_cdk::api::msg_caller;
use toolkit_utils::{api_error::ApiError, result::CanisterResult};

pub fn check_authorized_principal(caller: Principal) -> CanisterResult<()> {
    // const AUTHORIZED_PRINCIPAL: &str = "rr7if-rl2nx-kdawm-hhgiw-5vbnd-ezfax-z5m5b-z3kkq-oomoj-ewdpo-qae";

    // Local auth principal
    const AUTHORIZED_PRINCIPAL: &str =
        "zz3kx-z5d4u-h7lbt-ul2o2-g7t6w-4v2cc-voh5y-gq3ph-unj3t-ywedq-pqe";

    if caller.to_string() != AUTHORIZED_PRINCIPAL {
        return Err(ApiError::forbidden("Not authorized to perform this action"));
    }
    Ok(())
}

pub fn is_not_anonymous() -> CanisterResult<()> {
    if msg_caller() == Principal::anonymous() {
        return Err(ApiError::forbidden("Caller is anonymous").add_method_name("is_not_anonymous"));
    }

    Ok(())
}