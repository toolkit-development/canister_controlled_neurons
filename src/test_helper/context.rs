use std::{env, path::PathBuf};

use candid::{encode_args, CandidType, Decode, Nat, Principal};
use canister_controlled_neuron::api::icp_governance_api::ProposalInfo;
use canister_controlled_neuron::types::config::Config;

use ic_management_canister_types::CanisterSettings;
use pocket_ic::{PocketIc, PocketIcBuilder};
use serde::de::DeserializeOwned;
use toolkit_utils::ic_ledger_types::{
    Subaccount, MAINNET_GOVERNANCE_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use toolkit_utils::icrc_ledger_types::icrc1::account::Account;
use toolkit_utils::icrc_ledger_types::icrc1::transfer::TransferArg;
use toolkit_utils::icrc_ledger_types::icrc2::allowance::{Allowance, AllowanceArgs};
use toolkit_utils::icrc_ledger_types::icrc2::approve::ApproveArgs;

use crate::sender::Sender;
use crate::utils::generate_principal;

pub static OWNER_PRINCIPAL: &str =
    "vafd2-aurwj-5igu3-htth5-olb42-6ficf-ttehy-2oyrp-u6nsy-qjlay-7ae";

pub struct Context {
    pub pic: PocketIc,
    pub owner_account: Account,
    pub neuron_controller_canister: Principal,
    pub config: Config,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Self {
        let owner_account = Account::from(Principal::from_text(OWNER_PRINCIPAL).unwrap());

        let default_install_settings: Option<CanisterSettings> = Some(CanisterSettings {
            controllers: Some(vec![owner_account.owner]),
            compute_allocation: None,
            memory_allocation: None,
            freezing_threshold: None,
            reserved_cycles_limit: None,
            log_visibility: None,
            wasm_memory_limit: None,
            wasm_memory_threshold: None,
        });

        if !PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .expect("Failed to get parent dir")
            .join("test_helper/nns_state")
            .exists()
        {
            panic!("NNS state not found. Please run `bash scripts/prepare_test.sh` to load the NNS state.");
        }

        let nns_state_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .expect("Failed to get parent dir")
            .join("test_helper/nns_state");

        let pic = PocketIcBuilder::new()
            .with_nns_subnet()
            .with_nns_state(nns_state_path) // this first included state is the nns subnet id
            .with_sns_subnet()
            .with_application_subnet()
            .build();

        let canister_controlled_neuron_canister =
            pic.create_canister_with_settings(None, default_install_settings.clone());

        pic.add_cycles(canister_controlled_neuron_canister, 2_000_000_000_000);

        let canister_controlled_neuron_wasm_bytes =
            include_bytes!("../../wasm/canister_controlled_neuron.wasm.gz");

        let config = Config {
            governance_canister_id: generate_principal(),
            sns_ledger_canister_id: generate_principal(),
        };

        pic.install_canister(
            canister_controlled_neuron_canister,
            canister_controlled_neuron_wasm_bytes.to_vec(),
            encode_args((config.clone(),)).unwrap(),
            Some(owner_account.owner),
        );

        let context = Context {
            pic,
            owner_account,
            neuron_controller_canister: canister_controlled_neuron_canister,
            config,
        };

        context.mint_icp(100_000_000_000_000_000, owner_account.owner);
        context
    }

    pub fn query<T: DeserializeOwned + CandidType>(
        &self,
        sender: Sender,
        method: &str,
        args: Option<Vec<u8>>,
    ) -> Result<T, String> {
        let args = args.unwrap_or(encode_args(()).unwrap());
        let res = self.pic.query_call(
            self.neuron_controller_canister,
            sender.principal(),
            method,
            args,
        );

        match res {
            Ok(res) => Decode!(res.as_slice(), T).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn update<T: DeserializeOwned + CandidType>(
        &self,
        sender: Sender,
        method: &str,
        args: Option<Vec<u8>>,
    ) -> Result<T, String> {
        let args = args.unwrap_or(encode_args(()).unwrap());
        let res = self.pic.update_call(
            self.neuron_controller_canister,
            sender.principal(),
            method,
            args,
        );

        match res {
            Ok(res) => Decode!(res.as_slice(), T).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn mint_icp(&self, amount: u64, user_principal: Principal) {
        let transfer_args = TransferArg {
            from_subaccount: None,
            to: Account {
                owner: user_principal,
                subaccount: None,
            },
            fee: None,
            created_at_time: None,
            memo: None,
            amount: Nat::from(amount),
        };

        self.pic
            .update_call(
                MAINNET_LEDGER_CANISTER_ID,
                MAINNET_GOVERNANCE_CANISTER_ID,
                "icrc1_transfer",
                encode_args((transfer_args,)).unwrap(),
            )
            .expect("Failed to call canister");
    }

    pub fn transfer_icp(&self, amount: u64, from: Account, to: Account) {
        let transfer_args = TransferArg {
            from_subaccount: None,
            to,
            fee: None,
            created_at_time: None,
            memo: None,
            amount: Nat::from(amount),
        };

        self.pic
            .update_call(
                MAINNET_LEDGER_CANISTER_ID,
                from.owner,
                "icrc1_transfer",
                encode_args((transfer_args,)).unwrap(),
            )
            .expect("Failed to call canister");
    }

    pub fn get_proposal(
        &self,
        proposal_id: u64,
        sender: Sender,
    ) -> Result<Option<ProposalInfo>, String> {
        let res = self.pic.query_call(
            MAINNET_GOVERNANCE_CANISTER_ID,
            sender.principal(),
            "get_proposal_info",
            encode_args((proposal_id,)).unwrap(),
        );

        match res {
            Ok(res) => Decode!(res.as_slice(), Option<ProposalInfo>).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn mint_icp_subaccount(
        &self,
        amount: u64,
        user_principal: Principal,
        subaccount: Subaccount,
    ) {
        let transfer_args = TransferArg {
            from_subaccount: None,
            to: Account {
                owner: user_principal,
                subaccount: Some(subaccount.0),
            },
            fee: None,
            created_at_time: None,
            memo: None,
            amount: Nat::from(amount),
        };

        self.pic
            .update_call(
                MAINNET_LEDGER_CANISTER_ID,
                MAINNET_GOVERNANCE_CANISTER_ID,
                "icrc1_transfer",
                encode_args((transfer_args,)).unwrap(),
            )
            .expect("Failed to call canister");
    }

    pub fn approve_icp(
        &self,
        from_subaccount: Option<Subaccount>,
        sender: Principal,
        spender: Account,
        amount: u64,
        expires_at: Option<u64>,
    ) {
        let approve_args = ApproveArgs {
            from_subaccount: from_subaccount.map(|subaccount| subaccount.0),
            spender,
            amount: Nat::from(amount),
            expected_allowance: None,
            expires_at,
            fee: None,
            memo: None,
            created_at_time: None,
        };

        self.pic
            .update_call(
                MAINNET_LEDGER_CANISTER_ID,
                sender,
                "icrc2_approve",
                encode_args((approve_args,)).unwrap(),
            )
            .expect("Failed to call canister");
    }

    pub fn get_allowance(&self, owner: Account, spender: Account) -> Result<Allowance, String> {
        let allowance_args = AllowanceArgs {
            account: owner,
            spender,
        };

        let icp_allowance_result = self
            .pic
            .update_call(
                MAINNET_LEDGER_CANISTER_ID,
                MAINNET_GOVERNANCE_CANISTER_ID,
                "icrc2_allowance",
                encode_args((allowance_args,)).unwrap(),
            )
            .expect("Failed to call canister");

        Decode!(icp_allowance_result.as_slice(), Allowance).map_err(|e| e.to_string())
    }

    pub fn get_icp_balance(&self, user_principal: Principal) -> Result<Nat, String> {
        let icp_balance_result = self
            .pic
            .update_call(
                MAINNET_LEDGER_CANISTER_ID,
                MAINNET_GOVERNANCE_CANISTER_ID,
                "icrc1_balance_of",
                encode_args((Account::from(user_principal),)).unwrap(),
            )
            .expect("Failed to call canister");

        Decode!(icp_balance_result.as_slice(), Nat).map_err(|e| e.to_string())
    }
}

// use canister_test::Wasm;
// use ic_base_types::{CanisterId, PrincipalId};
// use ic_nervous_system_common::ONE_MONTH_SECONDS;
// use ic_nervous_system_integration_tests::{
//     create_service_nervous_system_builder::CreateServiceNervousSystemBuilder,
//     pocket_ic_helpers,
//     pocket_ic_helpers::{
//         add_wasm_via_nns_proposal, install_canister, nns, sns,
//         upgrade_nns_canister_to_tip_of_master_or_panic,
//     },
// };
// use ic_nns_constants::{self, GOVERNANCE_CANISTER_ID, SNS_WASM_CANISTER_ID};
// use ic_nns_test_utils::sns_wasm::{
//     build_archive_sns_wasm, build_index_ng_sns_wasm, build_ledger_sns_wasm,
// };
// use ic_test_utilities::universal_canister::UNIVERSAL_CANISTER_WASM;

// #[tokio::test]
// async fn test_deploy_fresh_sns() {
//     let create_service_nervous_system = CreateServiceNervousSystemBuilder::default()
//         .with_governance_parameters_neuron_minimum_dissolve_delay_to_vote(ONE_MONTH_SECONDS * 6)
//         .with_one_developer_neuron(
//             PrincipalId::new_user_test_id(830947),
//             ONE_MONTH_SECONDS * 6,
//             756575,
//             0,
//         )
//         .build();

//     let dapp_canister_ids: Vec<_> = create_service_nervous_system
//         .dapp_canisters
//         .iter()
//         .map(|canister| CanisterId::unchecked_from_principal(canister.id.unwrap()))
//         .collect();

//     eprintln!("1. Prepare the world (use mainnet WASMs for all NNS and SNS canisters) ...");
//     let (pocket_ic, _initial_sns_version) =
//         pocket_ic_helpers::pocket_ic_for_sns_tests_with_mainnet_versions().await;

//     eprintln!("Install the test dapp ...");
//     for dapp_canister_id in dapp_canister_ids.clone() {
//         install_canister(
//             &pocket_ic,
//             "My Test Dapp",
//             dapp_canister_id,
//             vec![],
//             Wasm::from_bytes(UNIVERSAL_CANISTER_WASM.to_vec()),
//             None,
//         )
//         .await;
//     }

//     eprintln!("Step 1. Upgrade NNS Governance and SNS-W to the latest version ...");
//     upgrade_nns_canister_to_tip_of_master_or_panic(&pocket_ic, GOVERNANCE_CANISTER_ID).await;

//     upgrade_nns_canister_to_tip_of_master_or_panic(&pocket_ic, SNS_WASM_CANISTER_ID).await;

//     eprintln!("Test upgrading SNS Ledger via proposals. First, add all the WASMs to SNS-W ...");
//     {
//         let wasm = build_index_ng_sns_wasm();
//         let proposal_info = add_wasm_via_nns_proposal(&pocket_ic, wasm).await.unwrap();
//         assert_eq!(proposal_info.failure_reason, None);
//     }
//     {
//         let wasm = build_ledger_sns_wasm();
//         let proposal_info = add_wasm_via_nns_proposal(&pocket_ic, wasm).await.unwrap();
//         assert_eq!(proposal_info.failure_reason, None);
//     }
//     {
//         let wasm = build_archive_sns_wasm();
//         let proposal_info = add_wasm_via_nns_proposal(&pocket_ic, wasm).await.unwrap();
//         assert_eq!(proposal_info.failure_reason, None);
//     }

//     // ---------------------------
//     // --- Run code under test ---
//     // ---------------------------

//     eprintln!("Deploy an SNS instance via proposal ...");
//     let sns_instance_label = "1";
//     let (sns, _) = nns::governance::propose_to_deploy_sns_and_wait(
//         &pocket_ic,
//         create_service_nervous_system,
//         sns_instance_label,
//     )
//     .await;

//     eprintln!("Testing the Archive canister requires that it can be spawned ...");
//     sns::ensure_archive_canister_is_spawned_or_panic(
//         &pocket_ic,
//         sns.governance.canister_id,
//         sns.ledger.canister_id,
//     )
//     .await;
//     // TODO eventually we need to test a swap
// }
