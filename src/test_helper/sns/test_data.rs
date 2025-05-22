use candid::Principal;
use canister_controlled_neuron::api::icp_governance_api::{
    Canister, Countries, CreateServiceNervousSystem, DeveloperDistribution, Duration,
    GlobalTimeOfDay, GovernanceParameters, Image, InitialTokenDistribution, LedgerParameters,
    NeuronBasketConstructionParameters, NeuronDistribution, Percentage, SwapDistribution,
    SwapParameters, Tokens, VotingRewardParameters,
};
use lazy_static::lazy_static;

use crate::sns::create_sns_builder::E8;

pub const ONE_DAY_SECONDS: u64 = 24 * 60 * 60;
pub const ONE_YEAR_SECONDS: u64 = (4 * 365 + 1) * ONE_DAY_SECONDS / 4;
pub const ONE_MONTH_SECONDS: u64 = ONE_YEAR_SECONDS / 12;

// Both are 1 pixel. The first contains #00FF0F. The second is black.
pub const IMAGE_1: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAD0lEQVQIHQEEAPv/AAD/DwIRAQ8HgT3GAAAAAElFTkSuQmCC";
pub const IMAGE_2: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAD0lEQVQIHQEEAPv/AAAAAAAEAAEvUrSNAAAAAElFTkSuQmCC";

lazy_static! {
    pub static ref CREATE_SERVICE_NERVOUS_SYSTEM: CreateServiceNervousSystem = CreateServiceNervousSystem {
        name: Some("Hello, world!".to_string()),
        description: Some("Best app that you ever did saw.".to_string()),
        url: Some("https://best.app".to_string()),
        logo: Some(Image {
            base64_encoding: Some(IMAGE_1.to_string()),
        }),
        fallback_controller_principal_ids: vec![Principal::anonymous()],
        initial_token_distribution: Some(InitialTokenDistribution {
            developer_distribution: Some(DeveloperDistribution {
                developer_neurons: vec![NeuronDistribution {
                    controller: Some(Principal::anonymous()),
                    dissolve_delay: Some(Duration {
                        seconds: Some(ONE_MONTH_SECONDS * 6),
                    }),
                    memo: Some(763535),
                    stake: Some(Tokens { e8s: Some(756575) }),
                    vesting_period: Some(Duration {
                        seconds: Some(0),
                    }),
                }],
            }),
            treasury_distribution: Some(SwapDistribution {
                total: Some(Tokens { e8s: Some(307064) }),
            }),
            swap_distribution: Some(SwapDistribution {
                total: Some(Tokens {
                    e8s: Some(1_840_880_000),
                }),
            }),
        }),
        ledger_parameters: Some(LedgerParameters {
            transaction_fee: Some(Tokens { e8s: Some(11143) }),
            token_name: Some("Most valuable SNS of all time.".to_string()),
            token_symbol: Some("Kanye".to_string()),
            token_logo: Some(Image {
                base64_encoding: Some(IMAGE_2.to_string()),
            }),
        }),
        governance_parameters: Some(GovernanceParameters {
            // Proposal Parameters
            // -------------------
            proposal_rejection_fee: Some(Tokens { e8s: Some(372250) }),
            proposal_initial_voting_period: Some(Duration {
                seconds: Some(709_499),
            }),
            proposal_wait_for_quiet_deadline_increase: Some(Duration {
                seconds: Some(75_891),
            }),

            // Neuron Parameters
            // -----------------
            neuron_minimum_stake: Some(Tokens { e8s: Some(250_000) }),

            neuron_minimum_dissolve_delay_to_vote: Some(Duration {
                seconds: Some(482538),
            }),
            neuron_maximum_dissolve_delay: Some(Duration {
                seconds: Some(ONE_MONTH_SECONDS * 12),
            }),
            neuron_maximum_dissolve_delay_bonus: Some(Percentage {
                basis_points: Some(18_00),
            }),

            neuron_maximum_age_for_age_bonus: Some(Duration {
                seconds: Some(740908),
            }),
            neuron_maximum_age_bonus: Some(Percentage {
                basis_points: Some(54_00),
            }),

            voting_reward_parameters: Some(VotingRewardParameters {
                initial_reward_rate: Some(Percentage {
                    basis_points: Some(25_92),
                }),
                final_reward_rate: Some(Percentage {
                    basis_points: Some(7_40),
                }),
                reward_rate_transition_duration: Some(Duration {
                    seconds: Some(378025),
                }),
            }),
        }),
        dapp_canisters: vec![Canister {
            id: Some(Principal::anonymous())
        }],

        swap_parameters: Some(SwapParameters {
            confirmation_text: Some("Confirm you are a human".to_string()),
            restricted_countries: Some(Countries {
                iso_codes: vec!["CH".to_string()]
            }),

            minimum_participants: Some(50),
            minimum_direct_participation_icp: Some(Tokens {
                e8s: Some(12_300_000_000-6_100_000_000), // Subtract neurons_fund_investment_icp
            }),
            maximum_direct_participation_icp: Some(Tokens {
                e8s: Some(25_000_000_000-6_100_000_000), // Subtract neurons_fund_investment_icp
            }),
            minimum_participant_icp: Some(Tokens {
                e8s:  Some(100_000_000)
            }),
            maximum_participant_icp: Some(Tokens {
                e8s:  Some(10_000_000_000)
            }),
            neuron_basket_construction_parameters: Some(NeuronBasketConstructionParameters {
                count: Some(2),
                dissolve_delay_interval: Some(Duration {
                    seconds: Some(10_001),
                })
            }),
            start_time: Some(GlobalTimeOfDay {
               seconds_after_utc_midnight: Some(0),
            }),
            duration: Some(Duration {
                seconds: Some(604_800),
            }),

            neurons_fund_participation: Some(false),

            // Deprecated fields must not be set.
            neurons_fund_investment_icp: None,
            minimum_icp: None,
            maximum_icp: None,
        })
    };

    pub static ref CREATE_SERVICE_NERVOUS_SYSTEM_WITH_MATCHED_FUNDING: CreateServiceNervousSystem = {
        let swap_parameters = CREATE_SERVICE_NERVOUS_SYSTEM
            .swap_parameters
            .clone()
            .unwrap();
        CreateServiceNervousSystem {
            swap_parameters: Some(SwapParameters {
                minimum_direct_participation_icp: Some(Tokens {
                    e8s: Some(36_000 * E8),
                }),
                maximum_direct_participation_icp: Some(Tokens {
                    e8s: Some(90_000 * E8),
                }),
                minimum_participant_icp: Some(Tokens {
                    e8s: Some(50 * E8),
                }),
                maximum_participant_icp: Some(Tokens {
                    e8s: Some(1_000 * E8),
                }),
                neurons_fund_participation: Some(true),

                // Unset legacy fields
                minimum_icp: None,
                maximum_icp: None,
                neurons_fund_investment_icp: None,
                ..swap_parameters
            }),
            ..CREATE_SERVICE_NERVOUS_SYSTEM.clone()
        }
    };
}
