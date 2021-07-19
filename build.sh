cargo build --all --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/near_reverse_auction.wasm ./res/contract.wasm