#!/bin/bash

FLAG_SERVER=false
FLAG_CLIENT=false
FLAG_RUN=false
FLAG_CLIENT_SIDE_ONLY=""

POSITIONAL=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --client ) FLAG_CLIENT=true; shift;;
    --server ) FLAG_SERVER=true; shift;;
    --client-side-only ) FLAG_CLIENT_SIDE_ONLY="--features client-side-only"; shift;;
    --run ) FLAG_RUN=true; shift;;
  esac
done

function compile-client() {
  #wasm-pack build --release --target web

  # RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128' \
  cargo build \
     --release \
    --lib \
    --target wasm32-unknown-unknown \
    --target-dir ./build/client \
    # -Z build-std=panic_abort,std \
    $FLAG_CLIENT_SIDE_ONLY
  wasm-bindgen \
    --target web \
    --out-dir pkg \
    --weak-refs \
    ./build/client/wasm32-unknown-unknown/release/punchy.wasm
  echo "Done!"
}

function compile-server() {
  RUSTFLAGS='-C target-cpu=native'
  cargo build \
    --release \
    --bin sports \
    --target-dir ./build/server \
    $FLAG_CLIENT_SIDE_ONLY
  echo "Done!"
}

function run-server() {
  RUSTFLAGS='-C target-cpu=native'
  cargo run \
    --release \
    --bin sports \
    --target-dir ./build/server \
    $FLAG_CLIENT_SIDE_ONLY
  echo "Done!"
}

#--reference-types \
if [[ $FLAG_CLIENT = true ]]; then
  echo "Building client"
  compile-client
  echo "Built client"

  fswatch -or src | while read MODFILE
  do
    compile-client
  done
fi

if [[ $FLAG_SERVER = true ]] && [[ $FLAG_RUN = false ]]; then
  echo "Building server"
  compile-server
  echo "Built server"

  fswatch -or src | while read MODFILE
  do
    compile-server
  done
fi

if [[ $FLAG_SERVER = true ]] && [[ $FLAG_RUN = true ]]; then
  echo "Building server"
  run-server
  echo "Built server"

  fswatch -or src | while read MODFILE
  do
    run-server
  done
fi