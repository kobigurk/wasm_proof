cargo +nightly build --target=wasm32-unknown-unknown --lib --release
wasm-bindgen ../target/wasm32-unknown-unknown/release/web_proof.wasm --out-dir
