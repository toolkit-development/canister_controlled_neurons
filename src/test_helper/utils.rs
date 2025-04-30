use candid::Principal;
use rand::{rng, Rng};

pub fn generate_principal() -> Principal {
    let random_bytes: Vec<u8> = (0..29)
        .map(|_| {
            let this = &mut rng();
            this.random()
        })
        .collect();
    Principal::from_slice(&random_bytes)
}
