use candid::{encode_args, Decode, Principal};
use ic_ledger_types::Subaccount;
use pocket_ic::PocketIc;
use toolkit_utils::ic_ledger_types::MAINNET_GOVERNANCE_CANISTER_ID;
use toolkit_utils::icrc_ledger_types::icrc1::account::Account;

use crate::declarations::icp_governance_api::{
    By, ClaimOrRefresh, ClaimOrRefreshResponse, Command1, Configure, Countries,
    CreateServiceNervousSystem, DeveloperDistribution, DissolveState, Duration,
    GovernanceParameters, Image, IncreaseDissolveDelay, InitialTokenDistribution, LedgerParameters,
    MakeProposalRequest, MakeProposalResponse, NeuronBasketConstructionParameters,
    NeuronDistribution, NeuronId as IcpNeuronId, Operation, Percentage, ProposalActionRequest,
    Result2, SwapDistribution, SwapParameters, Tokens, VotingRewardParameters,
};
use crate::declarations::sns_governance_api::{
    Command, GetProposal, GetProposalResponse, ListNeurons, ListNeuronsResponse, ManageNeuron,
    ManageNeuronResponse, Neuron as SnsNeuron, NeuronId as SnsNeuronId, ProposalId,
};
use crate::declarations::snsw_api::{
    DeployedSns, GetDeployedSnsByProposalIdRequest, GetDeployedSnsByProposalIdResponse,
    GetDeployedSnsByProposalIdResult, GetSnsSubnetIdsArg, GetSnsSubnetIdsResponse,
    UpdateSnsSubnetListRequest, UpdateSnsSubnetListResponse,
};
use crate::declarations::swap_api::{
    GetBuyerStateRequest, GetBuyerStateResponse, GetLifecycleArg, GetLifecycleResponse,
    NewSaleTicketRequest, NewSaleTicketResponse, RefreshBuyerTokensRequest,
    RefreshBuyerTokensResponse,
};
use crate::sender::Sender;
use crate::utils::generate_principal;
use crate::{context::Context, declarations::icp_governance_api::ManageNeuronCommandRequest};
use sha2::{Digest, Sha256};

pub static SNSW_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";
pub static DEVELOPER_ICP: u64 = 100_000_000_000;
pub static PARTICIPANT_ICP: u64 = 100_000_000_000;
pub struct SnsContext {
    pub icp_neuron_id: Option<IcpNeuronId>,
    pub sns_neurons: Vec<SnsNeuron>,
    pub sns_canisters: DeployedSns,
    pub participants: Vec<Principal>,
}

impl SnsContext {
    pub fn new(context: &Context) -> Self {
        let developer_icp = context.get_icp_balance(context.owner_account.owner);
        assert!(developer_icp.is_ok());
        if developer_icp.unwrap() < DEVELOPER_ICP {
            context.mint_icp(DEVELOPER_ICP, context.owner_account.owner);
        }

        let icp_neuron_id = Self::create_icp_neuron(context);
        Self::set_max_dissolve_delay(context, icp_neuron_id.clone());
        Self::set_correct_subnet_for_sns_w_canister(context);

        let sns_data = Self::get_sns_payload(context.owner_account.owner);
        let deployed_sns = Self::create_sns(context, icp_neuron_id.clone(), sns_data.clone());

        Self::participate_in_sns_sale(context, deployed_sns.clone(), 5);

        Self::finalize_sns_sale(context, deployed_sns.clone());

        let sns_neurons = Self::get_sns_neurons(context, deployed_sns.clone())
            .expect("Failed to get sns neuron id");

        SnsContext {
            icp_neuron_id,
            sns_canisters: deployed_sns,
            participants: vec![],
            sns_neurons: sns_neurons.neurons,
        }
    }

