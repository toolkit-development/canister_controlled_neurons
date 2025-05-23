pub fn create_sns_payload(owner: Principal) -> CreateServiceNervousSystem {
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