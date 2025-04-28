use std::fmt::Display;

use ic_stable_structures::Storable;

use crate::traits::cell::{CellStorage, StaticCellStorageRef};

pub struct GenericCellStorage<V: 'static + Clone + Storable> {
    name: String,
    storage: StaticCellStorageRef<V>,
}

impl<V: 'static + Clone + Storable> GenericCellStorage<V> {
    pub fn new<T: Display>(name: T, storage: StaticCellStorageRef<V>) -> Self {
        Self {
            name: name.to_string(),
            storage,
        }
    }
}

impl<V: 'static + Clone + Storable> CellStorage<V> for GenericCellStorage<V> {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn storage(&self) -> StaticCellStorageRef<V> {
        self.storage
    }
}
