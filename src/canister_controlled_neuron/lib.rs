pub mod api;
pub mod helpers;
pub mod logic;
pub mod methods;
pub mod misc;
pub mod storage;
pub mod timers;
pub mod traits;
pub mod types;

use candid::export_service;
use ic_cdk::query;

#[query]
pub fn __get_candid_interface_tmp_hack() -> String {
    use crate::api::icp_governance_api::Neuron as GovNeuron;
    use crate::types::args::icp_neuron_args::IcpNeuronIdentifier;
    use crate::types::config::Config;
    use crate::types::icp_neuron_reference::IcpNeuronReferenceResponse;
    use crate::types::modules::*;
    use toolkit_utils::icrc_types::*;
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
        dir.join("canister_controlled_neuron.did"),
        __get_candid_interface_tmp_hack(),
    )
    .expect("Write failed.");
}
