use ic_cdk::query;
use std::env;
use toolkit_utils::icrc_ledger_types::icrc::generic_value::Value;

#[query]
fn icts_name() -> String {
    env!("CARGO_PKG_NAME").to_string()
}

#[query]
fn icts_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[query]
fn icts_description() -> String {
    env!("CARGO_PKG_DESCRIPTION").to_string()
}

#[query]
fn icts_metadata() -> Vec<(String, Value)> {
    vec![
        ("icts:name".to_string(), Value::Text(icts_name())),
        ("icts:version".to_string(), Value::Text(icts_version())),
        (
            "icts:description".to_string(),
            Value::Text(icts_description()),
        ),
    ]
}
