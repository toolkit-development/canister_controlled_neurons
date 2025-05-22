use candid::{encode_args, CandidType, Decode, Encode, Principal};
use canister_controlled_neuron::api::icp_governance_api::{
    By, ClaimOrRefresh, ClaimOrRefreshResponse, Command, Command1, Configure, DissolveState,
    ExecuteNnsFunction, IncreaseDissolveDelay, MakeProposalRequest, MakeProposalResponse,
    ManageNeuron, ManageNeuronCommandRequest, ManageNeuronRequest, ManageNeuronResponse, NeuronId,
    NeuronIdOrSubaccount, Operation, ProposalActionRequest, Result2,
};
use ic_ledger_types::MAINNET_GOVERNANCE_CANISTER_ID;
use pocket_ic::common::rest::RawEffectivePrincipal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use test_helper::{context::Context, sender::Sender};
use toolkit_utils::icrc_ledger_types::icrc1::account::Account;

#[test]
fn test_get_config() -> Result<(), String> {
    let context = Context::new();
    Ok(())
}

#[test]
fn deploy_sns_test() -> Result<(), String> {
    let context = Context::new();

    let balance = context.get_icp_balance(context.owner_account.owner);
    println!("initial user balance: {:?}", balance);
    println!("--------------------------------");
    assert!(balance.is_ok());

    let subaccount = generate_subaccount_by_nonce(1, context.owner_account.owner);
    context.transfer_icp(
        100_000_000_000_000_000,
        Account {
            owner: context.owner_account.owner,
            subaccount: None,
        },
        Account {
            owner: MAINNET_GOVERNANCE_CANISTER_ID,
            subaccount: Some(subaccount),
        },
    );

    let balance =
        context.get_icp_balance_with_subaccount(MAINNET_GOVERNANCE_CANISTER_ID, subaccount);
    println!("governance + subaccount balance: {:?}", balance);
    println!("--------------------------------");
    assert!(balance.is_ok());

    // claim or refresh neuron
    let claim_or_refresh_args = ManageNeuron {
        id: None,
        command: Some(Command::ClaimOrRefresh(ClaimOrRefresh {
            by: Some(By::Memo(1)),
        })),
        neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::Subaccount(subaccount.to_vec())),
    };

    let result = context
        .pic
        .update_call(
            MAINNET_GOVERNANCE_CANISTER_ID,
            context.owner_account.owner,
            "manage_neuron",
            encode_args((claim_or_refresh_args,)).unwrap(),
        )
        .expect("Failed to call canister");

    let manage_neuron_result =
        Decode!(result.as_slice(), ManageNeuronResponse).map_err(|e| e.to_string())?;

    println!("claim_or_refresh_result: {:?}", manage_neuron_result);
    println!("--------------------------------");

    let neuron_id = match manage_neuron_result.command.unwrap() {
        Command1::ClaimOrRefresh(ClaimOrRefreshResponse {
            refreshed_neuron_id,
        }) => refreshed_neuron_id,
        _ => return Err("Invalid command".to_string()),
    };

    // get new neuron
    let result = context
        .pic
        .query_call(
            MAINNET_GOVERNANCE_CANISTER_ID,
            context.owner_account.owner,
            "get_full_neuron",
            encode_args((neuron_id.clone().unwrap().id,)).unwrap(),
        )
        .expect("Failed to call canister");

    let get_neuron_result = Decode!(result.as_slice(), Result2).map_err(|e| e.to_string())?;
    assert!(matches!(get_neuron_result, Result2::Ok(_)));
    let neuron = match get_neuron_result {
        Result2::Ok(neuron) => {
            assert!(matches!(
                neuron.dissolve_state,
                Some(DissolveState::DissolveDelaySeconds(604800))
            ));
            neuron
        }
        _ => return Err("Invalid result".to_string()),
    };
    println!("newly created neuron: {:?}", neuron);
    println!("--------------------------------");

    // set max dissolve delay
    let dissolve_delay_args = ManageNeuron {
        id: neuron_id.clone(),
        command: Some(Command::Configure(Configure {
            operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                additional_dissolve_delay_seconds: 8 * 365 * 24 * 60 * 60,
            })),
        })),
        neuron_id_or_subaccount: None,
    };

    let result = context
        .pic
        .update_call(
            MAINNET_GOVERNANCE_CANISTER_ID,
            context.owner_account.owner,
            "manage_neuron",
            encode_args((dissolve_delay_args,)).unwrap(),
        )
        .expect("Failed to call canister");

    let manage_neuron_result =
        Decode!(result.as_slice(), ManageNeuronResponse).map_err(|e| e.to_string())?;

    println!("dissolve_delay_result: {:?}", manage_neuron_result);
    println!("--------------------------------");
    // check if the dissolve delay is increased
    // get new neuron
    let result = context
        .pic
        .query_call(
            MAINNET_GOVERNANCE_CANISTER_ID,
            context.owner_account.owner,
            "get_full_neuron",
            encode_args((neuron_id.clone().unwrap().id,)).unwrap(),
        )
        .expect("Failed to call canister");

    let get_neuron_result = Decode!(result.as_slice(), Result2).map_err(|e| e.to_string())?;
    assert!(matches!(get_neuron_result, Result2::Ok(_)));
    let neuron = match get_neuron_result {
        Result2::Ok(neuron) => {
            assert!(matches!(
                neuron.dissolve_state,
                Some(DissolveState::DissolveDelaySeconds(252460800))
            ));
            neuron
        }
        _ => return Err("Invalid result".to_string()),
    };
    println!("updated neuron with dissolve delay: {:?}", neuron);
    println!("--------------------------------");

    // set wasms via proposal

    let sns_governance_wasm = include_bytes!(
        "../../test_helper/sns_testing_bundle/rs/sns/governance/sns-governance-canister.wasm.gz"
    );

    let governance = add_wasm_via_proposal(
        &context,
        neuron_id,
        SnsWasm {
            wasm: sns_governance_wasm.to_vec(),
            canister_type: SnsCanisterType::Governance,
            proposal_id: None,
        },
    )?;

    println!("governance proposal: {:?}", governance);
    println!("--------------------------------");

    let governance = match governance.command.unwrap() {
        Command1::MakeProposal(MakeProposalResponse {
            proposal_id,
            message: _,
        }) => proposal_id.unwrap(),
        _ => return Err("Invalid command".to_string()),
    };

    for _ in 0..100 {
        context.pic.tick();
        // context.pic.advance_time(Duration::from_secs(3600));
    }
    let ledger_proposal_info = context.get_proposal(governance.id, Sender::Owner);
    println!(
        "governance proposal failure reason: {:?}",
        ledger_proposal_info
            .clone()
            .unwrap()
            .unwrap()
            .failure_reason
    );
    println!("--------------------------------");
    assert!(ledger_proposal_info.is_ok());
    Ok(())
}

