use ic_cdk::query;
use toolkit_utils::icrc_types::{Icrc28TrustedOriginsResponse, SupportedStandard};

#[query]
pub fn icrc28_trusted_origins() -> Icrc28TrustedOriginsResponse {
    let trusted_origins = vec![
        // temporary trusted origins for testing
        String::from("localhost:5173"),
        String::from("http://localhost:5173"),
        String::from("https://bpzax-jaaaa-aaaal-acpca-cai.icp0.io"),
        String::from("https://bpzax-jaaaa-aaaal-acpca-cai.raw.icp0.io"),
        String::from("https://dev.ic-toolkit.app"),
    ];

    Icrc28TrustedOriginsResponse { trusted_origins }
}

#[query]
pub fn icrc10_supported_standards() -> Vec<SupportedStandard> {
    vec![
        SupportedStandard {
            url: "https://github.com/dfinity/ICRC/blob/main/ICRCs/ICRC-10/ICRC-10.md".to_string(),
            name: "ICRC-10".to_string(),
        },
        SupportedStandard {
            url: "https://github.com/dfinity/wg-identity-authentication/blob/main/topics/icrc_28_trusted_origins.md".to_string(),
            name: "ICRC-28".to_string(),
        },
    ]
}
