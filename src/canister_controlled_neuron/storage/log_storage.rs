use toolkit_utils::{
    storage::{Storage, StorageInsertable, StorageQueryable, StorageUpdateable},
    StaticStorageRef,
};

use super::storages::LOG;

pub struct LogStore;

impl Storage<u64, String> for LogStore {
    const NAME: &'static str = "log";

    fn storage() -> StaticStorageRef<u64, String> {
        &LOG
    }
}

impl StorageQueryable<u64, String> for LogStore {}
impl StorageUpdateable<u64, String> for LogStore {}
impl StorageInsertable<String> for LogStore {}

impl LogStore {}
