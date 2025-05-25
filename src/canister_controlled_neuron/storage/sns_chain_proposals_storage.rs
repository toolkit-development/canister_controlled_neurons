use toolkit_utils::{
    storage::{Storage, StorageInsertable, StorageQueryable, StorageUpdateable},
    StaticStorageRef,
};

use crate::types::sns_chain_proposals::SnsChainProposals;

use super::storages::SNS_CHAIN_PROPOSALS;

pub struct SnsChainProposalsStore;

impl Storage<u64, SnsChainProposals> for SnsChainProposalsStore {
    const NAME: &'static str = "sns_chain_proposals";

    fn storage() -> StaticStorageRef<u64, SnsChainProposals> {
        &SNS_CHAIN_PROPOSALS
    }
}

impl StorageQueryable<u64, SnsChainProposals> for SnsChainProposalsStore {}
impl StorageUpdateable<u64, SnsChainProposals> for SnsChainProposalsStore {}
impl StorageInsertable<SnsChainProposals> for SnsChainProposalsStore {}

impl SnsChainProposalsStore {
    pub fn get_latest_key() -> u64 {
        Self::storage().with(|data| data.borrow().last_key_value().map(|(k, _)| k).unwrap_or(0))
    }
}
