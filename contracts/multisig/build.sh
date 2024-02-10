set -e
cd "`dirname $0`"
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/multisig.wasm ../../out/multisig.wasm