#!/bin/bash
set -e

cd "$(dirname "$0")"

cargo build --target wasm32-unknown-unknown --release

mkdir -p ../ui/public/wasm
cp target/wasm32-unknown-unknown/release/kodama_dsp.wasm ../ui/public/wasm/

echo "WASM build complete: ui/public/wasm/kodama_dsp.wasm"
