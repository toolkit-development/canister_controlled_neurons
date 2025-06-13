#![allow(deprecated)] // This is an experimental feature to generate Rust binding from Candid.
use candid::{Encode, Principal};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, CanisterInstallMode, CreateCanisterArgument, InstallCodeArgument,
};
use ic_cdk::api::management_canister::provisional::CanisterSettings;
use toolkit_utils::{
    result::CanisterResult,
    storage::{StorageInsertableByKey, StorageQueryable},
};

use crate::api::canister_controlled_neuron_api::Config;
use crate::misc::ic_utils::check_payment;
use crate::{
    storage::{
        module_proposals_storage::ModuleProposalsStore,
        service_canisters_storage::ServiceCanistersStore,
    },
    types::{
        proposal::{PurchaseModulesProposalData, ServiceData},
        service_canisters::{GovernanceCanisterId, ServiceCanisterId},
    },
};


pub struct ServiceCanistersLogic;

impl ServiceCanistersLogic {
    // ===============================
    // Public methods
    // ===============================

    pub fn get_service_canisters() -> CanisterResult<Vec<ServiceCanisterId>> {
        let canisters = ServiceCanistersStore::get_all();
        Ok(canisters
            .into_iter()
            .map(|(_, canister)| canister)
            .collect())
    }

    pub fn set_service_canisters(
        governance_canister_id: GovernanceCanisterId,
        service_canister_id: ServiceCanisterId,
    ) {
        ServiceCanistersStore::upsert_by_key(governance_canister_id, service_canister_id);
    }

    pub async fn activate_modules(
        governance_canister_id: GovernanceCanisterId,
    ) -> CanisterResult<()> {
        let (_, service_data) = ModuleProposalsStore::get(governance_canister_id)?;
        
        for proposal in &service_data.proposals {
            Self::process_proposal(governance_canister_id, proposal, &service_data).await?;
        }
        
        Ok(())
    }

    async fn process_proposal(
        governance_canister_id: GovernanceCanisterId,
        proposal: &PurchaseModulesProposalData,
        service_data: &ServiceData,
    ) -> CanisterResult<()> {
        // Verify payment was received
        check_payment(governance_canister_id, proposal).await?;

        // Get or create service canister
        let service_canister_id = Self::get_or_create_service_canister(
            governance_canister_id,
            service_data,
        ).await?;

        ic_cdk::println!("Service canister id: {}", service_canister_id);

        // TODO: activate modules
        Ok(())
    }

    async fn get_or_create_service_canister(
        governance_canister_id: GovernanceCanisterId,
        service_data: &ServiceData,
    ) -> CanisterResult<ServiceCanisterId> {
        match ServiceCanistersStore::get(governance_canister_id) {
            Ok((_, canister_id)) => Ok(canister_id),
            Err(_) => {
                let canister_id = Self::deploy_service_canister(
                    service_data.ledger_canister_id,
                    governance_canister_id,
                ).await?;
                Self::set_service_canisters(governance_canister_id, canister_id);
                Ok(canister_id)
            }
        }
    }

    async fn deploy_service_canister(
        sns_ledger_canister_id: Principal,
        governance_canister_id: GovernanceCanisterId,
    ) -> CanisterResult<ServiceCanisterId> {
        // Step 1: Create the canister
        let create_args = CreateCanisterArgument {
            settings: Some(CanisterSettings {
                controllers: Some(vec![governance_canister_id]),
                compute_allocation: None,
                memory_allocation: None,
                freezing_threshold: None,
                reserved_cycles_limit: None,
                log_visibility: None,
                wasm_memory_limit: None,
            }),
        };
        let canister_record = create_canister(create_args, 10_000_000_000_000)
            .await
            .unwrap();
        let canister_id = canister_record.0.canister_id;
        let config = Config {
            sns_ledger_canister_id,
            governance_canister_id,
        };
        
        // Encode as Candid
        let arg = Encode!(&config).expect("Candid encoding failed");
        let install_args = InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module: include_bytes!("../../../wasm/canister_controlled_neuron.wasm.gz")
                .to_vec(),
            arg,
        };
        install_code(install_args).await.unwrap();

        Ok(canister_id)
    }
}
