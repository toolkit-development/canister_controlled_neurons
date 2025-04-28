pub mod api;
pub mod helpers;
pub mod logic;
pub mod methods;
pub mod misc;
pub mod storage;
pub mod traits;
pub mod types;

use candid::export_service;
use ic_cdk::query;

#[query]
pub fn __get_candid_interface_tmp_hack() -> String {
    use crate::api::governance_api::{ListNeuronsResponse, NeuronInfo};
    use crate::types::neuron::NeuronReferenceResponse;
    use crate::types::result::CanisterResult;
    use candid::Principal;
    use toolkit_utils::icrc_types::*;
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
        dir.join("canister_controlled_neuron.did"),
        __get_candid_interface_tmp_hack(),
    )
    .expect("Write failed.");
}
