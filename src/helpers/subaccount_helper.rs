use ic_cdk::api::canister_self;
use sha2::{Digest, Sha256};

pub fn generate_subaccount_by_nonce(nonce: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update([0x0c]);
    hasher.update(b"neuron-stake");

    hasher.update(canister_self().as_slice());

    hasher.update(nonce.to_be_bytes());

    let hash_result = hasher.finalize();

    let mut subaccount = [0u8; 32];
    subaccount.copy_from_slice(&hash_result[..]);

    subaccount
}
