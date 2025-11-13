pub mod api;
pub mod logic;
pub mod methods;
pub mod misc;
pub mod storage;
pub mod types;

use candid::export_service;
use ic_cdk::query;

#[query]
pub fn __get_candid_interface_tmp_hack() -> String {
    use crate::types::service_canisters::{
        GovernanceCanisterId, RootCanisterId, ServiceCanisterId, ServiceCanisterModules,
    };
    use crate::types::proposal::PurchaseModulesProposalData;
    use toolkit_utils::result::CanisterResult;
    export_service!();
    __export_service()
}

#[test]
pub fn candid() {
    use std::env;
    use std::fs::write;
    use std::path::PathBuf;

    let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    write(
        dir.join("service_index.did"),
        __get_candid_interface_tmp_hack(),
    )
    .expect("Write failed.");
}