    pub fn get_sns_proposal(
        &self,
        pic: &PocketIc,
        proposal_id: Option<ProposalId>,
        sender: Sender,
    ) -> Result<GetProposalResponse, String> {
        let args = GetProposal { proposal_id };

        let res = pic.query_call(
            self.sns_canisters.governance_canister_id.unwrap(),
            sender.principal(),
            "get_proposal",
            encode_args((args,)).unwrap(),
        );

        match res {
            Ok(res) => Decode!(res.as_slice(), GetProposalResponse).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn sns_command(
        &self,
        pic: &PocketIc,
        neuron_id: SnsNeuronId,
        command: Command,
        sender: Sender,
    ) -> Result<ManageNeuronResponse, String> {
        let manage_neuron_args = ManageNeuron {
            subaccount: neuron_id.id.to_vec(),
            command: Some(command),
        };

        let res = pic.update_call(
            self.sns_canisters.governance_canister_id.unwrap(),
            sender.principal(),
            "manage_neuron",
            encode_args((manage_neuron_args,)).unwrap(),
        );

        match res {
            Ok(res) => Decode!(res.as_slice(), ManageNeuronResponse).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn create_icp_neuron(context: &Context) -> Option<IcpNeuronId> {
        Self::get_start_function_print("create_icp_neuron");

        let memo = 1;
        let subaccount = Self::generate_subaccount_by_nonce(memo, context.owner_account.owner);
        context.mint_icp(DEVELOPER_ICP, context.owner_account.owner);

        context.transfer_icp(
            DEVELOPER_ICP,
            Account {
                owner: context.owner_account.owner,
                subaccount: None,
            },
            Account {
                owner: MAINNET_GOVERNANCE_CANISTER_ID,
                subaccount: Some(subaccount),
            },
        );

        let icp_transfer_to_subaccount_for_icp_neuron =
            context.get_icp_balance_with_subaccount(MAINNET_GOVERNANCE_CANISTER_ID, subaccount);
        println!(
            "icp_transfer_to_subaccount_for_icp_neuron: {:?}",
            icp_transfer_to_subaccount_for_icp_neuron
        );
        println!("--------------------------------");
        assert!(icp_transfer_to_subaccount_for_icp_neuron.is_ok());
        assert_eq!(
            icp_transfer_to_subaccount_for_icp_neuron.unwrap(),
            DEVELOPER_ICP
        );

        let claim_neuron_command = ManageNeuronCommandRequest::ClaimOrRefresh(ClaimOrRefresh {
            by: Some(By::Memo(memo)),
        });

        let manage_neuron_result = context
            .nns_command(None, claim_neuron_command, Sender::Owner)
            .expect("Failed to claim neuron");

        let neuron_id = match manage_neuron_result.command.unwrap() {
            Command1::ClaimOrRefresh(ClaimOrRefreshResponse {
                refreshed_neuron_id,
            }) => refreshed_neuron_id,
            _ => panic!("Invalid command"),
        };

        assert!(neuron_id.is_some());
        println!("icp_neuron_id: {:?}", neuron_id);
        Self::get_end_function_print();
        neuron_id
    }

    fn set_max_dissolve_delay(context: &Context, icp_neuron_id: Option<IcpNeuronId>) {
        Self::get_start_function_print("set_max_dissolve_delay_for_icp_neuron");

        let configure_command = ManageNeuronCommandRequest::Configure(Configure {
            operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
                additional_dissolve_delay_seconds: 8 * 365 * 24 * 60 * 60, // 8 years
            })),
        });

        let set_dissolve_delay_result = context
            .nns_command(icp_neuron_id.clone(), configure_command, Sender::Owner)
            .expect("Failed to configure neuron");

        assert!(matches!(
            set_dissolve_delay_result.command,
            Some(Command1::Configure {})
        ));

        println!("dissolve_delay_result: {:?}", set_dissolve_delay_result);
        println!("--------------------------------");
        // check if the dissolve delay is increased
        // get new neuron
        let get_neuron_result = context
            .pic
            .query_call(
                MAINNET_GOVERNANCE_CANISTER_ID,
                context.owner_account.owner,
                "get_full_neuron",
                encode_args((icp_neuron_id.clone().unwrap().id,)).unwrap(),
            )
            .expect("Failed to call canister");

        let get_neuron_result = Decode!(get_neuron_result.as_slice(), Result2)
            .map_err(|e| e.to_string())
            .expect("Failed to decode neuron");

        assert!(matches!(get_neuron_result, Result2::Ok(_)));

        let neuron = match get_neuron_result {
            Result2::Ok(neuron) => {
                assert!(matches!(
                    neuron.dissolve_state,
                    Some(DissolveState::DissolveDelaySeconds(252460800))
                ));
                neuron
            }
            _ => panic!("Invalid result"),
        };

        println!(
            "updated neuron with dissolve delay: {:?}",
            neuron.dissolve_state
        );
        Self::get_end_function_print();
    }

    fn set_correct_subnet_for_sns_w_canister(context: &Context) {
        Self::get_start_function_print("set_correct_subnets_for_sns_w_canister");

        let args = GetSnsSubnetIdsArg {};

        let subnet_ids = context
            .pic
            .query_call(
                Principal::from_text(SNSW_CANISTER_ID).unwrap(),
                context.owner_account.owner,
                "get_sns_subnet_ids",
                encode_args((args,)).unwrap(),
            )
            .expect("Failed to call canister");

        let subnet_ids = Decode!(subnet_ids.as_slice(), GetSnsSubnetIdsResponse)
            .map_err(|e| e.to_string())
            .expect("Failed to decode subnet ids");

        println!(
            "current subnet_id: {:?}",
            subnet_ids.sns_subnet_ids[0].to_string()
        );
        println!("--------------------------------");

        let args = UpdateSnsSubnetListRequest {
            sns_subnet_ids_to_add: vec![context.pic.topology().get_sns().unwrap()],
            sns_subnet_ids_to_remove: subnet_ids.sns_subnet_ids,
        };

        let updated_subnet_ids = context
            .pic
            .update_call(
                Principal::from_text(SNSW_CANISTER_ID).unwrap(),
                MAINNET_GOVERNANCE_CANISTER_ID,
                "update_sns_subnet_list",
                encode_args((args,)).unwrap(),
            )
            .expect("Failed to call canister");

        let updated_subnet_ids =
            Decode!(updated_subnet_ids.as_slice(), UpdateSnsSubnetListResponse)
                .map_err(|e| e.to_string())
                .expect("Failed to decode updated subnet ids");

        println!("updated subnet_id: {:?}", updated_subnet_ids);
        println!("--------------------------------");
        Self::get_end_function_print();
    }

    fn create_sns(
        context: &Context,
        neuron_id: Option<IcpNeuronId>,
        sns_data: CreateServiceNervousSystem,
    ) -> DeployedSns {
        Self::get_start_function_print("create_sns");

        context.pic.add_cycles(
            Principal::from_text(SNSW_CANISTER_ID).unwrap(),
            200_000_000_000_000_000,
        );

        let proposal = MakeProposalRequest {
            url: "".to_string(),
            title: Some("Deploy Toolkit SNS".to_string()),
            action: Some(ProposalActionRequest::CreateServiceNervousSystem(sns_data)),
            summary: "Deploy Toolkit SNS summary".to_string(),
        };

        let create_sns_proposal_result = context
            .nns_command(
                neuron_id.clone(),
                ManageNeuronCommandRequest::MakeProposal(proposal),
                Sender::Owner,
            )
            .expect("Failed to create sns");

        println!("sns_result: {:?}", create_sns_proposal_result);
        println!("--------------------------------");

        let proposal_id = match create_sns_proposal_result.command {
            Some(Command1::MakeProposal(MakeProposalResponse {
                proposal_id,
                message: _,
            })) => proposal_id.unwrap(),
            _ => panic!("Invalid command"),
        };

        let proposal = context
            .get_icp_proposal(proposal_id.id, Sender::Owner)
            .expect("failed to get proposal");

        assert!(proposal.is_some());
        println!("proposal: {:?}", proposal);

        let mut executed_timestamp = 0;

        while executed_timestamp == 0 {
            context
                .pic
                .advance_time(core::time::Duration::from_secs(60 * 60));
            context.pic.tick();

            let proposal = context
                .get_icp_proposal(proposal_id.id, Sender::Owner)
                .expect("failed to get proposal");

            executed_timestamp = proposal.unwrap().executed_timestamp_seconds;
        }

        assert!(executed_timestamp > 0);
        println!("executed_timestamp: {:?}", executed_timestamp);
        println!("--------------------------------");

        let args = GetDeployedSnsByProposalIdRequest {
            proposal_id: proposal_id.id,
        };

        let deployed_sns = context
            .pic
            .query_call(
                Principal::from_text(SNSW_CANISTER_ID).unwrap(),
                context.owner_account.owner,
                "get_deployed_sns_by_proposal_id",
                encode_args((args,)).unwrap(),
            )
            .expect("Failed to call canister");

        let deployed_sns = Decode!(deployed_sns.as_slice(), GetDeployedSnsByProposalIdResponse)
            .map_err(|e| e.to_string())
            .expect("Failed to decode deployed sns");

        let deployed_sns = match deployed_sns.get_deployed_sns_by_proposal_id_result {
            Some(GetDeployedSnsByProposalIdResult::DeployedSns(deployed_sns)) => deployed_sns,
            _ => panic!("Invalid result"),
        };

        println!("deployed_sns: {:?}", deployed_sns);
        println!("--------------------------------");
        Self::get_end_function_print();

        deployed_sns
    }

    fn participate_in_sns_sale(context: &Context, deployed_sns: DeployedSns, participants: u64) {
        Self::get_start_function_print("participate_in_sns_sale");

        for x in 0..participants {
            let participant = if x == 0 {
                context.owner_account.owner
            } else {
                generate_principal()
            };
            context.mint_icp(PARTICIPANT_ICP + 10_000, participant);

            context.pic.tick();

            let subaccount = Subaccount::from(participant);
            let args = NewSaleTicketRequest {
                amount_icp_e8s: PARTICIPANT_ICP - 10_000,
                subaccount: Some(subaccount.0.to_vec()),
            };

            let swap_ticket = context
                .pic
                .update_call(
                    deployed_sns.swap_canister_id.unwrap(),
                    participant,
                    "new_sale_ticket",
                    encode_args((args,)).unwrap(),
                )
                .expect("Failed to call canister");

            let new_sale_ticket_result = Decode!(swap_ticket.as_slice(), NewSaleTicketResponse)
                .map_err(|e| e.to_string())
                .expect("Failed to decode new sale ticket");

            println!("new_sale_ticket_result: {:?}", new_sale_ticket_result);
            println!("--------------------------------");

            context.transfer_icp(
                PARTICIPANT_ICP,
                Account {
                    owner: participant,
                    subaccount: None,
                },
                Account {
                    owner: deployed_sns.swap_canister_id.unwrap(),
                    subaccount: Some(subaccount.0),
                },
            );

            let balance = context.get_icp_balance_with_subaccount(
                deployed_sns.swap_canister_id.unwrap(),
                subaccount.0,
            );
            println!("balance: {:?}", balance);
            println!("--------------------------------");
            assert!(balance.is_ok());
            assert!(balance.unwrap() == PARTICIPANT_ICP);

            let args = RefreshBuyerTokensRequest {
                confirmation_text: None,
                buyer: participant.to_string(),
            };

            let refresh = context
                .pic
                .update_call(
                    deployed_sns.swap_canister_id.unwrap(),
                    participant,
                    "refresh_buyer_tokens",
                    encode_args((args,)).unwrap(),
                )
                .expect("Failed to call canister");

            let refresh_result = Decode!(refresh.as_slice(), RefreshBuyerTokensResponse)
                .map_err(|e| e.to_string())
                .expect("Failed to decode refresh buyer tokens");

            println!("refresh_result: {:?}", refresh_result);
            println!("--------------------------------");

            let args = GetBuyerStateRequest {
                principal_id: Some(participant),
            };

            let get_buyer_state = context
                .pic
                .query_call(
                    deployed_sns.swap_canister_id.unwrap(),
                    participant,
                    "get_buyer_state",
                    encode_args((args,)).unwrap(),
                )
                .expect("Failed to call canister");

            let get_buyer_state_result = Decode!(get_buyer_state.as_slice(), GetBuyerStateResponse)
                .map_err(|e| e.to_string())
                .expect("Failed to decode get buyer state");

            println!("get_buyer_state_result: {:?}", get_buyer_state_result);
            println!("--------------------------------");
        }
    }

    fn finalize_sns_sale(context: &Context, deployed_sns: DeployedSns) {
        Self::get_start_function_print("finalize_sns_sale");

        let mut lifecycle: i32 = 0;

        let args = GetLifecycleArg {};

        while lifecycle != 3 {
            context.pic.tick();
            context
                .pic
                .advance_time(core::time::Duration::from_secs(24 * 60 * 60));

            let lifecycle_data = context
                .pic
                .update_call(
                    deployed_sns.swap_canister_id.unwrap(),
                    context.owner_account.owner,
                    "get_lifecycle",
                    encode_args((&args,)).unwrap(),
                )
                .expect("Failed to call canister");

            let lifecycle_result = Decode!(lifecycle_data.as_slice(), GetLifecycleResponse)
                .map_err(|e| e.to_string())
                .expect("Failed to decode get lifecycle");

            println!("lifecycle_result: {:?}", lifecycle_result);
            println!("--------------------------------");
            lifecycle = lifecycle_result.lifecycle.unwrap();
        }

        println!("lifecycle: {:?}", lifecycle);
        println!("--------------------------------");
        Self::get_end_function_print();
    }

    fn get_sns_neurons(
        context: &Context,
        deployed_sns: DeployedSns,
    ) -> Result<ListNeuronsResponse, String> {
        let args = ListNeurons {
            of_principal: Some(context.owner_account.owner),
            limit: 100,
            start_page_at: None,
        };
        let neuron_id = context
            .pic
            .query_call(
                deployed_sns.governance_canister_id.unwrap(),
                context.owner_account.owner,
                "list_neurons",
                encode_args((args,)).unwrap(),
            )
            .expect("Failed to call canister");

        let response = Decode!(neuron_id.as_slice(), ListNeuronsResponse)
            .map_err(|e| e.to_string())
            .expect("Failed to decode neuron id");

        Ok(response)
    }

    fn get_sns_payload(owner: Principal) -> CreateServiceNervousSystem {
        CreateServiceNervousSystem {
    name: Some("Toolkit".to_string()),
    description: Some("Toolkit is a versatile suite for managing Service Nervous Systems (SNS) and projects on the Internet Computer. From governance proposals to canister deployment, it empowers users to innovate, collaborate, and decentralize seamlessly.".to_string()),
    url: Some("https://ic-toolkit.app".to_string()),
    logo: Some(Image {
        base64_encoding: Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAFJElEQVR4nG2WT4slZxXGf8+puvd298z0ZDLp9BCTjAlGBnElCAoDLty6cCTgwm8gfoCAe79CNu7cKAiShW6yUBAkKEgEEYOQsZ1BkkxPt9N/771VdR4Xb1Xd23On4HKr3ve8589zznnOq//+8K4YnuDxw+bkuJMAbHZvVW/cnRjAlXmiyQ9uv31YRQBm6Xj/zqMHN48XDkA1zz7xkz9mVL2+hFoYKDaEimqBKT+QleXVoyfjq2RpJSohW4AE".to_string()),
    }),

    fallback_controller_principal_ids: vec![
        owner
    ],
    dapp_canisters: vec![],
    ledger_parameters: Some(LedgerParameters {
        transaction_fee: Some(Tokens { e8s: Some(10_000) }),
        token_symbol: Some("TKT".to_string()),
        token_logo: Some(Image {
            base64_encoding: Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAFJElEQVR4nG2WT4slZxXGf8+puvd298z0ZDLp9BCTjAlGBnElCAoDLty6cCTgwm8gfoCAe79CNu7cKAiShW6yUBAkKEgEEYOQsZ1BkkxPt9N/771VdR4Xb1Xd23On4HKr3ve8589zznnOq//+8K4YnuDxw+bkuJMAbHZvVW/cnRjAlXmiyQ9uv31YRQBm6Xj/zqMHN48XDkA1zz7xkz9mVL2+hFoYKDaEimqBKT+QleXVoyfjq2RpJSohW4AE".to_string()),
        }),
        token_name: Some("Toolkit Token".to_string()),
    }),
    governance_parameters: Some(GovernanceParameters {
        neuron_maximum_dissolve_delay_bonus: Some(Percentage { basis_points: Some(10_000) }),
        neuron_maximum_age_bonus: Some(Percentage { basis_points: Some(2_500) }),
        neuron_minimum_stake: Some(Tokens { e8s: Some(10_000_000) }),
        neuron_maximum_age_for_age_bonus: Some(Duration { seconds: Some(4 * 365 * 24 * 60 * 60) }),
        neuron_maximum_dissolve_delay: Some(Duration { seconds: Some(8 * 365 * 24 * 60 * 60) }),
        neuron_minimum_dissolve_delay_to_vote: Some(Duration { seconds: Some(30 * 24 * 60 * 60) }),
        proposal_initial_voting_period: Some(Duration { seconds: Some(4 * 24 * 60 * 60) }),
        proposal_wait_for_quiet_deadline_increase: Some(Duration { seconds: Some(24 * 60 * 60) }),
        proposal_rejection_fee: Some(Tokens { e8s: Some(100_000_000) }),
        voting_reward_parameters: Some(VotingRewardParameters {
            initial_reward_rate: Some(Percentage { basis_points: Some(1000) }),
            final_reward_rate: Some(Percentage { basis_points: Some(225) }),
            reward_rate_transition_duration: Some(Duration { seconds: Some(12 * 365 * 24 * 60 * 60) }),
        }),
    }),
    swap_parameters: Some(SwapParameters {
        minimum_participants: Some(5),
        neurons_fund_participation: Some(false),
        minimum_direct_participation_icp: Some(Tokens { e8s: Some(100_000_000) }),
        maximum_direct_participation_icp: Some(Tokens { e8s: Some(1_000_000_000_000) }),
        minimum_participant_icp: Some(Tokens { e8s: Some(100_000_000_000) }),
        maximum_participant_icp: Some(Tokens { e8s: Some(1_000_000_000_000) }),
        confirmation_text: None,
        minimum_icp: None,
        maximum_icp: None,
        neurons_fund_investment_icp: None,
        restricted_countries: Some(Countries {
            iso_codes: vec!["AQ".to_string()],
        }),
        start_time: None,
        duration: Some(Duration { seconds: Some(7 * 24 * 60 * 60) }),
        neuron_basket_construction_parameters: Some(NeuronBasketConstructionParameters {
            count: Some(3),
            dissolve_delay_interval: Some(Duration { seconds: Some(30 * 24 * 60 * 60) }),
        }),
    }),
    initial_token_distribution: Some(InitialTokenDistribution {
        treasury_distribution: Some(SwapDistribution {
            total: Some(Tokens { e8s: Some(3_000_000_000) }),
        }),
        developer_distribution: Some(DeveloperDistribution {
            developer_neurons: vec![NeuronDistribution {
                controller: Some(owner),
                dissolve_delay: Some(Duration { seconds: Some(2 * 365 * 24 * 60 * 60) }),
                memo: Some(0),
                vesting_period: Some(Duration { seconds: Some(4 * 365 * 24 * 60 * 60) }),
                stake: Some(Tokens { e8s: Some(1_000_000_000) }),
            }],
        }),
        swap_distribution: Some(SwapDistribution {
            total: Some(Tokens { e8s: Some(1_000_000_000) }),
        }),
    }),
    }
    }

    fn generate_subaccount_by_nonce(nonce: u64, canister_id: Principal) -> [u8; 32] {
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

    fn get_start_function_print(function_name: &str) {
        println!("--------------------------------");
        println!("--------NEW FUNCTION------------");
        println!("--------------------------------");
        println!("{}", function_name);
        println!("--------------------------------");
    }

    fn get_end_function_print() {
        println!("--------------------------------");
        println!("----------END FUNCTION----------");
        println!(" ");
        println!(" ");
    }
}
