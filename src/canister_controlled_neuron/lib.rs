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
use ic_cdk::{init, query};
use logic::config_logic::ConfigLogic;
use types::config::Config;

#[init]
pub fn init(canisters: Config) {
    let _ = ConfigLogic::init(
        canisters.governance_canister_id,
        canisters.sns_ledger_canister_id,
    );
}

#[query]
pub fn __get_candid_interface_tmp_hack() -> String {
    use crate::api::icp_governance_api::Neuron as GovNeuron;
    use crate::types::config::Config;
    use crate::types::modules::*;
    use crate::types::neuron_reference::NeuronReferenceResponse;
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
