use ic_cdk::query;
use std::env;

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
