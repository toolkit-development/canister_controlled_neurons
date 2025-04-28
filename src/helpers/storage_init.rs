use std::{cell::RefCell, fmt::Display, thread::LocalKey};

use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    Cell, DefaultMemoryImpl, StableBTreeMap, Storable,
};

use crate::traits::{cell::CellStorageRef, storage::StorageRef};

pub type MemoryManagerStorage = RefCell<MemoryManager<DefaultMemoryImpl>>;

pub fn init_memory_manager() -> MemoryManagerStorage {
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()))
}

pub fn init_cell<S: Display, V: 'static + Clone + Storable>(
    memory_manager: &'static LocalKey<MemoryManagerStorage>,
    name: S,
    id: MemoryId,
) -> CellStorageRef<V> {
    let cell = Cell::init(memory_manager.with(|p| p.borrow().get(id)), None)
        .unwrap_or_else(|_| panic!("Failed to initialize {name} cell"));
    RefCell::new(cell)
}

pub fn init_btree<K: Storable + Ord + Clone, V: Storable>(
    memory_manager: &'static LocalKey<MemoryManagerStorage>,
    id: MemoryId,
) -> StorageRef<K, V> {
    RefCell::new(StableBTreeMap::init(
        memory_manager.with(|p| p.borrow().get(id)),
    ))
}
