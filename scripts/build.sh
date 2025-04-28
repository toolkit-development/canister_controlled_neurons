cargo test candid -p canister_controlled_neuron

cargo build -p canister_controlled_neuron --release --target wasm32-unknown-unknown

gzip -c target/wasm32-unknown-unknown/release/canister_controlled_neuron.wasm > target/wasm32-unknown-unknown/release/canister_controlled_neuron.wasm.gz

cp target/wasm32-unknown-unknown/release/canister_controlled_neuron.wasm.gz wasm/canister_controlled_neuron.wasm.gz