pub fn generate_subaccount_by_nonce(nonce: u64, canister_id: Principal) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update([0x0c]);
    hasher.update(b"neuron-stake");

    hasher.update(canister_id.as_slice());

    hasher.update(nonce.to_be_bytes());

    let hash_result = hasher.finalize();

    let mut subaccount = [0u8; 32];
    subaccount.copy_from_slice(&hash_result[..]);

    subaccount
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct AddWasmRequest {
    pub hash: Vec<u8>,
    pub wasm: Option<SnsWasm>,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct SnsWasm {
    pub wasm: Vec<u8>,
    pub canister_type: SnsCanisterType,
    pub proposal_id: Option<u64>,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
pub enum SnsCanisterType {
    Unspecified = 0,
    /// The type for the root canister.
    Root = 1,
    /// The type for the governance canister.
    Governance = 2,
    /// The type for the ledger canister.
    Ledger = 3,
    /// The type for the swap canister.
    Swap = 4,
    /// The type for the ledger archive canister.
    Archive = 5,
    /// The type for the index canister.
    Index = 6,
}

impl From<i32> for SnsCanisterType {
    fn from(n: i32) -> Self {
        match n {
            0 => SnsCanisterType::Unspecified,
            1 => SnsCanisterType::Root,
            2 => SnsCanisterType::Governance,
            3 => SnsCanisterType::Ledger,
            4 => SnsCanisterType::Swap,
            5 => SnsCanisterType::Archive,
            6 => SnsCanisterType::Index,
            _ => panic!("Invalid value for SnsCanisterType: {}", n),
        }
    }
}

pub fn add_wasm_via_proposal(
    context: &Context,
    neuron_id: Option<NeuronId>,
    wasm: SnsWasm,
) -> Result<ManageNeuronResponse, String> {
    let hash = hash(&wasm.wasm.clone());
    let payload = AddWasmRequest {
        hash: hash.to_vec(),
        wasm: Some(wasm.clone()),
    };

    let proposal = MakeProposalRequest {
        title: Some(format!(
            "Add WASM for SNS canister type {:?}",
            wasm.canister_type.clone()
        )),
        summary: "summary".to_string(),
        url: "".to_string(),
        action: Some(ProposalActionRequest::ExecuteNnsFunction(
            ExecuteNnsFunction {
                nns_function: 30_i32, // AddSnsWasm
                payload: Encode!(&payload).expect("Error encoding proposal payload"),
            },
        )),
    };

    let manage_neuron_args = ManageNeuronRequest {
        id: neuron_id,
        command: Some(ManageNeuronCommandRequest::MakeProposal(proposal)),
        neuron_id_or_subaccount: None,
    };

    let result = context
        .pic
        .update_call_with_effective_principal(
            MAINNET_GOVERNANCE_CANISTER_ID,
            RawEffectivePrincipal::None,
            context.owner_account.owner,
            "manage_neuron",
            encode_args((manage_neuron_args,)).unwrap(),
        )
        .expect("Failed to call canister");

    let manage_neuron_result =
        Decode!(result.as_slice(), ManageNeuronResponse).map_err(|e| e.to_string())?;

    Ok(manage_neuron_result)
}

pub fn hash(data: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(data);
    hash.finalize().into()
}
