use std::{cell::RefCell, thread::LocalKey};

use ic_stable_structures::{
    memory_manager::VirtualMemory, DefaultMemoryImpl, StableBTreeMap, Storable,
};

use crate::types::{api_error::ApiError, result::CanisterResult};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;
pub type StorageRef<K, V> = RefCell<StableBTreeMap<K, V, Memory>>;
pub type StaticStorageRef<K, V> = &'static LocalKey<StorageRef<K, V>>;

pub trait Storage<K: Storable + Ord + Clone, V: Storable + Clone> {
    const NAME: &'static str;
    fn storage() -> StaticStorageRef<K, V>;
}

pub trait StorageQueryable<K, V>: Storage<K, V>
where
    K: 'static + Storable + Ord + Clone,
    V: 'static + Storable + Clone,
{
    /// Get a single entity by key
    /// # Arguments
    /// * `key` - The key of the entity to get
    /// # Returns
    /// * `Result<(K, V), ApiError>` - The entity if found, otherwise an error
    fn get(key: K) -> CanisterResult<(K, V)> {
        Self::storage().with(|data| {
            data.borrow()
                .get(&key)
                .ok_or(
                    ApiError::not_found("")
                        .add_method_name("get")
                        .add_info(Self::NAME)
                        .add_info("storage"),
                )
                .map(|value| (key, value))
        })
    }

    /// Get multiple entities by key
    /// # Arguments
    /// * `keys` - The keys of the entities to get
    /// # Returns
    /// * `Vec<(K, V)>` - The entities if found, otherwise an empty vector
    fn get_many(keys: Vec<K>) -> Vec<(K, V)> {
        Self::storage().with(|data| {
            let mut entities = Vec::new();
            for key in keys {
                if let Some(value) = data.borrow().get(&key) {
                    entities.push((key, value));
                }
            }
            entities
        })
    }

    /// Get all entities by key
    /// # Returns
    /// * `Vec<(K, V)>` - The entities if found, otherwise an empty vector
    fn get_all() -> Vec<(K, V)> {
        Self::storage().with(|data| data.borrow().iter().collect())
    }

    /// Find a single entity by filter
    /// # Arguments
    /// * `filter` - The filter to apply
    /// # Returns
    /// * `Option<(K, V)>` - The entity if found, otherwise None
    fn find<F>(filter: F) -> Option<(K, V)>
    where
        F: Fn(&K, &V) -> bool,
    {
        Self::storage().with(|data| data.borrow().iter().find(|(id, value)| filter(id, value)))
    }

    /// Find all entities by filter
    /// # Arguments
    /// * `filter` - The filter to apply
    /// # Returns
    /// * `Vec<(K, V)>` - The entities if found, otherwise an empty vector
    fn filter<F>(filter: F) -> Vec<(K, V)>
    where
        F: Fn(&K, &V) -> bool,
    {
        Self::storage().with(|data| {
            data.borrow()
                .iter()
                .filter(|(id, value)| filter(id, value))
                .collect()
        })
    }
}

pub trait StorageInsertable<V>: Storage<u64, V>
where
    V: 'static + Storable + Clone,
{
    /// Insert a single entity with an iterating key
    fn insert(value: V) -> CanisterResult<(u64, V)> {
        Self::storage().with(|data| {
            let key = data
                .borrow()
                .last_key_value()
                .map(|(k, _)| k + 1)
                .unwrap_or_else(|| 1);

            if data.borrow().contains_key(&key) {
                return Err(ApiError::duplicate("Key already exists")
                    .add_method_name("insert")
                    .add_info(Self::NAME)
                    .add_info("storage"));
            }

            data.borrow_mut().insert(key, value.clone());
            Ok((key, value))
        })
    }
}

pub trait StorageInsertableByKey<K, V>: Storage<K, V>
where
    K: 'static + Storable + Ord + Clone,
    V: 'static + Storable + Clone,
{
    /// Insert a single entity by key
    fn insert_by_key(key: K, value: V) -> CanisterResult<(K, V)> {
        Self::storage().with(|data| {
            if data.borrow().contains_key(&key) {
                return Err(ApiError::duplicate("Key already exists")
                    .add_method_name("insert_by_key")
                    .add_info(Self::NAME)
                    .add_info("storage"));
            }

            data.borrow_mut().insert(key.clone(), value.clone());
            Ok((key, value))
        })
    }

    fn upsert_by_key(key: K, value: V) -> (K, V) {
        Self::storage().with(|data| {
            data.borrow_mut().insert(key.clone(), value.clone());
            (key, value)
        })
    }
}

pub trait StorageUpdateable<K, V>: Storage<K, V>
where
    K: 'static + Storable + Ord + Clone,
    V: 'static + Storable + Clone,
{
    fn update(key: K, value: V) -> CanisterResult<(K, V)> {
        Self::storage().with(|data| {
            if !data.borrow().contains_key(&key) {
                return Err(ApiError::not_found("Key does not exist")
                    .add_method_name("update")
                    .add_info(Self::NAME)
                    .add_info("storage"));
            }

            data.borrow_mut().insert(key.clone(), value.clone());
            Ok((key, value))
        })
    }

    fn remove(key: K) -> bool {
        Self::storage().with(|data| data.borrow_mut().remove(&key).is_some())
    }

    fn remove_many(keys: Vec<K>) {
        Self::storage().with(|data| {
            for key in keys {
                data.borrow_mut().remove(&key);
            }
        })
    }
}
