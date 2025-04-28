pub trait Sorter<K, V>: candid::CandidType + Clone + Send + Sync
where
    K: 'static + candid::CandidType + Ord + Clone + Send + Sync,
    V: candid::CandidType,
{
    fn sort(&self, values: Vec<(K, V)>) -> Vec<(K, V)>;
}

pub trait Filter<K, V>: candid::CandidType + Clone + Send + Sync
where
    K: 'static + candid::CandidType + Ord + Clone + Send + Sync,
    V: candid::CandidType,
{
    fn matches(&self, key: &K, value: &V) -> bool;
}
