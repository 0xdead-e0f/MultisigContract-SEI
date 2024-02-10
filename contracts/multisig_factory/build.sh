set -e
cd "`dirname $0`"
RUSTFLAGS='-C link-arg=-s' cargo wasm
cp ../../target/wasm32-unknown-unknown/release/multisig_factory.wasm ../../out/multisig_factory.wasm