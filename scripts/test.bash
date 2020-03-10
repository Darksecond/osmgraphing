#!/usr/bin/env bash

set -e

# build
CUR_DIR="$(dirname "$(pwd)${0:1}")"
"${CUR_DIR}/build.sh"

# test
cargo test
cargo run --example dijkstra
cargo run --example parser
cargo fmt -- --check
