use std::{env, path::PathBuf};

use candid::{encode_args, CandidType, Decode, Nat, Principal};
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
